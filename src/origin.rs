//! Optional provenance sidecars keyed by stable structural identity.

use std::collections::{HashMap, HashSet};

use thiserror::Error;

use crate::{TermBudget, TermDigest, TermError, TermId, TermStore};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ByteSpan {
    pub start: u64,
    pub end: u64,
}

impl ByteSpan {
    pub fn new(start: u64, end: u64) -> Result<Self, OriginError> {
        if start > end {
            return Err(OriginError::Invalid("byte span is reversed"));
        }
        Ok(Self { start, end })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OriginKind {
    Authored,
    Generated {
        transformation: String,
        parent: Option<TermDigest>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OriginRecord {
    /// Generic source/module/artifact locator owned by the caller.
    pub locator: Option<String>,
    pub byte_span: Option<ByteSpan>,
    pub kind: OriginKind,
    /// Opaque caller-owned identity; Resolvent never interprets it.
    pub consumer_id: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OriginBudget {
    pub max_records_per_term: usize,
    pub max_total_records: usize,
    pub max_text_bytes: usize,
    pub max_work: usize,
}

impl Default for OriginBudget {
    fn default() -> Self {
        Self {
            max_records_per_term: 1_024,
            max_total_records: 1_000_000,
            max_text_bytes: 64 << 20,
            max_work: 100_000,
        }
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum OriginError {
    #[error("invalid origin record: {0}")]
    Invalid(&'static str),
    #[error("origin budget exceeded for {resource}: limit {limit}")]
    BudgetExceeded {
        resource: &'static str,
        limit: usize,
    },
    #[error(transparent)]
    Term(#[from] TermError),
}

/// Zero/one/many origin records outside canonical Term identity.
#[derive(Clone, Debug, Default)]
struct OriginBucket {
    ordered: Vec<OriginRecord>,
    indexed: HashSet<OriginRecord>,
}

#[derive(Clone, Debug, Default)]
pub struct OriginMap {
    records: HashMap<TermDigest, OriginBucket>,
    record_count: usize,
    text_bytes: usize,
}

impl OriginMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn term_count(&self) -> usize {
        self.records.len()
    }

    pub fn record_count(&self) -> usize {
        self.record_count
    }

    pub fn text_bytes(&self) -> usize {
        self.text_bytes
    }

    pub fn records(&self, digest: TermDigest) -> &[OriginRecord] {
        self.records
            .get(&digest)
            .map_or(&[], |bucket| bucket.ordered.as_slice())
    }

    /// Attach one record, deduplicating exact repeats while retaining order.
    pub fn attach_digest(
        &mut self,
        digest: TermDigest,
        record: OriginRecord,
        budget: OriginBudget,
    ) -> Result<bool, OriginError> {
        Ok(self.attach_many_digest(digest, &[record], budget)? == 1)
    }

    /// Atomically attach a bounded batch, deduplicating in expected constant time.
    pub fn attach_many_digest(
        &mut self,
        digest: TermDigest,
        records: &[OriginRecord],
        budget: OriginBudget,
    ) -> Result<usize, OriginError> {
        if records.len() > budget.max_work {
            return Err(exceeded("work", budget.max_work));
        }
        let existing = self.records.get(&digest);
        let mut batch_seen = HashSet::with_capacity(records.len());
        let mut additions = Vec::with_capacity(records.len());
        let mut added_text = 0usize;
        for record in records {
            validate_record(record)?;
            if existing.is_some_and(|bucket| bucket.indexed.contains(record))
                || batch_seen.contains(record)
            {
                continue;
            }
            let text = record_text_bytes(record)?;
            added_text = added_text
                .checked_add(text)
                .ok_or_else(|| exceeded("text bytes", budget.max_text_bytes))?;
            let prospective_text = self
                .text_bytes
                .checked_add(added_text)
                .ok_or_else(|| exceeded("text bytes", budget.max_text_bytes))?;
            if prospective_text > budget.max_text_bytes {
                return Err(exceeded("text bytes", budget.max_text_bytes));
            }
            batch_seen.insert(record.clone());
            additions.push(record.clone());
        }
        let per_term = existing
            .map_or(0, |bucket| bucket.ordered.len())
            .checked_add(additions.len())
            .ok_or_else(|| exceeded("records per term", budget.max_records_per_term))?;
        if per_term > budget.max_records_per_term {
            return Err(exceeded("records per term", budget.max_records_per_term));
        }
        let total = self
            .record_count
            .checked_add(additions.len())
            .ok_or_else(|| exceeded("total records", budget.max_total_records))?;
        if total > budget.max_total_records {
            return Err(exceeded("total records", budget.max_total_records));
        }
        let text = self
            .text_bytes
            .checked_add(added_text)
            .ok_or_else(|| exceeded("text bytes", budget.max_text_bytes))?;
        if text > budget.max_text_bytes {
            return Err(exceeded("text bytes", budget.max_text_bytes));
        }

        let count = additions.len();
        if count != 0 {
            let bucket = self.records.entry(digest).or_default();
            for record in additions {
                bucket.indexed.insert(record.clone());
                bucket.ordered.push(record);
            }
            self.record_count = total;
            self.text_bytes = text;
        }
        Ok(count)
    }

    pub fn attach_term(
        &mut self,
        store: &TermStore,
        term: TermId,
        record: OriginRecord,
        term_budget: TermBudget,
        origin_budget: OriginBudget,
    ) -> Result<bool, OriginError> {
        self.attach_digest(store.digest(term, term_budget)?, record, origin_budget)
    }
}

fn exceeded(resource: &'static str, limit: usize) -> OriginError {
    OriginError::BudgetExceeded { resource, limit }
}

fn record_text_bytes(record: &OriginRecord) -> Result<usize, OriginError> {
    let mut total = 0usize;
    for text in [record.locator.as_ref(), record.consumer_id.as_ref()]
        .into_iter()
        .flatten()
    {
        total = total
            .checked_add(text.len())
            .ok_or_else(|| exceeded("text bytes", usize::MAX))?;
    }
    if let OriginKind::Generated { transformation, .. } = &record.kind {
        total = total
            .checked_add(transformation.len())
            .ok_or_else(|| exceeded("text bytes", usize::MAX))?;
    }
    Ok(total)
}

fn validate_record(record: &OriginRecord) -> Result<(), OriginError> {
    let nonblank =
        |value: &Option<String>| value.as_ref().is_none_or(|value| !value.trim().is_empty());
    if !nonblank(&record.locator) || !nonblank(&record.consumer_id) {
        return Err(OriginError::Invalid(
            "locator and consumer ID must be nonblank",
        ));
    }
    if record.byte_span.is_some() && record.locator.is_none() {
        return Err(OriginError::Invalid("byte span requires a locator"));
    }
    if record.byte_span.is_some_and(|span| span.start > span.end) {
        return Err(OriginError::Invalid("byte span is reversed"));
    }
    match &record.kind {
        OriginKind::Authored if record.locator.is_none() && record.consumer_id.is_none() => Err(
            OriginError::Invalid("authored origin requires a locator or consumer ID"),
        ),
        OriginKind::Generated { transformation, .. } if transformation.trim().is_empty() => Err(
            OriginError::Invalid("generated transformation must be nonblank"),
        ),
        _ => Ok(()),
    }
}
