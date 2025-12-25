use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::BufRead;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BeadsError {
    #[error("Command execution failed: {0}")]
    CommandFailed(#[from] std::io::Error),

    #[error("Beads command returned error: {0}")]
    CommandError(String),

    #[error("Failed to parse response: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Issue not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, BeadsError>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Issue {
    pub id: String,
    pub title: String,
    pub status: Status,
    pub priority: Option<u8>,
    pub issue_type: IssueType,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub closed_at: Option<DateTime<FixedOffset>>,
    pub assignee: Option<String>,
    pub labels: Option<Vec<String>>,
    pub description: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub close_reason: Option<String>,
    pub estimate: Option<u32>,
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Activity {
    pub timestamp: DateTime<FixedOffset>,
    pub r#type: String,
    pub issue_id: String,
    pub message: String,
    pub old_status: Option<Status>,
    pub new_status: Option<Status>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dependency {
    pub issue_id: String,
    pub depends_on_id: String,
    #[serde(rename = "type")]
    pub dep_type: String,
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

    /// Returns sort order (lower = higher priority in list)
    pub fn sort_order(&self) -> u8 {
        match self {
            Status::InProgress => 0,
            Status::Blocked => 1,
            Status::Open => 2,
            Status::Deferred => 3,
            Status::Closed => 4,
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

impl IssueType {
    /// Returns a CSS-friendly class name (lowercase, no spaces)
    pub fn as_css_class(&self) -> &'static str {
        match self {
            IssueType::Task => "task",
            IssueType::Bug => "bug",
            IssueType::Feature => "feature",
            IssueType::Epic => "epic",
            IssueType::Chore => "chore",
            IssueType::MergeRequest => "merge-request",
            IssueType::Molecule => "molecule",
        }
    }
}

#[derive(Clone)]
pub struct Client {
    bin_path: String,
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
pub struct IssueUpdate {
    pub title: Option<String>,
    pub status: Option<Status>,
}

#[derive(Debug, Deserialize)]
pub struct IssueCreate {
    pub title: String,
    pub issue_type: Option<String>,
    pub priority: Option<u8>,
    pub description: Option<String>,
}

impl Client {
    pub fn new() -> Self {
        let bin_path = std::env::var("BD_BIN").unwrap_or_else(|_| "bd".to_string());
        Self { bin_path }
    }

    pub fn list_issues(&self) -> Result<Vec<Issue>> {
        let output = Command::new(&self.bin_path).arg("export").output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(BeadsError::CommandError(error_msg.to_string()));
        }

        let mut issues = Vec::new();
        for line in output.stdout.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let issue: Issue = serde_json::from_str(&line)?;
            issues.push(issue);
        }

        Ok(issues)
    }

    pub fn get_issue(&self, id: &str) -> Result<Issue> {
        let issues = self.list_issues()?;
        issues
            .into_iter()
            .find(|i| i.id == id)
            .ok_or_else(|| BeadsError::NotFound(id.to_string()))
    }

    pub fn update_issue(&self, id: &str, update: IssueUpdate) -> Result<()> {
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
            return Err(BeadsError::CommandError(error_msg.to_string()));
        }

        Ok(())
    }

    pub fn create_issue(&self, create: IssueCreate) -> Result<String> {
        let mut cmd = Command::new(&self.bin_path);
        cmd.arg("create")
            .arg("--title")
            .arg(&create.title)
            .arg("--silent");

        if let Some(issue_type) = &create.issue_type {
            cmd.arg("--type").arg(issue_type);
        }
        if let Some(priority) = create.priority {
            cmd.arg("--priority").arg(priority.to_string());
        }
        if let Some(description) = &create.description {
            cmd.arg("--description").arg(description);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(BeadsError::CommandError(error_msg.to_string()));
        }

        // bd create --silent outputs just the issue ID
        let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(id)
    }

    pub fn get_activity(&self) -> Result<Vec<Activity>> {
        let output = Command::new(&self.bin_path)
            .arg("activity")
            .arg("--json")
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(BeadsError::CommandError(error_msg.to_string()));
        }

        let activities: Vec<Activity> = serde_json::from_slice(&output.stdout)?;
        Ok(activities)
    }

    pub fn get_status_summary(&self) -> Result<serde_json::Value> {
        let output = Command::new(&self.bin_path)
            .arg("status")
            .arg("--json")
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(BeadsError::CommandError(error_msg.to_string()));
        }

        let summary: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        Ok(summary)
    }
}
