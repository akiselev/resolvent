use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl SourceSpan {
    pub fn new(source: &str, start: usize, end: usize) -> Self {
        let bounded = start.min(source.len());
        let prefix = &source[..bounded];
        let line = prefix.bytes().filter(|b| *b == b'\n').count() + 1;
        let column = prefix
            .rsplit_once('\n')
            .map_or(prefix.chars().count() + 1, |(_, tail)| {
                tail.chars().count() + 1
            });
        Self {
            start: bounded,
            end: end.min(source.len()).max(bounded),
            line,
            column,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Note,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLabel {
    pub span: SourceSpan,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuggestedFix {
    pub span: SourceSpan,
    pub replacement: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub phase: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary: Option<SourceLabel>,
    #[serde(default)]
    pub related: Vec<SourceLabel>,
    #[serde(default)]
    pub fixes: Vec<SuggestedFix>,
    #[serde(default)]
    pub causal_chain: Vec<String>,
}

impl Diagnostic {
    pub fn error(
        code: impl Into<String>,
        phase: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: DiagnosticSeverity::Error,
            phase: phase.into(),
            message: message.into(),
            primary: None,
            related: Vec::new(),
            fixes: Vec::new(),
            causal_chain: Vec::new(),
        }
    }

    pub fn at(mut self, span: SourceSpan, message: impl Into<String>) -> Self {
        self.primary = Some(SourceLabel {
            span,
            message: message.into(),
        });
        self
    }
}
