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

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EvidenceGrade {
    /// The claim is declared but has no attached warrant yet.
    Open,
    /// The claim is an explicit modeling assumption or external premise.
    Assumed,
    /// A deterministic structural/checking procedure established the claim.
    Checked,
    /// An independently checkable certificate establishes the claim.
    CertificateChecked,
    /// A theorem instance or proof object establishes the claim.
    TheoremProved,
    /// A trusted kernel checked the theorem/proof term.
    KernelProved,
    /// A numerical reference implementation or oracle independently reproduced the behavior.
    ReferenceCrosschecked,
    /// Convergence, metamorphic, mutation, or other numerical validation supports the claim.
    NumericallyValidated,
    /// Repeated observations under a declared measurement/uncertainty model support adequacy.
    ExperimentallyValidated,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evidence {
    pub axis: EvidenceAxis,
    pub grade: EvidenceGrade,
    pub artifact: Option<String>,
    pub note: String,
}

impl Evidence {
    pub fn new(axis: EvidenceAxis, grade: EvidenceGrade, note: impl Into<String>) -> Self {
        Self {
            axis,
            grade,
            artifact: None,
            note: note.into(),
        }
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
        self.0.iter().filter(move |e| e.axis == axis)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
