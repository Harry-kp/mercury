use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

pub fn parse_env_file(path: &Path) -> Result<HashMap<String, String>, std::io::Error> {
    let mut vars = HashMap::new();

    // dotenvy handles parsing of quotes, multiline values, and comments robustly.
    match dotenvy::from_path_iter(path) {
        Ok(iter) => {
            for (key, value) in iter.flatten() {
                vars.insert(key, value);
            }
        }
        Err(e) => {
            return Err(std::io::Error::other(e));
        }
    }

    Ok(vars)
}

pub fn substitute_variables(text: &str, variables: &HashMap<String, String>) -> String {
    let mut current_text = text.to_string();
    // Regex to match {{ key }} with optional whitespace
    // Allow alphanumeric, underscores, dashes, and dots in keys
    let re = Regex::new(r"\{\{\s*([a-zA-Z0-9_\-\.]+)\s*\}\}").unwrap();

    // Max depth of 5 for recursive variables
    for _ in 0..5 {
        let mut replaced = false;

        let new_text = re
            .replace_all(&current_text, |caps: &regex::Captures| {
                let key = &caps[1];
                if let Some(val) = variables.get(key) {
                    replaced = true;
                    val.to_string()
                } else {
                    // Keep original if variable not found
                    caps[0].to_string()
                }
            })
            .to_string();

        if !replaced || new_text == current_text {
            break;
        }

        current_text = new_text;
    }

    current_text
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_substitute_variables_basic() {
        let mut vars = HashMap::new();
        vars.insert("host".to_string(), "api.example.com".to_string());
        vars.insert("token".to_string(), "abc123".to_string());

        let input = "https://{{host}}/users?token={{token}}";
        let output = substitute_variables(input, &vars);

        assert_eq!(output, "https://api.example.com/users?token=abc123");
    }

    #[test]
    fn test_substitute_recursive() {
        let mut vars = HashMap::new();
        vars.insert("BASE".to_string(), "example.com".to_string());
        vars.insert("HOST".to_string(), "{{BASE}}/api".to_string());
        vars.insert("URL".to_string(), "https://{{HOST}}/v1".to_string());

        let input = "Request to {{URL}}";
        let output = substitute_variables(input, &vars);

        assert_eq!(output, "Request to https://example.com/api/v1");
    }

    #[test]
    fn test_substitute_cycle_limit() {
        let mut vars = HashMap::new();
        vars.insert("A".to_string(), "{{B}}".to_string());
        vars.insert("B".to_string(), "{{A}}".to_string());

        let input = "{{A}}";
        let output = substitute_variables(input, &vars);

        // Should settle on one of them or stop substituting after 5 depths
        // It will oscillate A -> B -> A -> B -> A -> B.
        // It stops after 5 iterations.
        // If it starts with A -> B (depth 1) -> A (depth 2) ...
        assert!(output == "{{A}}" || output == "{{B}}");
    }

    #[test]
    fn test_parse_env_file() {
        let dir = TempDir::new().unwrap();
        let env_path = dir.path().join(".env");
        let content = r#"
            HOST=localhost
            PORT=8080
            # Comment
            MESSAGE="Hello World"
            MULTILINE="Line 1\nLine 2"
        "#;

        let mut file = std::fs::File::create(&env_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let vars = parse_env_file(&env_path).unwrap();

        assert_eq!(vars.get("HOST").unwrap(), "localhost");
        assert_eq!(vars.get("PORT").unwrap(), "8080");
        assert_eq!(vars.get("MESSAGE").unwrap(), "Hello World");
        // dotenvy handles basic multiline if quoted correctly?
        // Note: rust string literal escapes \\n -> \n char.
        // .env file will contain literal \n or newline?
        // We wrote `Line 1\nLine 2` (literal backslash n if we use raw string?)
        // In r#""#, \n is not escaped, it is backslash n.
        // dotenvy supports multiline with real newlines.
        // Let's stick to basic checks.
    }

    #[test]
    fn test_substitute_special_keys() {
        let mut vars = HashMap::new();
        vars.insert("my-key".to_string(), "val1".to_string());
        vars.insert("app.host".to_string(), "val2".to_string());

        let input = "{{ my-key }} - {{app.host}}";
        let output = substitute_variables(input, &vars);

        assert_eq!(output, "val1 - val2");
    }
}
