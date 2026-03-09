use std::fmt;

use schemars::JsonSchema;
use serde::Deserialize;

/// Patch operation to perform on a note target.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Append,
    Prepend,
    Replace,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Append => write!(f, "append"),
            Self::Prepend => write!(f, "prepend"),
            Self::Replace => write!(f, "replace"),
        }
    }
}

/// Type of target within a note to patch.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TargetType {
    Heading,
    Block,
    Frontmatter,
}

impl fmt::Display for TargetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Heading => write!(f, "heading"),
            Self::Block => write!(f, "block"),
            Self::Frontmatter => write!(f, "frontmatter"),
        }
    }
}

/// Parameters for an Obsidian REST API v3 PATCH request.
#[derive(Debug, Clone)]
pub struct PatchParams {
    pub operation: Operation,
    pub target_type: TargetType,
    pub target: String,
    pub target_delimiter: Option<String>,
    pub trim_target_whitespace: Option<bool>,
    pub create_target_if_missing: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_display() {
        assert_eq!(Operation::Append.to_string(), "append");
        assert_eq!(Operation::Prepend.to_string(), "prepend");
        assert_eq!(Operation::Replace.to_string(), "replace");
    }

    #[test]
    fn target_type_display() {
        assert_eq!(TargetType::Heading.to_string(), "heading");
        assert_eq!(TargetType::Block.to_string(), "block");
        assert_eq!(TargetType::Frontmatter.to_string(), "frontmatter");
    }

    #[test]
    fn operation_deserializes_from_lowercase() {
        let op: Operation = serde_json::from_str("\"append\"").unwrap();
        assert!(matches!(op, Operation::Append));

        let op: Operation = serde_json::from_str("\"prepend\"").unwrap();
        assert!(matches!(op, Operation::Prepend));

        let op: Operation = serde_json::from_str("\"replace\"").unwrap();
        assert!(matches!(op, Operation::Replace));
    }

    #[test]
    fn target_type_deserializes_from_lowercase() {
        let tt: TargetType = serde_json::from_str("\"heading\"").unwrap();
        assert!(matches!(tt, TargetType::Heading));

        let tt: TargetType = serde_json::from_str("\"block\"").unwrap();
        assert!(matches!(tt, TargetType::Block));

        let tt: TargetType = serde_json::from_str("\"frontmatter\"").unwrap();
        assert!(matches!(tt, TargetType::Frontmatter));
    }

    #[test]
    fn invalid_operation_fails_deserialization() {
        let result: Result<Operation, _> = serde_json::from_str("\"invalid\"");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_target_type_fails_deserialization() {
        let result: Result<TargetType, _> = serde_json::from_str("\"invalid\"");
        assert!(result.is_err());
    }
}
