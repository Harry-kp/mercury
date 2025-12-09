use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct InsomniaExport {
    resources: Vec<InsomniaResource>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "_type")]
enum InsomniaResource {
    #[serde(rename = "request")]
    Request(InsomniaRequest),
    #[serde(rename = "request_group")]
    RequestGroup(InsomniaRequestGroup),
    #[serde(rename = "environment")]
    Environment(InsomniaEnvironment),
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
struct InsomniaRequest {
    name: String,
    method: String,
    url: String,
    #[serde(default)]
    headers: Vec<InsomniaHeader>,
    #[serde(default)]
    body: Option<InsomniaBody>,
    #[serde(rename = "parentId")]
    parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct InsomniaRequestGroup {
    #[serde(rename = "_id")]
    id: String,
    name: String,
    #[serde(rename = "parentId")]
    parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct InsomniaEnvironment {
    name: String,
    data: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
struct InsomniaHeader {
    name: String,
    value: String,
    #[serde(default)]
    disabled: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct InsomniaBody {
    #[serde(rename = "mimeType")]
    mime_type: Option<String>,
    text: Option<String>,
}

pub fn import_insomnia_collection(json_path: &Path, output_dir: &Path) -> Result<(usize, usize), String> {
    let content = fs::read_to_string(json_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let export: InsomniaExport = match serde_json::from_str(&content) {
        Ok(json) => json,
        Err(_) => {
            // Try parsing as YAML if JSON fails
            serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse as JSON or YAML: {}", e))?
        }
    };
    
    // Extract request groups (folders)
    let mut groups: HashMap<String, String> = HashMap::new();
    for resource in &export.resources {
        if let InsomniaResource::RequestGroup(group) = resource {
            groups.insert(group.id.clone(), group.name.clone());
        }
    }
    
    // Extract environments
    let mut env_count = 0;
    for resource in &export.resources {
        if let InsomniaResource::Environment(env) = resource {
            if !env.data.is_empty() {
                let env_name = env.name.to_lowercase().replace(' ', "-");
                let env_path = output_dir.join(format!(".env.{}", env_name));
                
                let mut env_content = String::new();
                for (key, value) in &env.data {
                    let value_str = match value {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        _ => value.to_string(),
                    };
                    env_content.push_str(&format!("{}={}\n", key, value_str));
                }
                
                fs::write(&env_path, env_content)
                    .map_err(|e| format!("Failed to write environment file: {}", e))?;
                env_count += 1;
            }
        }
    }
    
    // Convert requests to .http files
    let mut request_count = 0;
    for resource in &export.resources {
        if let InsomniaResource::Request(request) = resource {
            let folder_name = request.parent_id
                .as_ref()
                .and_then(|id| groups.get(id))
                .map(|name| name.to_lowercase().replace(' ', "-"))
                .unwrap_or_else(|| "imported".to_string());
            
            let folder_path = output_dir.join(&folder_name);
            fs::create_dir_all(&folder_path)
                .map_err(|e| format!("Failed to create folder: {}", e))?;
            
            let file_name = format!("{}.http", request.name.to_lowercase().replace(' ', "-"));
            let file_path = folder_path.join(&file_name);
            
            let mut http_content = String::new();
            
            // Method and URL
            http_content.push_str(&format!("{} {}\n", request.method, request.url));
            
            // Headers
            for header in &request.headers {
                if !header.disabled {
                    http_content.push_str(&format!("{}: {}\n", header.name, header.value));
                }
            }
            
            // Body
            if let Some(body) = &request.body {
                if let Some(text) = &body.text {
                    if !text.is_empty() {
                        http_content.push_str("\n");
                        http_content.push_str(text);
                        http_content.push_str("\n");
                    }
                }
            }
            
            fs::write(&file_path, http_content)
                .map_err(|e| format!("Failed to write request file: {}", e))?;
            request_count += 1;
        }
    }
    
    Ok((request_count, env_count))
}
