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

/// Save boxes to the JSON file (with automatic backup)
pub fn save_boxes(boxes: &[CtfBox]) -> Result<()> {
    let path = get_data_path()?;
    
    // Create a backup before overwriting
    if path.exists() {
        backup_data(&path)?;
    }
    
    // Serialize to pretty JSON for readability
    let json = serde_json::to_string_pretty(boxes)
        .context("Failed to serialize boxes")?;
    
    // Write to file
    fs::write(&path, json)
        .context("Failed to write boxes.json")?;
    
    Ok(())
}

/// Create a rotating backup of the data file.
/// Keeps up to 5 backups: boxes.json.bak.1 (newest) to boxes.json.bak.5 (oldest)
fn backup_data(path: &PathBuf) -> Result<()> {
    let max_backups = 5;
    
    // Rotate existing backups: .bak.5 is deleted, .bak.4 → .bak.5, etc.
    for i in (1..max_backups).rev() {
        let older = path.with_extension(format!("json.bak.{}", i + 1));
        let newer = path.with_extension(format!("json.bak.{}", i));
        if newer.exists() {
            let _ = fs::rename(&newer, &older);
        }
    }
    
    // Copy current file to .bak.1
    let backup_path = path.with_extension("json.bak.1");
    fs::copy(path, &backup_path)
        .context("Failed to create backup of boxes.json")?;
    
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

/// Generate a write-up markdown for a box, skipping empty sections
pub fn generate_writeup(ctf_box: &CtfBox) -> String {
    let mut md = String::new();
    
    // Header
    md.push_str(&format!("# {} - Write-up\n\n", ctf_box.title));
    md.push_str(&format!("**Platform:** {}  \n", ctf_box.platform));
    md.push_str(&format!("**IP:** {}  \n", ctf_box.ip_address));
    if !ctf_box.tags.is_empty() {
        md.push_str(&format!("**Tags:** {}  \n", ctf_box.tags.join(", ")));
    }
    md.push_str(&format!("**Date:** {}  \n\n", ctf_box.created_date.format("%Y-%m-%d")));
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
    let misc_notes: Vec<_> = ctf_box.notes.iter()
        .filter(|n| matches!(n.category, 
            crate::models::NoteCategory::Misc | 
            crate::models::NoteCategory::Crypto | 
            crate::models::NoteCategory::Pwn | 
            crate::models::NoteCategory::Stego | 
            crate::models::NoteCategory::Reversing))
        .collect();

    // Collect commands by category
    let recon_commands: Vec<_> = ctf_box.actions.iter()
        .filter(|a| a.command.contains("nmap") || a.command.contains("ping") || a.command.contains("whois"))
        .collect();
    let enum_commands: Vec<_> = ctf_box.actions.iter()
        .filter(|a| {
            a.command.contains("gobuster") || a.command.contains("ffuf") || 
            a.command.contains("nikto") || a.command.contains("dirb") ||
            a.command.contains("enum4linux") || a.command.contains("smbclient")
        })
        .collect();
    let exploit_commands: Vec<_> = ctf_box.actions.iter()
        .filter(|a| {
            a.command.contains("exploit") || a.command.contains("msfconsole") || 
            a.command.contains("searchsploit") || a.command.contains("sqlmap") ||
            a.command.contains("hydra") || a.command.contains("nc ") || a.command.contains("reverse")
        })
        .collect();
    let privesc_commands: Vec<_> = ctf_box.actions.iter()
        .filter(|a| {
            a.command.contains("sudo") || a.command.contains("linpeas") || 
            a.command.contains("linenum") || a.command.contains("SUID") ||
            a.command.contains("getcap") || a.command.contains("find / ")
        })
        .collect();
    let other_commands: Vec<_> = ctf_box.actions.iter()
        .filter(|a| {
            !a.command.contains("nmap") && !a.command.contains("ping") &&
            !a.command.contains("gobuster") && !a.command.contains("ffuf") &&
            !a.command.contains("nikto") && !a.command.contains("exploit") &&
            !a.command.contains("msfconsole") && !a.command.contains("searchsploit") &&
            !a.command.contains("sqlmap") && !a.command.contains("hydra") &&
            !a.command.contains("sudo") && !a.command.contains("linpeas") &&
            a.output.is_some()
        })
        .collect();

    // Build dynamic table of contents
    let mut toc = Vec::new();
    let has_recon = !recon_notes.is_empty() || !recon_commands.is_empty();
    let has_enum = !web_notes.is_empty() || !enum_commands.is_empty();
    let has_exploit = !foothold_notes.is_empty() || !exploit_commands.is_empty();
    let has_privesc = !privesc_notes.is_empty() || !privesc_commands.is_empty();
    let has_misc = !misc_notes.is_empty();
    let has_other = !other_commands.is_empty();

    if has_recon { toc.push(("Reconnaissance", "reconnaissance")); }
    if has_enum { toc.push(("Enumeration", "enumeration")); }
    if has_exploit { toc.push(("Exploitation", "exploitation")); }
    if has_privesc { toc.push(("Privilege Escalation", "privilege-escalation")); }
    toc.push(("Flags", "flags"));
    if has_misc { toc.push(("Additional Notes", "additional-notes")); }
    if has_other { toc.push(("Command Log", "command-log")); }

    md.push_str("## Table of Contents\n\n");
    for (i, (title, anchor)) in toc.iter().enumerate() {
        md.push_str(&format!("{}. [{}](#{})\n", i + 1, title, anchor));
    }
    md.push_str("\n---\n\n");
    
    // Reconnaissance
    if has_recon {
        md.push_str("## Reconnaissance\n\n");
        for note in &recon_notes {
            md.push_str(&format!("- {}\n", note.content));
        }
        if !recon_notes.is_empty() { md.push_str("\n"); }
        for action in &recon_commands {
            render_action(&mut md, action, 3000);
        }
    }
    
    // Enumeration
    if has_enum {
        md.push_str("## Enumeration\n\n");
        for note in &web_notes {
            md.push_str(&format!("- {}\n", note.content));
        }
        if !web_notes.is_empty() { md.push_str("\n"); }
        for action in &enum_commands {
            render_action(&mut md, action, 3000);
        }
    }
    
    // Exploitation
    if has_exploit {
        md.push_str("## Exploitation\n\n");
        for note in &foothold_notes {
            md.push_str(&format!("- {}\n", note.content));
        }
        if !foothold_notes.is_empty() { md.push_str("\n"); }
        for action in &exploit_commands {
            render_action(&mut md, action, 2000);
        }
    }
    
    // Privilege Escalation
    if has_privesc {
        md.push_str("## Privilege Escalation\n\n");
        for note in &privesc_notes {
            md.push_str(&format!("- {}\n", note.content));
        }
        if !privesc_notes.is_empty() { md.push_str("\n"); }
        for action in &privesc_commands {
            render_action(&mut md, action, 2000);
        }
    }
    
    // Flags — always present
    md.push_str("## Flags\n\n");
    md.push_str("### User Flag\n\n");
    md.push_str("```\n[USER FLAG HERE]\n```\n\n");
    md.push_str("### Root Flag\n\n");
    md.push_str("```\n[ROOT FLAG HERE]\n```\n\n");
    
    // Additional Notes
    if has_misc {
        md.push_str("---\n\n## Additional Notes\n\n");
        for note in &misc_notes {
            md.push_str(&format!("- **{:?}:** {}\n", note.category, note.content));
        }
        md.push_str("\n");
    }
    
    // Command Log
    if has_other {
        md.push_str("---\n\n## Command Log\n\n");
        for action in &other_commands {
            let result_icon = match action.result {
                crate::models::ActionResult::Success => "✓",
                crate::models::ActionResult::Fail => "✗",
                crate::models::ActionResult::Unknown => "?",
            };
            md.push_str(&format!("### {} `{}`\n\n", result_icon, action.command));
            if let Some(output) = &action.output {
                md.push_str("```\n");
                let truncated = truncate_output(output, 1500);
                md.push_str(&truncated);
                if !truncated.ends_with('\n') { md.push_str("\n"); }
                md.push_str("```\n\n");
            }
        }
    }
    
    md.push_str("\n---\n\n*Generated by CTF Brain*\n");
    md
}

/// Render a single action as a markdown code block
fn render_action(md: &mut String, action: &crate::models::Action, max_output: usize) {
    let tool_name = action.command.split_whitespace().next().unwrap_or("Command");
    md.push_str(&format!("### {}\n\n", tool_name));
    md.push_str(&format!("```bash\n$ {}\n", action.command));
    if let Some(output) = &action.output {
        let truncated = truncate_output(output, max_output);
        md.push_str(&truncated);
        if !truncated.ends_with('\n') { md.push_str("\n"); }
    }
    md.push_str("```\n\n");
    if let Some(note) = &action.note {
        md.push_str(&format!("> **Note:** {}\n\n", note));
    }
}

/// Truncate output to max_len chars
fn truncate_output(output: &str, max_len: usize) -> String {
    if output.len() > max_len {
        format!("{}...\n[Output truncated]", &output[..max_len])
    } else {
        output.to_string()
    }
}
