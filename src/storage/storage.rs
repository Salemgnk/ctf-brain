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

/// Import actions from shell logs for a specific box
pub fn import_shell_logs(box_id: i32) -> Result<Vec<crate::models::Action>> {
    let base_dir = dirs::home_dir()
        .context("Unable to determine home directory")?
        .join(".ctf-brain/logs");
    
    let log_file = base_dir.join(format!("box-{}.jsonl", box_id));
    
    if !log_file.exists() {
        return Ok(Vec::new());
    }
    
    let content = fs::read_to_string(&log_file)
        .context("Failed to read log file")?;
    
    let mut actions = Vec::new();
    
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        
        if let Ok(entry) = serde_json::from_str::<LogEntry>(line) {
            // Skip auto-logged commands without output (they're duplicates)
            if entry.auto.unwrap_or(false) && entry.output.is_none() {
                continue;
            }
            
            let result = match entry.result.as_deref() {
                Some("success") => crate::models::ActionResult::Success,
                Some("fail") => crate::models::ActionResult::Fail,
                _ => crate::models::ActionResult::Unknown,
            };
            
            let timestamp = chrono::DateTime::parse_from_rfc3339(&entry.time)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
            
            actions.push(crate::models::Action {
                timestamp,
                command: entry.cmd,
                result,
                note: None,
                output: entry.output,
            });
        }
    }
    
    Ok(actions)
}

/// Clear shell logs for a specific box (after import)
pub fn clear_shell_logs(box_id: i32) -> Result<()> {
    let base_dir = dirs::home_dir()
        .context("Unable to determine home directory")?
        .join(".ctf-brain/logs");
    
    let log_file = base_dir.join(format!("box-{}.jsonl", box_id));
    
    if log_file.exists() {
        fs::remove_file(&log_file).context("Failed to remove log file")?;
    }
    
    Ok(())
}

#[derive(serde::Deserialize)]
struct LogEntry {
    time: String,
    #[allow(dead_code)]
    box_id: i32,
    cmd: String,
    result: Option<String>,
    output: Option<String>,
    auto: Option<bool>,
}

/// Generate a write-up markdown for a box
pub fn generate_writeup(ctf_box: &CtfBox) -> String {
    let mut md = String::new();
    
    // Header
    md.push_str(&format!("# {} - Write-up\n\n", ctf_box.title));
    md.push_str(&format!("**Platform:** {}  \n", ctf_box.platform));
    md.push_str(&format!("**IP:** {}  \n", ctf_box.ip_address));
    md.push_str(&format!("**Tags:** {}  \n", ctf_box.tags.join(", ")));
    md.push_str(&format!("**Date:** {}  \n\n", ctf_box.created_date.format("%Y-%m-%d")));
    md.push_str("---\n\n");
    
    // Table of contents
    md.push_str("## Table of Contents\n\n");
    md.push_str("1. [Reconnaissance](#reconnaissance)\n");
    md.push_str("2. [Enumeration](#enumeration)\n");
    md.push_str("3. [Exploitation](#exploitation)\n");
    md.push_str("4. [Privilege Escalation](#privilege-escalation)\n");
    md.push_str("5. [Flags](#flags)\n\n");
    md.push_str("---\n\n");
    
    // Collect notes by category
    let recon_notes: Vec<_> = ctf_box.notes.iter()
        .filter(|n| matches!(n.category, crate::models::NoteCategory::Recon))
        .collect();
    let foothold_notes: Vec<_> = ctf_box.notes.iter()
        .filter(|n| matches!(n.category, crate::models::NoteCategory::Foothold))
        .collect();
    let privesc_notes: Vec<_> = ctf_box.notes.iter()
        .filter(|n| matches!(n.category, crate::models::NoteCategory::Privesc))
        .collect();
    let web_notes: Vec<_> = ctf_box.notes.iter()
        .filter(|n| matches!(n.category, crate::models::NoteCategory::Web))
        .collect();
    
    // Reconnaissance section
    md.push_str("## Reconnaissance\n\n");
    
    // Add recon notes
    for note in &recon_notes {
        md.push_str(&format!("- {}\n", note.content));
    }
    if !recon_notes.is_empty() {
        md.push_str("\n");
    }
    
    // Add nmap-like commands
    let recon_commands: Vec<_> = ctf_box.actions.iter()
        .filter(|a| a.command.contains("nmap") || a.command.contains("ping") || a.command.contains("whois"))
        .collect();
    
    for action in recon_commands {
        md.push_str(&format!("### {}\n\n", action.command.split_whitespace().next().unwrap_or("Command")));
        md.push_str(&format!("```bash\n$ {}\n", action.command));
        if let Some(output) = &action.output {
            md.push_str(output);
            if !output.ends_with('\n') {
                md.push_str("\n");
            }
        }
        md.push_str("```\n\n");
        
        if let Some(note) = &action.note {
            md.push_str(&format!("> **Note:** {}\n\n", note));
        }
    }
    
    // Enumeration section
    md.push_str("## Enumeration\n\n");
    
    // Add web notes
    for note in &web_notes {
        md.push_str(&format!("- {}\n", note.content));
    }
    if !web_notes.is_empty() {
        md.push_str("\n");
    }
    
    // Add enumeration commands
    let enum_commands: Vec<_> = ctf_box.actions.iter()
        .filter(|a| {
            a.command.contains("gobuster") || 
            a.command.contains("ffuf") || 
            a.command.contains("nikto") ||
            a.command.contains("dirb") ||
            a.command.contains("enum4linux") ||
            a.command.contains("smbclient")
        })
        .collect();
    
    for action in enum_commands {
        md.push_str(&format!("### {}\n\n", action.command.split_whitespace().next().unwrap_or("Command")));
        md.push_str(&format!("```bash\n$ {}\n", action.command));
        if let Some(output) = &action.output {
            // Truncate very long outputs
            let truncated = if output.len() > 3000 {
                format!("{}...\n[Output truncated]", &output[..3000])
            } else {
                output.clone()
            };
            md.push_str(&truncated);
            if !truncated.ends_with('\n') {
                md.push_str("\n");
            }
        }
        md.push_str("```\n\n");
    }
    
    // Exploitation section
    md.push_str("## Exploitation\n\n");
    
    // Add foothold notes
    for note in &foothold_notes {
        md.push_str(&format!("- {}\n", note.content));
    }
    if !foothold_notes.is_empty() {
        md.push_str("\n");
    }
    
    // Add exploit commands
    let exploit_commands: Vec<_> = ctf_box.actions.iter()
        .filter(|a| {
            a.command.contains("exploit") || 
            a.command.contains("msfconsole") || 
            a.command.contains("searchsploit") ||
            a.command.contains("sqlmap") ||
            a.command.contains("hydra") ||
            a.command.contains("nc ") ||
            a.command.contains("reverse")
        })
        .collect();
    
    for action in exploit_commands {
        md.push_str(&format!("```bash\n$ {}\n", action.command));
        if let Some(output) = &action.output {
            let truncated = if output.len() > 2000 {
                format!("{}...\n[Output truncated]", &output[..2000])
            } else {
                output.clone()
            };
            md.push_str(&truncated);
            if !truncated.ends_with('\n') {
                md.push_str("\n");
            }
        }
        md.push_str("```\n\n");
    }
    
    // Privilege Escalation section
    md.push_str("## Privilege Escalation\n\n");
    
    // Add privesc notes
    for note in &privesc_notes {
        md.push_str(&format!("- {}\n", note.content));
    }
    if !privesc_notes.is_empty() {
        md.push_str("\n");
    }
    
    // Add privesc commands
    let privesc_commands: Vec<_> = ctf_box.actions.iter()
        .filter(|a| {
            a.command.contains("sudo") || 
            a.command.contains("linpeas") || 
            a.command.contains("linenum") ||
            a.command.contains("SUID") ||
            a.command.contains("getcap") ||
            a.command.contains("find / ")
        })
        .collect();
    
    for action in privesc_commands {
        md.push_str(&format!("```bash\n$ {}\n", action.command));
        if let Some(output) = &action.output {
            let truncated = if output.len() > 2000 {
                format!("{}...\n[Output truncated]", &output[..2000])
            } else {
                output.clone()
            };
            md.push_str(&truncated);
            if !truncated.ends_with('\n') {
                md.push_str("\n");
            }
        }
        md.push_str("```\n\n");
    }
    
    // Flags section
    md.push_str("## Flags\n\n");
    md.push_str("### User Flag\n\n");
    md.push_str("```\n[USER FLAG HERE]\n```\n\n");
    md.push_str("### Root Flag\n\n");
    md.push_str("```\n[ROOT FLAG HERE]\n```\n\n");
    
    // Miscellaneous notes
    let misc_notes: Vec<_> = ctf_box.notes.iter()
        .filter(|n| matches!(n.category, crate::models::NoteCategory::Misc | crate::models::NoteCategory::Crypto | crate::models::NoteCategory::Pwn | crate::models::NoteCategory::Stego | crate::models::NoteCategory::Reversing))
        .collect();
    
    if !misc_notes.is_empty() {
        md.push_str("---\n\n## Additional Notes\n\n");
        for note in misc_notes {
            md.push_str(&format!("- **{:?}:** {}\n", note.category, note.content));
        }
    }
    
    // All other commands that weren't categorized
    let other_commands: Vec<_> = ctf_box.actions.iter()
        .filter(|a| {
            !a.command.contains("nmap") &&
            !a.command.contains("ping") &&
            !a.command.contains("gobuster") &&
            !a.command.contains("ffuf") &&
            !a.command.contains("nikto") &&
            !a.command.contains("exploit") &&
            !a.command.contains("msfconsole") &&
            !a.command.contains("searchsploit") &&
            !a.command.contains("sqlmap") &&
            !a.command.contains("hydra") &&
            !a.command.contains("sudo") &&
            !a.command.contains("linpeas") &&
            a.output.is_some()
        })
        .collect();
    
    if !other_commands.is_empty() {
        md.push_str("\n---\n\n## Command Log\n\n");
        for action in other_commands {
            let result_icon = match action.result {
                crate::models::ActionResult::Success => "✓",
                crate::models::ActionResult::Fail => "✗",
                crate::models::ActionResult::Unknown => "?",
            };
            md.push_str(&format!("### {} `{}`\n\n", result_icon, action.command));
            if let Some(output) = &action.output {
                md.push_str("```\n");
                let truncated = if output.len() > 1500 {
                    format!("{}...\n[Output truncated]", &output[..1500])
                } else {
                    output.clone()
                };
                md.push_str(&truncated);
                if !truncated.ends_with('\n') {
                    md.push_str("\n");
                }
                md.push_str("```\n\n");
            }
        }
    }
    
    md.push_str("\n---\n\n*Generated by CTF Brain*\n");
    
    md
}

/// Save write-up to file
pub fn save_writeup(ctf_box: &CtfBox) -> Result<PathBuf> {
    let base_dir = dirs::home_dir()
        .context("Unable to determine home directory")?
        .join(".ctf-brain/writeups");
    
    fs::create_dir_all(&base_dir).context("Failed to create writeups directory")?;
    
    let filename = format!("{}-{}.md", 
        ctf_box.title.to_lowercase().replace(' ', "-"),
        ctf_box.created_date.format("%Y%m%d")
    );
    let path = base_dir.join(&filename);
    
    let content = generate_writeup(ctf_box);
    fs::write(&path, &content).context("Failed to write writeup file")?;
    
    Ok(path)
}
