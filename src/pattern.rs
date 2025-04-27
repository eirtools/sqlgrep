use crate::error::SQLError;

pub(crate) enum PatternKind {
    Fixed,
    Regex,
}
pub(crate) struct PatternOptions {
    pub case_insensitive: bool,
    pub whole_string: bool,
}

pub(crate) enum Pattern {
    Always,
    Fixed((String, PatternOptions)),
    Regex((regex::Regex, PatternOptions)),
}

impl Pattern {
    pub fn new(
        pattern: &str,
        kind: &PatternKind,
        options: PatternOptions,
    ) -> Result<Self, SQLError> {
        match kind {
            PatternKind::Regex => {
                if pattern.is_empty() || pattern == ".*" {
                    Ok(Self::Always)
                } else {
                    regex::RegexBuilder::new(pattern)
                        .case_insensitive(options.case_insensitive)
                        .build()
                        .map(|value| Self::Regex((value, options)))
                        .map_err(SQLError::Regex)
                }
            }
            PatternKind::Fixed => Ok(if pattern.is_empty() {
                Self::Always
            } else {
                let pattern = if options.case_insensitive {
                    pattern.to_lowercase()
                } else {
                    pattern.to_owned()
                };
                Self::Fixed((pattern, options))
            }),
        }
    }

    pub fn is_match(&self, value: &str) -> bool {
        match self {
            Pattern::Always => true,
            Pattern::Fixed((pattern, options)) => {
                match (options.case_insensitive, options.whole_string) {
                    (true, true) => value.to_lowercase() == *pattern,
                    (false, true) => value == pattern,
                    (true, false) => value.to_lowercase().contains(pattern),
                    (false, false) => value.contains(pattern),
                }
            }
            Pattern::Regex((pattern, options)) => {
                let Some(pattern_match) = pattern.find(value) else {
                    return false;
                };

                if options.whole_string {
                    pattern_match.len() == value.len()
                } else {
                    true
                }
            }
        }
    }
}
