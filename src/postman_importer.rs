use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct PostmanCollection {
    info: PostmanInfo,
    item: Vec<PostmanItem>,
    #[serde(default)]
    variable: Vec<PostmanVariable>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PostmanInfo {
    name: String,
    #[serde(default)]
    schema: String,
}

#[derive(Debug, Deserialize)]
struct PostmanItem {
    name: String,
    #[serde(default)]
    item: Vec<PostmanItem>,
    #[serde(default)]
    request: Option<PostmanRequest>,
}

#[derive(Debug, Deserialize)]
struct PostmanRequest {
    method: String,
    #[serde(default)]
    header: Vec<PostmanHeader>,
    url: PostmanUrl,
    #[serde(default)]
    body: Option<PostmanBody>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PostmanUrl {
    String(String),
    Object {
        #[serde(default)]
        raw: Option<String>,
        #[serde(default)]
        protocol: Option<String>,
        #[serde(default)]
        host: Vec<String>,
        #[serde(default)]
        path: Vec<String>,
        #[serde(default)]
        query: Vec<PostmanQueryParam>,
    },
}

#[derive(Debug, Deserialize)]
struct PostmanQueryParam {
    key: String,
    value: String,
    #[serde(default)]
    disabled: bool,
}

#[derive(Debug, Deserialize)]
struct PostmanHeader {
    key: String,
    value: String,
    #[serde(default)]
    disabled: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PostmanBody {
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    raw: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PostmanVariable {
    key: String,
    value: Value,
}

/// Reconstructs a URL string from Postman's URL format.
///
/// Postman URLs can be either:
/// - A simple string (returned as-is)
/// - An object with components (protocol, host, path, query)
///
/// When the URL is an object:
/// - If `raw` field is present, it's used directly
/// - Otherwise, the URL is reconstructed from protocol, host, path, and query components
/// - Query parameters marked as disabled are excluded
fn reconstruct_url(url: &PostmanUrl) -> String {
    match url {
        PostmanUrl::String(s) => s.clone(),
        PostmanUrl::Object {
            raw,
            protocol,
            host,
            path,
            query,
        } => {
            if let Some(raw_url) = raw {
                raw_url.clone()
            } else {
                let proto = protocol.as_deref().unwrap_or("https");
                let host_str = host.join(".");
                let path_str = if path.is_empty() {
                    String::new()
                } else {
                    format!("/{}", path.join("/"))
                };
                let query_str = if query.is_empty() {
                    String::new()
                } else {
                    let active_params: Vec<String> = query
                        .iter()
                        .filter(|q| !q.disabled)
                        .map(|q| format!("{}={}", q.key, q.value))
                        .collect();
                    if active_params.is_empty() {
                        String::new()
                    } else {
                        format!("?{}", active_params.join("&"))
                    }
                };
                format!("{}://{}{}{}", proto, host_str, path_str, query_str)
            }
        }
    }
}

/// Recursively processes a Postman collection item (request or folder).
///
/// # Arguments
/// * `item` - The Postman item to process (can be a request or a folder)
/// * `parent_dir` - The parent directory where this item should be created
/// * `depth` - Current nesting depth (used for tracking recursion level)
///
/// # Returns
/// The number of requests processed (0 for empty folders, 1 for requests, sum of children for folders)
///
/// # Behavior
/// - If item contains a request: creates a .http file
/// - If item contains sub-items: creates a folder and recursively processes children
/// - If item is empty: returns 0
fn process_item(
    item: &PostmanItem,
    parent_dir: &Path,
    depth: usize,
) -> Result<usize, String> {
    if let Some(request) = &item.request {
        // This is a request - create .http file
        let file_name = format!("{}.http", item.name.to_lowercase().replace(' ', "-"));
        let file_path = parent_dir.join(&file_name);

        let mut http_content = String::new();

        // Method and URL
        let url_str = reconstruct_url(&request.url);
        http_content.push_str(&format!("{} {}\n", request.method, url_str));

        // Headers
        for header in &request.header {
            if !header.disabled {
                http_content.push_str(&format!("{}: {}\n", header.key, header.value));
            }
        }

        // Body
        if let Some(body) = &request.body {
            if let Some(raw) = &body.raw {
                if !raw.is_empty() {
                    http_content.push('\n');
                    http_content.push_str(raw);
                    http_content.push('\n');
                }
            }
        }

        fs::write(&file_path, http_content)
            .map_err(|e| format!("Failed to write request file: {}", e))?;
        Ok(1)
    } else if !item.item.is_empty() {
        // This is a folder - create directory and recurse
        let folder_name = item.name.to_lowercase().replace(' ', "-");
        let folder_path = parent_dir.join(&folder_name);
        fs::create_dir_all(&folder_path)
            .map_err(|e| format!("Failed to create folder: {}", e))?;

        let mut count = 0;
        for child in &item.item {
            count += process_item(child, &folder_path, depth + 1)?;
        }
        Ok(count)
    } else {
        // Empty item
        Ok(0)
    }
}

/// Imports a Postman Collection v2.1 file into Mercury's .http file format.
///
/// # Arguments
/// * `json_path` - Path to the Postman collection JSON file
/// * `output_dir` - Directory where imported files will be created
///
/// # Returns
/// A tuple of (request_count, environment_count) on success, or an error message on failure
///
/// # Behavior
/// - Parses the Postman collection JSON file
/// - Creates .http files for each request, preserving folder structure
/// - Extracts collection variables to a .env file (if any exist)
/// - Handles nested folders with unlimited depth
/// - Reconstructs URLs from Postman's object format
///
/// # Errors
/// Returns an error if:
/// - The file cannot be read
/// - The JSON is invalid or not a valid Postman collection
/// - File system operations fail (creating directories, writing files)
pub fn import_postman_collection(
    json_path: &Path,
    output_dir: &Path,
) -> Result<(usize, usize), String> {
    let content =
        fs::read_to_string(json_path).map_err(|e| format!("Failed to read file: {}", e))?;

    let collection: PostmanCollection = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse Postman collection: {}", e))?;

    // Extract collection variables to .env file
    let mut env_count = 0;
    if !collection.variable.is_empty() {
        let collection_name = collection.info.name.to_lowercase().replace(' ', "-");
        let env_path = output_dir.join(format!(".env.{}", collection_name));

        let mut env_content = String::new();
        for var in &collection.variable {
            let value_str = match &var.value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => var.value.to_string(),
            };
            env_content.push_str(&format!("{}={}\n", var.key, value_str));
        }

        fs::write(&env_path, env_content)
            .map_err(|e| format!("Failed to write environment file: {}", e))?;
        env_count = 1;
    }

    // Process all items (requests and folders)
    let mut request_count = 0;
    for item in &collection.item {
        request_count += process_item(item, output_dir, 0)?;
    }

    Ok((request_count, env_count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_temp_file(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_import_simple_collection() {
        let dir = TempDir::new().unwrap();
        let json_content = r#"{
            "info": {
                "name": "Test Collection",
                "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
            },
            "item": [
                {
                    "name": "Test Request",
                    "request": {
                        "method": "GET",
                        "header": [],
                        "url": {
                            "raw": "https://example.com/api",
                            "protocol": "https",
                            "host": ["example", "com"],
                            "path": ["api"]
                        }
                    }
                }
            ],
            "variable": []
        }"#;
        let file_path = create_temp_file(dir.path(), "collection.json", json_content);
        let output_dir = dir.path().join("output");
        fs::create_dir(&output_dir).unwrap();

        let result = import_postman_collection(&file_path, &output_dir);
        assert!(result.is_ok());
        let (req_count, env_count) = result.unwrap();
        assert_eq!(req_count, 1);
        assert_eq!(env_count, 0);

        // Check that file was created
        let http_file = output_dir.join("test-request.http");
        assert!(http_file.exists());
        let content = fs::read_to_string(http_file).unwrap();
        assert!(content.contains("GET https://example.com/api"));
    }

    #[test]
    fn test_import_with_folders() {
        let dir = TempDir::new().unwrap();
        let json_content = r#"{
            "info": {
                "name": "Test Collection",
                "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
            },
            "item": [
                {
                    "name": "Users",
                    "item": [
                        {
                            "name": "Get User",
                            "request": {
                                "method": "GET",
                                "header": [],
                                "url": "https://api.example.com/users/1"
                            }
                        }
                    ]
                }
            ],
            "variable": []
        }"#;
        let file_path = create_temp_file(dir.path(), "collection.json", json_content);
        let output_dir = dir.path().join("output");
        fs::create_dir(&output_dir).unwrap();

        let result = import_postman_collection(&file_path, &output_dir);
        assert!(result.is_ok());
        let (req_count, _) = result.unwrap();
        assert_eq!(req_count, 1);

        // Check that folder and file were created
        let folder = output_dir.join("users");
        assert!(folder.exists());
        let http_file = folder.join("get-user.http");
        assert!(http_file.exists());
    }

    #[test]
    fn test_import_with_variables() {
        let dir = TempDir::new().unwrap();
        let json_content = r#"{
            "info": {
                "name": "My API",
                "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
            },
            "item": [],
            "variable": [
                {
                    "key": "host",
                    "value": "api.example.com"
                },
                {
                    "key": "token",
                    "value": "abc123"
                }
            ]
        }"#;
        let file_path = create_temp_file(dir.path(), "collection.json", json_content);
        let output_dir = dir.path().join("output");
        fs::create_dir(&output_dir).unwrap();

        let result = import_postman_collection(&file_path, &output_dir);
        assert!(result.is_ok());
        let (req_count, env_count) = result.unwrap();
        assert_eq!(req_count, 0);
        assert_eq!(env_count, 1);

        // Check that .env file was created
        let env_file = output_dir.join(".env.my-api");
        assert!(env_file.exists());
        let content = fs::read_to_string(env_file).unwrap();
        assert!(content.contains("host=api.example.com"));
        assert!(content.contains("token=abc123"));
    }

    #[test]
    fn test_import_nested_folders() {
        let dir = TempDir::new().unwrap();
        let json_content = r#"{
            "info": {
                "name": "Test",
                "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
            },
            "item": [
                {
                    "name": "API",
                    "item": [
                        {
                            "name": "V1",
                            "item": [
                                {
                                    "name": "Users",
                                    "item": [
                                        {
                                            "name": "List Users",
                                            "request": {
                                                "method": "GET",
                                                "header": [],
                                                "url": "https://api.example.com/v1/users"
                                            }
                                        }
                                    ]
                                }
                            ]
                        }
                    ]
                }
            ],
            "variable": []
        }"#;
        let file_path = create_temp_file(dir.path(), "collection.json", json_content);
        let output_dir = dir.path().join("output");
        fs::create_dir(&output_dir).unwrap();

        let result = import_postman_collection(&file_path, &output_dir);
        assert!(result.is_ok());
        let (req_count, _) = result.unwrap();
        assert_eq!(req_count, 1);

        // Check nested folder structure
        let http_file = output_dir.join("api/v1/users/list-users.http");
        assert!(http_file.exists());
    }

    #[test]
    fn test_import_with_headers_and_body() {
        let dir = TempDir::new().unwrap();
        let json_content = r#"{
            "info": {
                "name": "Test",
                "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
            },
            "item": [
                {
                    "name": "Create User",
                    "request": {
                        "method": "POST",
                        "header": [
                            {
                                "key": "Content-Type",
                                "value": "application/json"
                            },
                            {
                                "key": "Authorization",
                                "value": "Bearer {{token}}"
                            }
                        ],
                        "url": "https://api.example.com/users",
                        "body": {
                            "mode": "raw",
                            "raw": "{\"name\": \"John Doe\"}"
                        }
                    }
                }
            ],
            "variable": []
        }"#;
        let file_path = create_temp_file(dir.path(), "collection.json", json_content);
        let output_dir = dir.path().join("output");
        fs::create_dir(&output_dir).unwrap();

        let result = import_postman_collection(&file_path, &output_dir);
        assert!(result.is_ok());

        let http_file = output_dir.join("create-user.http");
        assert!(http_file.exists());
        let content = fs::read_to_string(http_file).unwrap();
        assert!(content.contains("POST https://api.example.com/users"));
        assert!(content.contains("Content-Type: application/json"));
        assert!(content.contains("Authorization: Bearer {{token}}"));
        assert!(content.contains("{\"name\": \"John Doe\"}"));
    }

    #[test]
    fn test_import_comprehensive_collection() {
        let dir = TempDir::new().unwrap();
        let json_content = r#"{
            "info": {
                "name": "Comprehensive API",
                "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
            },
            "item": [
                {
                    "name": "Auth",
                    "item": [
                        {
                            "name": "Login",
                            "request": {
                                "method": "POST",
                                "header": [
                                    {"key": "Content-Type", "value": "application/json"}
                                ],
                                "url": "{{host}}/auth/login",
                                "body": {
                                    "mode": "raw",
                                    "raw": "{\"user\":\"test\"}"
                                }
                            }
                        }
                    ]
                },
                {
                    "name": "Users",
                    "item": [
                        {
                            "name": "V1",
                            "item": [
                                {
                                    "name": "List",
                                    "request": {
                                        "method": "GET",
                                        "header": [],
                                        "url": {
                                            "raw": "{{host}}/v1/users?page=1",
                                            "protocol": "https",
                                            "host": ["{{host}}"],
                                            "path": ["v1", "users"],
                                            "query": [
                                                {"key": "page", "value": "1"}
                                            ]
                                        }
                                    }
                                }
                            ]
                        }
                    ]
                },
                {
                    "name": "Health",
                    "request": {
                        "method": "GET",
                        "header": [],
                        "url": "{{host}}/health"
                    }
                }
            ],
            "variable": [
                {"key": "host", "value": "api.test.com"},
                {"key": "token", "value": "secret123"}
            ]
        }"#;
        let file_path = create_temp_file(dir.path(), "comprehensive.json", json_content);
        let output_dir = dir.path().join("output");
        fs::create_dir(&output_dir).unwrap();

        let result = import_postman_collection(&file_path, &output_dir);
        assert!(result.is_ok());
        let (req_count, env_count) = result.unwrap();
        assert_eq!(req_count, 3); // Login, List, Health
        assert_eq!(env_count, 1);

        // Verify folder structure
        assert!(output_dir.join("auth/login.http").exists());
        assert!(output_dir.join("users/v1/list.http").exists());
        assert!(output_dir.join("health.http").exists());
        assert!(output_dir.join(".env.comprehensive-api").exists());

        // Verify URL with query params
        let list_content = fs::read_to_string(output_dir.join("users/v1/list.http")).unwrap();
        assert!(list_content.contains("?page=1"));
    }

    #[test]
    fn test_import_invalid_json() {
        let dir = TempDir::new().unwrap();
        let invalid_content = "NOT JSON";
        let file_path = create_temp_file(dir.path(), "invalid.json", invalid_content);
        let output_dir = dir.path().join("output");
        fs::create_dir(&output_dir).unwrap();

        let result = import_postman_collection(&file_path, &output_dir);
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("Failed to parse Postman collection"));
    }

    #[test]
    fn test_url_reconstruction() {
        // Test with raw URL
        let url1 = PostmanUrl::Object {
            raw: Some("https://example.com/api/v1".to_string()),
            protocol: None,
            host: vec![],
            path: vec![],
            query: vec![],
        };
        assert_eq!(reconstruct_url(&url1), "https://example.com/api/v1");

        // Test with components
        let url2 = PostmanUrl::Object {
            raw: None,
            protocol: Some("https".to_string()),
            host: vec!["api".to_string(), "example".to_string(), "com".to_string()],
            path: vec!["v1".to_string(), "users".to_string()],
            query: vec![],
        };
        assert_eq!(reconstruct_url(&url2), "https://api.example.com/v1/users");

        // Test with query params
        let url3 = PostmanUrl::Object {
            raw: None,
            protocol: Some("https".to_string()),
            host: vec!["api".to_string(), "example".to_string(), "com".to_string()],
            path: vec!["search".to_string()],
            query: vec![
                PostmanQueryParam {
                    key: "q".to_string(),
                    value: "test".to_string(),
                    disabled: false,
                },
                PostmanQueryParam {
                    key: "limit".to_string(),
                    value: "10".to_string(),
                    disabled: false,
                },
            ],
        };
        assert_eq!(reconstruct_url(&url3), "https://api.example.com/search?q=test&limit=10");
    }
}
