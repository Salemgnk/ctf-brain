use chrono::{DateTime, Utc};
use std::net::IpAddr;
use std::collections::HashMap;
use super::{Action, Note};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtfBox {
    pub id: i32,
    pub title: String,
    pub platform: String,
    pub ip_address: IpAddr,
    pub tags: Vec<String>,
    pub created_date: DateTime<Utc>,
    pub updated_date: DateTime<Utc>,
    pub actions: Vec<Action>,
    pub notes: Vec<Note>,
    
    // Custom environment variables for this box
    // Default to empty HashMap if not present in JSON (backward compatibility)
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
}
