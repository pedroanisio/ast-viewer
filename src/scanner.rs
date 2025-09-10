use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use std::fs;

pub struct FileScanner {
    ignore_patterns: Vec<String>,
}

impl FileScanner {
    pub fn new() -> Self {
        Self {
            ignore_patterns: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".next".to_string(),
                "__pycache__".to_string(),
                ".pytest_cache".to_string(),
                "venv".to_string(),
                ".env".to_string(),
            ],
        }
    }
    
    pub fn scan_directory(&self, dir: &Path) -> Result<Vec<SourceFile>> {
        let mut files = Vec::new();
        
        for entry in WalkDir::new(dir)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !self.should_ignore(e.path())) 
        {
            let entry = entry?;
            if entry.file_type().is_file() {
                if let Some(source_file) = self.process_file(entry.path())? {
                    files.push(source_file);
                }
            }
        }
        
        Ok(files)
    }
    
    fn should_ignore(&self, path: &Path) -> bool {
        path.components().any(|component| {
            if let Some(name) = component.as_os_str().to_str() {
                self.ignore_patterns.iter().any(|pattern| name == pattern)
            } else {
                false
            }
        })
    }
    
    fn process_file(&self, path: &Path) -> Result<Option<SourceFile>> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        let language = match extension {
            "py" => Some("python"),
            "js" | "mjs" => Some("javascript"),
            "ts" | "mts" => Some("typescript"),
            "jsx" => Some("javascript"),
            "tsx" => Some("tsx"),
            "rs" => Some("rust"),
            _ => {
                // Handle special files without extensions
                let filename = path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("");
                
                match filename {
                    "__init__.py" => Some("python"),  // Handle __init__.py files
                    "Dockerfile" => Some("dockerfile"),
                    "Makefile" => Some("makefile"),
                    "Rakefile" => Some("ruby"),
                    "Gemfile" => Some("ruby"),
                    "Cargo.toml" => Some("toml"),
                    "package.json" => Some("json"),
                    "tsconfig.json" => Some("json"),
                    "pyproject.toml" => Some("toml"),
                    _ => None,
                }
            }
        };
        
        if let Some(language) = language {
            // Try to read as UTF-8, skip if it contains invalid UTF-8
            let content = match fs::read_to_string(path) {
                Ok(content) => content,
                Err(_) => {
                    // Skip files that can't be read as UTF-8 (binary files)
                    return Ok(None);
                }
            };
            
            let relative_path = path.strip_prefix(std::env::current_dir()?).ok()
                .or_else(|| Some(path))
                .unwrap()
                .to_path_buf();
            
            // Special handling for empty files - still include them
            let filename = path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
                
            let final_content = if content.trim().is_empty() {
                match filename {
                    "__init__.py" => "# Empty __init__.py file\n".to_string(),
                    _ => content,
                }
            } else {
                content
            };
            
            Ok(Some(SourceFile {
                path: relative_path,
                content: final_content.clone(),
                language: language.to_string(),
                hash: self.calculate_hash(&final_content),
            }))
        } else {
            Ok(None)
        }
    }
    
    fn calculate_hash(&self, content: &str) -> String {
        blake3::hash(content.as_bytes()).to_hex().to_string()
    }
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: PathBuf,
    pub content: String,
    pub language: String,
    pub hash: String,
}
