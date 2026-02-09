use crate::models::CtfBox;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

// Get the base directory for ctf-brain data
fn get_base_dir() -> Result<PathBuf> {
    let base = dirs::home_dir()
        .context("Could not find home directory")?
        .join(".ctf-brain");

    fs::create_dir_all(&base)
        .context("Failed to create .ctf-brain directory")?;
    
    Ok(base)
}

