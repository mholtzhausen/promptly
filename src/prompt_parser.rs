//! Parse `{{name|type|default|description}}` placeholders from templates.

use regex::Regex;
use serde::{Deserialize, Serialize};

/// A single variable extracted from a template placeholder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    /// One of: text, number, option, multiline
    pub var_type: VarType,
    pub default_value: String,
    pub description: String,
}

/// The kind of input widget to use for a variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VarType {
    Text,
    Number,
    Option(Vec<String>), // comma-delimited values stored in default field
    Multiline,
}

impl std::fmt::Display for VarType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VarType::Text => write!(f, "text"),
            VarType::Number => write!(f, "number"),
            VarType::Option(_) => write!(f, "option"),
            VarType::Multiline => write!(f, "multiline"),
        }
    }
}

/// Regex for `{{name|type|default|description}}`.
static PLACEHOLDER_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"\{\{(\w+)\|(\w+)(?:\|([^|]*))?(?:\|([^}]*))?}}").unwrap()
});

/// Parse all variable placeholders from a template string.
pub fn parse_variables(content: &str) -> Vec<Variable> {
    PLACEHOLDER_RE
        .captures_iter(content)
        .filter_map(|cap| {
            let name = cap.get(1)?.as_str().to_string();
            let type_str = cap.get(2)?.as_str();
            let default_val = cap.get(3).map_or("", |m| m.as_str()).to_string();
            let description = cap.get(4).map_or("", |m| m.as_str()).to_string();

            let var_type = match type_str {
                "number" => VarType::Number,
                "option" => {
                    // For option types, default field contains comma-separated choices
                    let options: Vec<String> = if default_val.is_empty() {
                        vec![]
                    } else {
                        default_val
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect()
                    };
                    VarType::Option(options)
                }
                "multiline" => VarType::Multiline,
                _ => VarType::Text,
            };

            Some(Variable {
                name,
                var_type,
                default_value: default_val,
                description,
            })
        })
        .collect()
}

/// Interpolate a template with provided variable values.
pub fn interpolate(template: &str, values: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (name, value) in values {
        // Replace all occurrences of {{name|...}} with the actual value
        let pattern = format!(r"\{{\{{{}\|[^\}}]*\}}}}", regex::escape(name));
        if let Ok(re) = Regex::new(&pattern) {
            result = re.replace_all(&result, *value).to_string();
        }
    }
    // Replace any remaining unmatched placeholders with empty string
    result = PLACEHOLDER_RE.replace_all(&result, "").to_string();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_variable() {
        let vars = parse_variables("Hello {{name|text|world|your name}}");
        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].name, "name");
        assert!(matches!(vars[0].var_type, VarType::Text));
        assert_eq!(vars[0].default_value, "world");
        assert_eq!(vars[0].description, "your name");
    }

    #[test]
    fn test_parse_multiple_variables() {
        let content = "{{greeting|text|Hello|Say hi}} {{name|text|World|Who}}!";
        let vars = parse_variables(content);
        assert_eq!(vars.len(), 2);
        assert_eq!(vars[0].name, "greeting");
        assert_eq!(vars[1].name, "name");
    }

    #[test]
    fn test_parse_number_type() {
        let vars = parse_variables("Count: {{num|number|42|how many}}");
        assert_eq!(vars.len(), 1);
        assert!(matches!(vars[0].var_type, VarType::Number));
    }

    #[test]
    fn test_parse_option_type() {
        let vars = parse_variables("Color: {{color|option|red,green,blue|pick one}}");
        assert_eq!(vars.len(), 1);
        if let VarType::Option(opts) = &vars[0].var_type {
            assert_eq!(
                opts,
                &vec!["red".to_string(), "green".to_string(), "blue".to_string()]
            );
        } else {
            panic!("Expected Option type");
        }
    }

    #[test]
    fn test_parse_multiline_type() {
        let vars = parse_variables("Body: {{body|multiline||enter text}}");
        assert_eq!(vars.len(), 1);
        assert!(matches!(vars[0].var_type, VarType::Multiline));
    }

    #[test]
    fn test_parse_no_defaults() {
        let vars = parse_variables("{{name|text||}}");
        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].default_value, "");
        assert_eq!(vars[0].description, "");
    }

    #[test]
    fn test_interpolate_simple() {
        let result = interpolate("Hello {{name|text|world|}}", &[("name", "Alice")]);
        assert_eq!(result, "Hello Alice");
    }

    #[test]
    fn test_interpolate_multiple() {
        let template = "{{greeting|text|Hello|}}, {{name|text|World|}}!";
        let result = interpolate(template, &[("greeting", "Hi"), ("name", "Bob")]);
        assert_eq!(result, "Hi, Bob!");
    }

    #[test]
    fn test_interpolate_empty_values() {
        // No values provided — placeholders removed
        let result = interpolate("Hello {{name|text|world|}}", &[]);
        assert_eq!(result, "Hello ");
    }

    #[test]
    fn test_no_placeholders() {
        let vars = parse_variables("Just plain text");
        assert!(vars.is_empty());
    }

    #[test]
    fn test_interpolate_preserves_non_placeholder_text() {
        let result = interpolate(
            "Dear {{name|text|friend|}},\n\nThank you for your message.",
            &[("name", "Alice")],
        );
        assert_eq!(result, "Dear Alice,\n\nThank you for your message.");
    }
}
