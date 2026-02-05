use crate::models::CtfBox;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

/// Get the path to the data file where boxes are stored
fn get_data_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("", "", "ctf-brain")
        .context("Unable to determine data directory")?;
    
    let data_dir = proj_dirs.data_dir();
    
    // Create the directory if it doesn't exist
    fs::create_dir_all(data_dir)
        .context("Failed to create data directory")?;
    
    Ok(data_dir.join("boxes.json"))
}

/// Load boxes from the JSON file
/// Returns empty Vec if file doesn't exist
pub fn load_boxes() -> Result<Vec<CtfBox>> {
    let path = get_data_path()?;
    
    // If file doesn't exist, return empty vector
    if !path.exists() {
        return Ok(Vec::new());
    }
    
    // Read the file
    let content = fs::read_to_string(&path)
        .context("Failed to read boxes.json")?;
    
    // Deserialize from JSON
    let boxes: Vec<CtfBox> = serde_json::from_str(&content)
        .context("Invalid JSON in boxes.json")?;
    
    Ok(boxes)
}

/// Save boxes to the JSON file
pub fn save_boxes(boxes: &[CtfBox]) -> Result<()> {
    let path = get_data_path()?;
    
    // Serialize to pretty JSON for readability
    let json = serde_json::to_string_pretty(boxes)
        .context("Failed to serialize boxes")?;
    
    // Write to file
    fs::write(&path, json)
        .context("Failed to write boxes.json")?;
    
    Ok(())
}
