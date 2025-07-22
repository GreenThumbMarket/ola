use ola::{Project, ProjectManager, Goal, Context, ProjectFile};
use tempfile::TempDir;

#[test]
fn test_project_creation() {
    let project = Project::new("Test Project".to_string());
    
    assert_eq!(project.name, "Test Project");
    assert!(!project.id.is_empty());
    assert_eq!(project.files.len(), 0);
    assert_eq!(project.goals.len(), 0);
    assert_eq!(project.contexts.len(), 0);
}

#[test]
fn test_project_add_goal() {
    let mut project = Project::new("Test Project".to_string());
    let goal = Goal::new("Test Goal".to_string(), 0);
    
    project.add_goal(goal.clone());
    
    assert_eq!(project.goals.len(), 1);
    assert_eq!(project.goals[0].text, "Test Goal");
    assert_eq!(project.goals[0].order, 0);
}

#[test]
fn test_project_add_context() {
    let mut project = Project::new("Test Project".to_string());
    let context = Context::new("Test Context".to_string(), 0);
    
    project.add_context(context.clone());
    
    assert_eq!(project.contexts.len(), 1);
    assert_eq!(project.contexts[0].text, "Test Context");
    assert_eq!(project.contexts[0].order, 0);
}

#[test]
fn test_project_add_file() {
    let mut project = Project::new("Test Project".to_string());
    let file = ProjectFile::new("test.rs".to_string(), 100, Some("text/rust".to_string()));
    
    project.add_file(file.clone());
    
    assert_eq!(project.files.len(), 1);
    assert_eq!(project.files[0].filename, "test.rs");
    assert_eq!(project.files[0].size, 100);
}

#[test]
fn test_project_manager_create_project() -> Result<(), Box<dyn std::error::Error>> {
    // Use temporary directory for testing
    let temp_dir = TempDir::new()?;
    std::env::set_var("HOME", temp_dir.path());
    
    let project_manager = ProjectManager::new()?;
    let project = project_manager.create_project("Test Project".to_string())?;
    
    assert_eq!(project.name, "Test Project");
    assert!(!project.id.is_empty());
    
    // Verify project directory was created
    let project_dir = temp_dir.path().join(".ola").join("data").join("projects").join(&project.id);
    assert!(project_dir.exists());
    
    // Verify files directory was created
    let files_dir = project_dir.join("files");
    assert!(files_dir.exists());
    
    // Verify project.json was created
    let project_file = project_dir.join("project.json");
    assert!(project_file.exists());
    
    Ok(())
}

#[test]
fn test_project_manager_save_and_load() -> Result<(), Box<dyn std::error::Error>> {
    // Use temporary directory for testing
    let temp_dir = TempDir::new()?;
    std::env::set_var("HOME", temp_dir.path());
    
    let project_manager = ProjectManager::new()?;
    let mut project = project_manager.create_project("Test Project".to_string())?;
    
    // Add some data to the project
    let goal = Goal::new("Test Goal".to_string(), 0);
    let context = Context::new("Test Context".to_string(), 0);
    project.add_goal(goal);
    project.add_context(context);
    
    // Save the project
    project_manager.save_project(&project)?;
    
    // Load the project
    let loaded_project = project_manager.load_project(&project.id)?;
    assert!(loaded_project.is_some());
    
    let loaded_project = loaded_project.unwrap();
    assert_eq!(loaded_project.name, "Test Project");
    assert_eq!(loaded_project.goals.len(), 1);
    assert_eq!(loaded_project.contexts.len(), 1);
    assert_eq!(loaded_project.goals[0].text, "Test Goal");
    assert_eq!(loaded_project.contexts[0].text, "Test Context");
    
    Ok(())
}

#[test]
fn test_project_manager_upload_file() -> Result<(), Box<dyn std::error::Error>> {
    // Use temporary directory for testing
    let temp_dir = TempDir::new()?;
    std::env::set_var("HOME", temp_dir.path());
    
    let project_manager = ProjectManager::new()?;
    let project = project_manager.create_project("Test Project".to_string())?;
    
    // Upload a test file
    let file_content = b"fn main() { println!(\"Hello, world!\"); }";
    let file_obj = project_manager.upload_file(&project.id, "test.rs".to_string(), file_content)?;
    
    assert_eq!(file_obj.filename, "test.rs");
    assert_eq!(file_obj.size, file_content.len() as u64);
    
    // Verify file was saved
    let downloaded = project_manager.download_file(&project.id, &file_obj.id)?;
    assert!(downloaded.is_some());
    assert_eq!(downloaded.unwrap(), file_content);
    
    // Verify file can be read as text
    let text = project_manager.read_file_as_text(&project.id, &file_obj.id)?;
    assert!(text.is_some());
    assert_eq!(text.unwrap(), String::from_utf8(file_content.to_vec())?);
    
    Ok(())
}

#[test]
fn test_project_manager_list_projects() -> Result<(), Box<dyn std::error::Error>> {
    // Use temporary directory for testing
    let temp_dir = TempDir::new()?;
    std::env::set_var("HOME", temp_dir.path());
    
    let project_manager = ProjectManager::new()?;
    
    // Initially no projects
    let projects = project_manager.list_projects()?;
    assert_eq!(projects.len(), 0);
    
    // Create a project
    let _project1 = project_manager.create_project("Project 1".to_string())?;
    let _project2 = project_manager.create_project("Project 2".to_string())?;
    
    // List projects
    let projects = project_manager.list_projects()?;
    assert_eq!(projects.len(), 2);
    
    // Projects should be sorted by updated_at (most recent first)
    assert_eq!(projects[0].name, "Project 2");
    assert_eq!(projects[1].name, "Project 1");
    
    Ok(())
}