//! IPC payload size limits and validation.

pub const MAX_PROMPT_NAME_LEN: usize = 256;
pub const MAX_PROMPT_DESCRIPTION_LEN: usize = 4_096;
pub const MAX_PROMPT_CONTENT_LEN: usize = 256 * 1024;
pub const MAX_HISTORY_CONTENT_LEN: usize = 256 * 1024;
pub const MAX_VARIABLES_PER_TEMPLATE: usize = 128;
pub const MAX_VARIABLE_VALUE_LEN: usize = 64 * 1024;
pub const MAX_PRUNE_KEEP: i64 = 100_000;

pub fn validate_prompt_fields(name: &str, description: &str, content: &str) -> Result<(), String> {
    if name.len() > MAX_PROMPT_NAME_LEN {
        return Err(format!(
            "Prompt name exceeds {MAX_PROMPT_NAME_LEN} characters"
        ));
    }
    if description.len() > MAX_PROMPT_DESCRIPTION_LEN {
        return Err(format!(
            "Prompt description exceeds {MAX_PROMPT_DESCRIPTION_LEN} characters"
        ));
    }
    if content.len() > MAX_PROMPT_CONTENT_LEN {
        return Err(format!(
            "Prompt content exceeds {} bytes",
            MAX_PROMPT_CONTENT_LEN
        ));
    }
    Ok(())
}

pub fn validate_template_content(content: &str) -> Result<(), String> {
    if content.len() > MAX_PROMPT_CONTENT_LEN {
        return Err(format!(
            "Template content exceeds {} bytes",
            MAX_PROMPT_CONTENT_LEN
        ));
    }
    Ok(())
}

pub fn validate_interpolate_values(values: &[(String, String)]) -> Result<(), String> {
    if values.len() > MAX_VARIABLES_PER_TEMPLATE {
        return Err(format!(
            "Too many variables (max {MAX_VARIABLES_PER_TEMPLATE})"
        ));
    }
    for (name, value) in values {
        if name.len() > MAX_PROMPT_NAME_LEN {
            return Err("Variable name too long".into());
        }
        if value.len() > MAX_VARIABLE_VALUE_LEN {
            return Err(format!(
                "Variable '{name}' value exceeds {MAX_VARIABLE_VALUE_LEN} bytes"
            ));
        }
    }
    Ok(())
}

pub fn validate_copy_text(text: &str) -> Result<(), String> {
    if text.len() > MAX_HISTORY_CONTENT_LEN {
        return Err(format!(
            "Copy text exceeds {} bytes",
            MAX_HISTORY_CONTENT_LEN
        ));
    }
    Ok(())
}

pub fn validate_prune_keep(keep: i64) -> Result<(), String> {
    if !(0..=MAX_PRUNE_KEEP).contains(&keep) {
        return Err(format!("Prune keep must be between 0 and {MAX_PRUNE_KEEP}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_oversized_content() {
        let huge = "x".repeat(MAX_PROMPT_CONTENT_LEN + 1);
        assert!(validate_prompt_fields("n", "d", &huge).is_err());
    }

    #[test]
    fn accepts_valid_prompt() {
        assert!(validate_prompt_fields("n", "d", "body").is_ok());
    }
}
