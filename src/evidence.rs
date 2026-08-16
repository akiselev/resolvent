/// Independent dimensions of scientific warrant.
///
/// These are intentionally not one linear confidence ladder. A kernel proof can establish a
/// mathematical implication without saying that the model describes a laboratory apparatus;
/// experimental agreement can support model adequacy without proving a universal theorem.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EvidenceAxis {
    Formal,
    Numerical,
    Empirical,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FormalEvidenceGrade {
    Open,
    Assumed,
    Checked,
    CertificateChecked,
    TheoremProved,
    KernelProved,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NumericalEvidenceGrade {
    Open,
    ReferenceCrosschecked,
    NumericallyValidated,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EmpiricalEvidenceGrade {
    Open,
    ExperimentallyValidated,
}

/// The grade carries its axis in the type, making invalid combinations such as
/// "empirical/kernel-proved" unrepresentable.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EvidenceGrade {
    Formal(FormalEvidenceGrade),
    Numerical(NumericalEvidenceGrade),
    Empirical(EmpiricalEvidenceGrade),
}

impl EvidenceGrade {
    pub const fn axis(self) -> EvidenceAxis {
        match self {
            EvidenceGrade::Formal(_) => EvidenceAxis::Formal,
            EvidenceGrade::Numerical(_) => EvidenceAxis::Numerical,
            EvidenceGrade::Empirical(_) => EvidenceAxis::Empirical,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evidence {
    pub grade: EvidenceGrade,
    pub artifact: Option<String>,
    pub note: String,
}

impl Evidence {
    pub fn new(grade: EvidenceGrade, note: impl Into<String>) -> Self {
        Self {
            grade,
            artifact: None,
            note: note.into(),
        }
    }

    pub const fn axis(&self) -> EvidenceAxis {
        self.grade.axis()
    }

    pub fn with_artifact(mut self, artifact: impl Into<String>) -> Self {
        self.artifact = Some(artifact.into());
        self
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EvidenceSet(Vec<Evidence>);

impl EvidenceSet {
    pub fn push(&mut self, evidence: Evidence) {
        self.0.push(evidence);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Evidence> {
        self.0.iter()
    }

    pub fn on_axis(&self, axis: EvidenceAxis) -> impl Iterator<Item = &Evidence> {
        self.0.iter().filter(move |e| e.axis() == axis)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_grade_encodes_axis() {
        let evidence = Evidence::new(
            EvidenceGrade::Empirical(EmpiricalEvidenceGrade::ExperimentallyValidated),
            "measurement agreement",
        );
        assert_eq!(evidence.axis(), EvidenceAxis::Empirical);
    }
}
