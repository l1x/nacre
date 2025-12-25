use serde::{Deserialize, Serialize};
use std::process::Command;
use std::io;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Issue {
    pub id: String,
    pub title: String,
    pub status: Status,
    pub priority: Option<u8>,
    pub issue_type: IssueType,
    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub assignee: Option<String>,
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Open,
    InProgress,
    Blocked,
    Deferred,
    Closed,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum IssueType {
    Task,
    Bug,
    Feature,
    Epic,
    Chore,
    MergeRequest,
    Molecule,
}

pub struct Client {
    bin_path: String,
}

impl Client {
    pub fn new() -> Self {
        let bin_path = std::env::var("BD_BIN").unwrap_or_else(|_| "bd".to_string());
        Self { bin_path }
    }

    pub fn list_issues(&self) -> io::Result<Vec<Issue>> {
        let output = Command::new(&self.bin_path)
            .arg("list")
            .arg("--json")
            .arg("--all")
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(io::Error::new(io::ErrorKind::Other, error_msg.to_string()));
        }

        let issues: Vec<Issue> = serde_json::from_slice(&output.stdout)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(issues)
    }
}
