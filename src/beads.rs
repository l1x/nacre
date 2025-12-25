use serde::{Deserialize, Serialize};
use std::fmt;
use std::io;
use std::process::Command;

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

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Open => write!(f, "Open"),
            Status::InProgress => write!(f, "In Progress"),
            Status::Blocked => write!(f, "Blocked"),
            Status::Deferred => write!(f, "Deferred"),
            Status::Closed => write!(f, "Closed"),
        }
    }
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Open => "open",
            Status::InProgress => "in_progress",
            Status::Blocked => "blocked",
            Status::Deferred => "deferred",
            Status::Closed => "closed",
        }
    }
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

impl fmt::Display for IssueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueType::Task => write!(f, "Task"),
            IssueType::Bug => write!(f, "Bug"),
            IssueType::Feature => write!(f, "Feature"),
            IssueType::Epic => write!(f, "Epic"),
            IssueType::Chore => write!(f, "Chore"),
            IssueType::MergeRequest => write!(f, "Merge Request"),
            IssueType::Molecule => write!(f, "Molecule"),
        }
    }
}

pub struct Client {
    bin_path: String,
}

#[derive(Debug, Deserialize)]
pub struct IssueUpdate {
    pub title: Option<String>,
    pub status: Option<Status>,
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
            return Err(io::Error::other(error_msg.to_string()));
        }

        let issues: Vec<Issue> = serde_json::from_slice(&output.stdout)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(issues)
    }

    pub fn update_issue(&self, id: &str, update: IssueUpdate) -> io::Result<()> {
        let mut cmd = Command::new(&self.bin_path);
        cmd.arg("update").arg(id);

        if let Some(title) = &update.title {
            cmd.arg("--title").arg(title);
        }
        if let Some(status) = &update.status {
            cmd.arg("--status").arg(status.as_str());
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(io::Error::other(error_msg.to_string()));
        }

        Ok(())
    }
}
