const DEFAULT_STRING_PREFIX: &str = "msg_";
const DEFAULT_PATTERN_PREFIX: &str = "ptn_";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputMode {
    String {
        prefix: String,
    },
    Pattern {
        prefix: String,
    },
    Both {
        string_prefix: String,
        pattern_prefix: String,
    },
}

impl Default for OutputMode {
    fn default() -> Self {
        Self::String {
            prefix: DEFAULT_STRING_PREFIX.to_string(),
        }
    }
}

impl OutputMode {
    pub fn default_pattern() -> Self {
        Self::Pattern {
            prefix: DEFAULT_PATTERN_PREFIX.to_string(),
        }
    }

    pub fn default_both() -> Self {
        Self::Both {
            string_prefix: DEFAULT_STRING_PREFIX.to_string(),
            pattern_prefix: DEFAULT_PATTERN_PREFIX.to_string(),
        }
    }

    pub fn string_prefix(&self) -> Option<&str> {
        match self {
            Self::String { prefix }
            | Self::Both {
                string_prefix: prefix,
                ..
            } => Some(prefix),
            Self::Pattern { .. } => None,
        }
    }

    pub fn pattern_prefix(&self) -> Option<&str> {
        match self {
            Self::Pattern { prefix }
            | Self::Both {
                pattern_prefix: prefix,
                ..
            } => Some(prefix),
            Self::String { .. } => None,
        }
    }
}
