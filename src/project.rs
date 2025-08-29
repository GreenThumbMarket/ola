use crate::models::{Project, ProjectFile};
use anyhow::{Result, Context as AnyhowContext};
use std::fs;
use std::path::PathBuf;

pub struct ProjectManager {
    base_path: PathBuf,
}

impl ProjectManager {
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME")
            .map_err(|_| anyhow::anyhow!("HOME directory not found"))?;
        
        let base_path = PathBuf::from(home).join(".ola").join("data").join("projects");
        fs::create_dir_all(&base_path)
            .with_context(|| format!("Failed to create project directory: {}", base_path.display()))?;
        
        Ok(Self { base_path })
    }

    pub fn create_project(&self, name: String) -> Result<Project> {
        let project = Project::new(name);
        let project_dir = self.base_path.join(&project.id);
        fs::create_dir_all(&project_dir)
            .with_context(|| format!("Failed to create project directory: {}", project_dir.display()))?;
        
        // Create files subdirectory
        let files_dir = project_dir.join("files");
        fs::create_dir_all(&files_dir)
            .with_context(|| format!("Failed to create files directory: {}", files_dir.display()))?;
        
        self.save_project(&project)?;
        Ok(project)
    }

    pub fn load_project(&self, project_id: &str) -> Result<Option<Project>> {
        let project_file = self.base_path.join(project_id).join("project.json");
        
        if !project_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&project_file)
            .with_context(|| format!("Failed to read project file: {}", project_file.display()))?;
        
        let project: Project = serde_json::from_str(&content)
            .with_context(|| "Failed to parse project JSON")?;
        
        Ok(Some(project))
    }

    pub fn save_project(&self, project: &Project) -> Result<()> {
        let project_dir = self.base_path.join(&project.id);
        let project_file = project_dir.join("project.json");
        
        let content = serde_json::to_string_pretty(project)
            .with_context(|| "Failed to serialize project")?;
        
        fs::write(&project_file, content)
            .with_context(|| format!("Failed to write project file: {}", project_file.display()))?;
        
        Ok(())
    }

    pub fn list_projects(&self) -> Result<Vec<Project>> {
        let mut projects = Vec::new();
        
        if !self.base_path.exists() {
            return Ok(projects);
        }

        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let project_id = entry.file_name().to_string_lossy().to_string();
                if let Ok(Some(project)) = self.load_project(&project_id) {
                    projects.push(project);
                }
            }
        }
        
        // Sort by updated_at descending (most recent first)
        projects.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(projects)
    }

    pub fn get_default_project(&self) -> Result<Project> {
        // Try to load existing default project
        if let Ok(Some(project)) = self.load_project("default") {
            return Ok(project);
        }
        
        // Create new default project
        let mut project = Project::new("Default".to_string());
        project.id = "default".to_string();
        
        let project_dir = self.base_path.join("default");
        fs::create_dir_all(&project_dir)
            .with_context(|| format!("Failed to create default project directory: {}", project_dir.display()))?;
        
        let files_dir = project_dir.join("files");
        fs::create_dir_all(&files_dir)
            .with_context(|| format!("Failed to create files directory: {}", files_dir.display()))?;
        
        self.save_project(&project)?;
        Ok(project)
    }

    pub fn upload_file(&self, project_id: &str, filename: String, content: &[u8]) -> Result<ProjectFile> {
        let files_dir = self.base_path.join(project_id).join("files");
        fs::create_dir_all(&files_dir)
            .with_context(|| format!("Failed to create files directory: {}", files_dir.display()))?;
        
        let file_obj = ProjectFile::new(filename.clone(), content.len() as u64, Self::guess_mime_type(&filename));
        let file_path = files_dir.join(&file_obj.id);
        
        fs::write(&file_path, content)
            .with_context(|| format!("Failed to write file: {}", file_path.display()))?;
        
        Ok(file_obj)
    }

    pub fn download_file(&self, project_id: &str, file_id: &str) -> Result<Option<Vec<u8>>> {
        let file_path = self.base_path.join(project_id).join("files").join(file_id);
        
        if !file_path.exists() {
            return Ok(None);
        }
        
        let content = fs::read(&file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
        
        Ok(Some(content))
    }

    pub fn delete_file(&self, project_id: &str, file_id: &str) -> Result<bool> {
        let file_path = self.base_path.join(project_id).join("files").join(file_id);
        
        if !file_path.exists() {
            return Ok(false);
        }
        
        fs::remove_file(&file_path)
            .with_context(|| format!("Failed to delete file: {}", file_path.display()))?;
        
        Ok(true)
    }

    pub fn read_file_as_text(&self, project_id: &str, file_id: &str) -> Result<Option<String>> {
        if let Some(content) = self.download_file(project_id, file_id)? {
            // Try to convert to UTF-8 string
            match String::from_utf8(content.clone()) {
                Ok(text) => Ok(Some(text)),
                Err(_) => {
                    // If not valid UTF-8, return base64 encoded content
                    use base64::{Engine, engine::general_purpose};
                    let encoded = general_purpose::STANDARD.encode(&content);
                    Ok(Some(format!("[Binary file - base64 encoded: {}]", encoded)))
                }
            }
        } else {
            Ok(None)
        }
    }

    pub fn delete_project(&self, project_id: &str) -> Result<()> {
        let project_dir = self.base_path.join(project_id);
        
        if !project_dir.exists() {
            return Err(anyhow::anyhow!("Project '{}' not found", project_id));
        }
        
        fs::remove_dir_all(&project_dir)
            .with_context(|| format!("Failed to delete project directory: {}", project_dir.display()))?;
        
        Ok(())
    }
    
    pub fn set_active_project(&self, project_id: &str) -> Result<()> {
        // Verify project exists
        if self.load_project(project_id)?.is_none() {
            return Err(anyhow::anyhow!("Project '{}' not found", project_id));
        }
        
        let active_file = self.base_path.parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid base path"))?
            .join("active_project");
        
        fs::write(&active_file, project_id)
            .with_context(|| format!("Failed to write active project file: {}", active_file.display()))?;
        
        Ok(())
    }
    
    pub fn get_active_project(&self) -> Result<Option<String>> {
        let active_file = self.base_path.parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid base path"))?
            .join("active_project");
        
        if !active_file.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&active_file)
            .with_context(|| format!("Failed to read active project file: {}", active_file.display()))?;
        
        let project_id = content.trim();
        
        // Verify the active project still exists
        if self.load_project(project_id)?.is_some() {
            Ok(Some(project_id.to_string()))
        } else {
            // Clean up invalid active project reference
            let _ = fs::remove_file(&active_file);
            Ok(None)
        }
    }
    
    pub fn edit_project(&self, project_id: &str, new_name: Option<String>) -> Result<Project> {
        let mut project = self.load_project(project_id)?
            .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", project_id))?;
        
        if let Some(name) = new_name {
            project.name = name;
            project.updated_at = chrono::Utc::now();
        }
        
        self.save_project(&project)?;
        Ok(project)
    }

    fn guess_mime_type(filename: &str) -> Option<String> {
        let extension = std::path::Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());
        
        match extension.as_deref() {
            Some("rs") => Some("text/rust".to_string()),
            Some("py") => Some("text/python".to_string()),
            Some("js") => Some("text/javascript".to_string()),
            Some("ts") => Some("text/typescript".to_string()),
            Some("json") => Some("application/json".to_string()),
            Some("yaml") | Some("yml") => Some("text/yaml".to_string()),
            Some("toml") => Some("text/toml".to_string()),
            Some("md") => Some("text/markdown".to_string()),
            Some("txt") => Some("text/plain".to_string()),
            Some("html") => Some("text/html".to_string()),
            Some("css") => Some("text/css".to_string()),
            _ => Some("application/octet-stream".to_string()),
        }
    }
}