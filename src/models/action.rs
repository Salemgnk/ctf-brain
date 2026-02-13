use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionResult {
    Success,
    Fail,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub timestamp: DateTime<Utc>,
    pub command: String,
    pub result: ActionResult,
    pub note: Option<String>,
    #[serde(default)]
    pub output: Option<String>,
}

