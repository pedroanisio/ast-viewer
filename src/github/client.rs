use anyhow::{Result, Context};
use git2::{Repository, Cred, RemoteCallbacks, FetchOptions};
use std::path::Path;
use octocrab::Octocrab;

pub struct GitHubClient {
    token: Option<String>,
    #[allow(dead_code)]
    client: Option<Octocrab>,
}

impl GitHubClient {
    pub fn new(token: Option<String>) -> Result<Self> {
        let client = if let Some(ref token) = token {
            Some(Octocrab::builder()
                .personal_token(token.clone())
                .build()?)
        } else {
            None
        };
        
        Ok(Self { token, client })
    }
    
    pub async fn clone_repository(&self, repo_url: &str, target_dir: &Path) -> Result<Repository> {
        // Parse repo URL
        let repo_url = self.normalize_url(repo_url)?;
        
        // Handle existing directory - remove it if it exists
        if target_dir.exists() {
            std::fs::remove_dir_all(target_dir)
                .context("Failed to remove existing directory")?;
        }
        
        // Ensure parent directory exists
        if let Some(parent) = target_dir.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create parent directory")?;
        }
        
        // Setup callbacks for authentication if token is provided
        let mut callbacks = RemoteCallbacks::new();
        if let Some(ref token) = self.token {
            callbacks.credentials(move |_url, _username, _allowed| {
                Cred::userpass_plaintext("x-access-token", token)
            });
        }
        
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);
        
        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_options);
        
        // Clone the repository
        let repo = builder.clone(&repo_url, target_dir)
            .context(format!("Failed to clone repository: {}", repo_url))?;
        
        Ok(repo)
    }
    
    pub fn get_current_commit(&self, repo: &Repository) -> Result<String> {
        let head = repo.head()?;
        let commit = head.peel_to_commit()?;
        Ok(commit.id().to_string())
    }
    
    pub fn get_repo_name(&self, repo_url: &str) -> String {
        repo_url
            .trim_end_matches(".git")
            .split('/')
            .last()
            .unwrap_or("unknown")
            .to_string()
    }
    
    fn normalize_url(&self, url: &str) -> Result<String> {
        if url.starts_with("http://") || url.starts_with("https://") {
            Ok(url.to_string())
        } else if url.starts_with("git@github.com:") {
            // Convert SSH URL to HTTPS
            Ok(url.replace("git@github.com:", "https://github.com/"))
        } else if url.contains("/") && !url.contains("://") {
            // Assume it's a GitHub shorthand (e.g., "user/repo")
            Ok(format!("https://github.com/{}.git", url))
        } else {
            Ok(url.to_string())
        }
    }
}
