use chrono::{DateTime, Utc};
use std::net::IpAddr;
use crate::models::{Action, Note};
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
}
