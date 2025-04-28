use crate::error::SQLError;

pub(crate) enum PatternKind {
    Regex,
}

pub(crate) enum Pattern {
    Regex(regex::Regex),
}

impl Pattern {
    pub fn new(pattern: &str, kind: &PatternKind) -> Result<Self, SQLError> {
        match kind {
            PatternKind::Regex => regex::Regex::new(pattern)
                .map(Self::Regex)
                .map_err(SQLError::Regex),
        }
    }

    pub fn is_match(&self, value: &str) -> bool {
        match self {
            Pattern::Regex(regex) => regex.is_match(value),
        }
    }
}
