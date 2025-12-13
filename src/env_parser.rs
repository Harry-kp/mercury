use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Parse result with variables and any warnings encountered
pub struct EnvParseResult {
    pub vars: HashMap<String, String>,
    pub warnings: Vec<String>,
}

pub fn parse_env_file(path: &Path) -> Result<EnvParseResult, std::io::Error> {
    let content = fs::read_to_string(path)?;
    let mut vars = HashMap::new();
    let mut warnings = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse KEY=VALUE
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();

            // Proper quote handling: only strip if both ends have matching quotes and len >= 2
            let value = value.trim();
            let value = if value.len() >= 2
                && ((value.starts_with('"') && value.ends_with('"'))
                    || (value.starts_with('\'') && value.ends_with('\'')))
            {
                value[1..value.len() - 1].to_string()
            } else {
                value.to_string()
            };

            // Warn on duplicate keys
            if vars.contains_key(&key) {
                warnings.push(format!("Line {}: duplicate key '{}'", line_num + 1, key));
            }

            vars.insert(key, value);
        } else {
            // Malformed line - no = sign
            warnings.push(format!("Line {}: skipped (no '=' found)", line_num + 1));
        }
    }

    Ok(EnvParseResult { vars, warnings })
}

pub fn substitute_variables(text: &str, variables: &HashMap<String, String>) -> String {
    let mut result = text.to_string();

    for (key, value) in variables {
        let pattern = format!("{{{{{}}}}}", key);
        result = result.replace(&pattern, value);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_variables() {
        let mut vars = HashMap::new();
        vars.insert("host".to_string(), "api.example.com".to_string());
        vars.insert("token".to_string(), "abc123".to_string());

        let input = "https://{{host}}/users?token={{token}}";
        let output = substitute_variables(input, &vars);

        assert_eq!(output, "https://api.example.com/users?token=abc123");
    }
}
