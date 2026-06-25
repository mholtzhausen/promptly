//! Parse `<var name="..." type="..." />` placeholders from templates.

use regex::Regex;
use serde::{Deserialize, Serialize};

/// A single variable extracted from a template placeholder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    /// One of: text, number, option, multiline
    pub var_type: VarType,
    pub default_value: String,
    pub label: String,
    pub placeholder: String,
}

/// The kind of input widget to use for a variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VarType {
    Text,
    Number,
    Option(Vec<String>),
    Multiline,
}

static VAR_TAG_RE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"<var\s+([^>]*?)\s*/>").unwrap());

/// Parse all variable placeholders from a template string.
pub fn parse_variables(content: &str) -> Vec<Variable> {
    VAR_TAG_RE
        .captures_iter(content)
        .filter_map(|cap| parse_var_tag_attrs(cap.get(1)?.as_str()))
        .collect()
}

fn parse_var_tag_attrs(attrs_str: &str) -> Option<Variable> {
    let attrs = parse_attributes(attrs_str);
    let name = attrs.get("name")?.trim().to_string();
    if name.is_empty() || !is_valid_name(&name) {
        return None;
    }
    let type_str = attrs.get("type")?.trim();
    if type_str.is_empty() {
        return None;
    }

    let value = attrs
        .get("value")
        .map(|s| s.to_string())
        .unwrap_or_default();
    let label = attrs
        .get("label")
        .map(|s| s.to_string())
        .unwrap_or_default();
    let placeholder = attrs
        .get("placeholder")
        .map(|s| s.to_string())
        .unwrap_or_default();
    let options_raw = attrs
        .get("options")
        .map(|s| s.to_string())
        .unwrap_or_default();

    let var_type = match type_str {
        "number" => VarType::Number,
        "option" => {
            let options: Vec<String> = if options_raw.is_empty() {
                vec![]
            } else {
                options_raw
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            };
            VarType::Option(options)
        }
        "multiline" => VarType::Multiline,
        "text" => VarType::Text,
        _ => return None,
    };

    let default_value = if matches!(var_type, VarType::Option(_)) {
        if !value.is_empty() {
            value
        } else if let VarType::Option(ref opts) = var_type {
            opts.first().cloned().unwrap_or_default()
        } else {
            String::new()
        }
    } else {
        value
    };

    Some(Variable {
        name,
        var_type,
        default_value,
        label,
        placeholder,
    })
}

fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn parse_attributes(attrs_str: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let mut i = 0;
    let bytes = attrs_str.as_bytes();
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let key_start = i;
        while i < bytes.len() && bytes[i] != b'=' && !bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b'=' {
            break;
        }
        let key = &attrs_str[key_start..i];
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b'"' {
            break;
        }
        i += 1;
        let val_start = i;
        while i < bytes.len() {
            if bytes[i] == b'&' {
                i += 1;
                while i < bytes.len() && bytes[i] != b';' {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
            } else if bytes[i] == b'"' {
                break;
            } else {
                i += 1;
            }
        }
        let raw_val = &attrs_str[val_start..i.min(attrs_str.len())];
        if i < bytes.len() {
            i += 1;
        }
        map.insert(key.to_string(), unescape_attr(raw_val));
    }
    map
}

fn unescape_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '&' {
            let mut entity = String::new();
            while let Some(&next) = chars.peek() {
                if next == ';' {
                    chars.next();
                    break;
                }
                entity.push(chars.next().unwrap());
            }
            match entity.as_str() {
                "amp" => out.push('&'),
                "quot" => out.push('"'),
                "lt" => out.push('<'),
                _ => {
                    out.push('&');
                    out.push_str(&entity);
                    out.push(';');
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

/// Serialize a variable tag (stable attribute order). Used by tests and tooling.
#[allow(dead_code)]
pub fn serialize_var(
    name: &str,
    type_str: &str,
    value: &str,
    label: &str,
    placeholder: &str,
    options: &str,
) -> String {
    let mut parts = vec![
        format!(r#"name="{}""#, escape_attr(name)),
        format!(r#"type="{}""#, escape_attr(type_str)),
    ];
    if !value.is_empty() {
        parts.push(format!(r#"value="{}""#, escape_attr(value)));
    }
    if !label.is_empty() {
        parts.push(format!(r#"label="{}""#, escape_attr(label)));
    }
    if !placeholder.is_empty() {
        parts.push(format!(r#"placeholder="{}""#, escape_attr(placeholder)));
    }
    if !options.is_empty() {
        parts.push(format!(r#"options="{}""#, escape_attr(options)));
    }
    format!("<var {} />", parts.join(" "))
}

/// Interpolate a template with provided variable values.
pub fn interpolate(template: &str, values: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (name, value) in values {
        let pattern = format!(r#"(?i)<var\s+[^>]*\bname="{}"[^>]*/>"#, regex::escape(name));
        if let Ok(re) = Regex::new(&pattern) {
            result = re.replace_all(&result, *value).to_string();
        }
    }
    result = VAR_TAG_RE.replace_all(&result, "").to_string();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_variable() {
        let vars = parse_variables(
            r#"Hello <var name="name" type="text" value="world" label="your name" />"#,
        );
        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].name, "name");
        assert!(matches!(vars[0].var_type, VarType::Text));
        assert_eq!(vars[0].default_value, "world");
        assert_eq!(vars[0].label, "your name");
    }

    #[test]
    fn test_parse_multiple_variables() {
        let content = r#"<var name="greeting" type="text" value="Hello" label="Say hi" /> <var name="name" type="text" value="World" label="Who" />!"#;
        let vars = parse_variables(content);
        assert_eq!(vars.len(), 2);
        assert_eq!(vars[0].name, "greeting");
        assert_eq!(vars[1].name, "name");
    }

    #[test]
    fn test_parse_number_type() {
        let vars = parse_variables(
            r#"Count: <var name="num" type="number" value="42" label="how many" />"#,
        );
        assert_eq!(vars.len(), 1);
        assert!(matches!(vars[0].var_type, VarType::Number));
    }

    #[test]
    fn test_parse_option_type() {
        let vars = parse_variables(
            r#"<var name="color" type="option" options="red,green,blue" value="red" label="pick one" />"#,
        );
        assert_eq!(vars.len(), 1);
        if let VarType::Option(opts) = &vars[0].var_type {
            assert_eq!(
                opts,
                &vec!["red".to_string(), "green".to_string(), "blue".to_string()]
            );
        } else {
            panic!("Expected Option type");
        }
        assert_eq!(vars[0].default_value, "red");
    }

    #[test]
    fn test_parse_option_default_falls_back_to_first() {
        let vars = parse_variables(
            r#"<var name="color" type="option" options="red,green,blue" label="pick one" />"#,
        );
        assert_eq!(vars[0].default_value, "red");
    }

    #[test]
    fn test_parse_multiline_type() {
        let vars = parse_variables(
            r#"<var name="body" type="multiline" label="enter text" placeholder="..." />"#,
        );
        assert_eq!(vars.len(), 1);
        assert!(matches!(vars[0].var_type, VarType::Multiline));
        assert_eq!(vars[0].placeholder, "...");
    }

    #[test]
    fn test_parse_no_optionals() {
        let vars = parse_variables(r#"<var name="name" type="text" />"#);
        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].default_value, "");
        assert_eq!(vars[0].label, "");
    }

    #[test]
    fn test_parse_attribute_order_independent() {
        let vars = parse_variables(r#"<var type="text" label="hi" name="x" value="y" />"#);
        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].name, "x");
        assert_eq!(vars[0].default_value, "y");
        assert_eq!(vars[0].label, "hi");
    }

    #[test]
    fn test_parse_escaped_attributes() {
        let vars = parse_variables(r#"<var name="msg" type="text" value="a&amp;b&quot;c" />"#);
        assert_eq!(vars[0].default_value, "a&b\"c");
    }

    #[test]
    fn test_parse_invalid_tags_skipped() {
        let vars = parse_variables(r#"<var type="text" /> <var name="ok" type="text" />"#);
        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].name, "ok");
    }

    #[test]
    fn test_interpolate_simple() {
        let result = interpolate(
            r#"Hello <var name="name" type="text" value="world" />"#,
            &[("name", "Alice")],
        );
        assert_eq!(result, "Hello Alice");
    }

    #[test]
    fn test_interpolate_multiple() {
        let template = r#"<var name="greeting" type="text" value="Hello" />, <var name="name" type="text" value="World" />!"#;
        let result = interpolate(template, &[("greeting", "Hi"), ("name", "Bob")]);
        assert_eq!(result, "Hi, Bob!");
    }

    #[test]
    fn test_interpolate_empty_values() {
        let result = interpolate(
            r#"Hello <var name="name" type="text" value="world" />"#,
            &[],
        );
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
            "Dear <var name=\"name\" type=\"text\" value=\"friend\" />,\n\nThank you.",
            &[("name", "Alice")],
        );
        assert_eq!(result, "Dear Alice,\n\nThank you.");
    }

    #[test]
    fn test_serialize_var_stable_order() {
        let s = serialize_var("n", "text", "v", "l", "p", "");
        assert_eq!(
            s,
            r#"<var name="n" type="text" value="v" label="l" placeholder="p" />"#
        );
    }

    #[test]
    fn test_hyphenated_name() {
        let vars = parse_variables(r#"<var name="my-var" type="text" />"#);
        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].name, "my-var");
    }
}
