use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteCategory {
    Privesc,
    Web,
    Recon,
    Foothold,
    Misc,
    Crypto,
    Pwn,
    Stego,
    Reversing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub category: NoteCategory,
    pub content: String,
    pub created_date: DateTime<Utc>,
}