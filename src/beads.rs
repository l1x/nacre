use serde::{Deserialize, Serialize};
use std::fmt;
use std::process::Command;
use thiserror::Error;
use time::OffsetDateTime;

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
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub closed_at: Option<OffsetDateTime>,
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
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
    #[serde(rename = "type")]
    pub r#type: EventType,
    pub issue_id: String,
    pub message: String,
    pub old_status: Option<Status>,
    pub new_status: Option<Status>,
}

/// Defines relationships between issues.
///
/// This enum mirrors the Go Beads DependencyType type from `internal/types/types.go`.
/// Each variant represents a specific type of relationship with clear semantics.
///
/// Workflow types affect ready work calculation:
/// - `Blocks`: Standard blocking relationship
/// - `ParentChild`: Hierarchical parent-child relationship  
/// - `ConditionalBlocks`: B runs only if A fails (bd-kzda)
/// - `WaitsFor`: Fanout gate: wait for dynamic children (bd-xo1o.2)
///
/// Association types don't affect workflow:
/// - `Related`: General association
/// - `DiscoveredFrom`: Discovery relationship
///
/// Graph link types (bd-kwro):
/// - `RepliesTo`: Conversation threading
/// - `RelatesTo`: Loose knowledge graph edges
/// - `Duplicates`: Deduplication link
/// - `Supersedes`: Version chain link
///
/// Entity types (HOP foundation - Decision 004):
/// - `AuthoredBy`: Creator relationship
/// - `AssignedTo`: Assignment relationship
/// - `ApprovedBy`: Approval relationship
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum DependencyType {
    // Workflow types (affect ready work calculation)
    /// Standard blocking relationship
    #[default]
    Blocks,
    /// Hierarchical parent-child relationship
    ParentChild,
    /// B runs only if A fails (bd-kzda)
    ConditionalBlocks,
    /// Fanout gate: wait for dynamic children (bd-xo1o.2)
    WaitsFor,

    // Association types
    /// General association
    Related,
    /// Discovery relationship
    DiscoveredFrom,

    // Graph link types (bd-kwro)
    /// Conversation threading
    RepliesTo,
    /// Loose knowledge graph edges
    RelatesTo,
    /// Deduplication link
    Duplicates,
    /// Version chain link
    Supersedes,

    // Entity types (HOP foundation - Decision 004)
    /// Creator relationship
    AuthoredBy,
    /// Assignment relationship
    AssignedTo,
    /// Approval relationship
    ApprovedBy,
}

impl DependencyType {
    /// Returns the kebab-case string representation used by Beads CLI/API
    pub fn as_str(&self) -> &'static str {
        match self {
            DependencyType::Blocks => "blocks",
            DependencyType::ParentChild => "parent-child",
            DependencyType::ConditionalBlocks => "conditional-blocks",
            DependencyType::WaitsFor => "waits-for",
            DependencyType::Related => "related",
            DependencyType::DiscoveredFrom => "discovered-from",
            DependencyType::RepliesTo => "replies-to",
            DependencyType::RelatesTo => "relates-to",
            DependencyType::Duplicates => "duplicates",
            DependencyType::Supersedes => "supersedes",
            DependencyType::AuthoredBy => "authored-by",
            DependencyType::AssignedTo => "assigned-to",
            DependencyType::ApprovedBy => "approved-by",
        }
    }

    /// Returns true if this dependency type affects workflow calculations
    pub fn affects_workflow(&self) -> bool {
        matches!(
            self,
            DependencyType::Blocks
                | DependencyType::ParentChild
                | DependencyType::ConditionalBlocks
                | DependencyType::WaitsFor
        )
    }

    /// Returns true if this is a valid dependency type variant
    pub fn is_valid(&self) -> bool {
        true
    }
}

/// Categorizes audit trail events.
///
/// This enum mirrors the Go Beads EventType type from `internal/types/types.go`.
/// Each variant represents a specific type of event that can occur in the system.
///
/// Core workflow events:
/// - `Created`: Issue was created
/// - `Updated`: General issue update
/// - `StatusChanged`: Issue status changed
/// - `Closed`: Issue was closed
/// - `Reopened`: Previously closed issue was reopened
///
/// Content events:
/// - `Commented`: Comment was added
///
/// Relationship events:
/// - `DependencyAdded`: Dependency relationship was added
/// - `DependencyRemoved`: Dependency relationship was removed
///
/// Organization events:
/// - `LabelAdded`: Label was added to issue
/// - `LabelRemoved`: Label was removed from issue
///
/// System events:
/// - `Compacted`: Database compaction event
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum EventType {
    /// Issue was created
    #[default]
    #[serde(rename = "create")]
    Created,
    /// General issue update
    #[serde(rename = "update")]
    Updated,
    /// Issue status changed
    #[serde(rename = "status")]
    StatusChanged,
    /// Comment was added
    Commented,
    /// Issue was closed
    Closed,
    /// Previously closed issue was reopened
    Reopened,
    /// Dependency relationship was added
    #[serde(rename = "dependency_added")]
    DependencyAdded,
    /// Dependency relationship was removed
    #[serde(rename = "dependency_removed")]
    DependencyRemoved,
    /// Label was added to issue
    #[serde(rename = "label_added")]
    LabelAdded,
    /// Label was removed from issue
    #[serde(rename = "label_removed")]
    LabelRemoved,
    /// Database compaction event
    Compacted,
    /// Issue was deleted
    #[serde(rename = "delete")]
    Deleted,
}

impl EventType {
    /// Returns the string representation used by Beads CLI/API
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::Created => "create",
            EventType::Updated => "update",
            EventType::StatusChanged => "status",
            EventType::Commented => "commented",
            EventType::Closed => "closed",
            EventType::Reopened => "reopened",
            EventType::DependencyAdded => "dependency_added",
            EventType::DependencyRemoved => "dependency_removed",
            EventType::LabelAdded => "label_added",
            EventType::LabelRemoved => "label_removed",
            EventType::Compacted => "compacted",
            EventType::Deleted => "delete",
        }
    }

    /// Returns true if this is a valid event type variant
    pub fn is_valid(&self) -> bool {
        true
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dependency {
    pub issue_id: String,
    pub depends_on_id: String,
    #[serde(rename = "type")]
    pub dep_type: DependencyType,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub created_at: Option<OffsetDateTime>,
    pub created_by: Option<String>,
}

/// Represents the current state of an issue in the workflow.
///
/// This enum mirrors the Go Beads Status type from `internal/types/types.go`.
/// Each variant represents a specific workflow state with clear semantics:
///
/// - `Open`: New issue ready for work consideration
/// - `InProgress`: Actively being worked on
/// - `Blocked`: Waiting on external dependencies or blockers  
/// - `Deferred`: Deliberately put on ice for later (bd-4jr)
/// - `Closed`: Completed or resolved
/// - `Tombstone`: Soft-deleted issue (bd-vw8)
/// - `Pinned`: Persistent bead that stays open indefinitely (bd-6v2)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// New issue ready for work consideration
    #[default]
    Open,
    /// Actively being worked on
    InProgress,
    /// Waiting on external dependencies or blockers
    Blocked,
    /// Deliberately put on ice for later (bd-4jr)
    Deferred,
    /// Completed or resolved
    Closed,
    /// Soft-deleted issue (bd-vw8)
    Tombstone,
    /// Persistent bead that stays open indefinitely (bd-6v2)
    Pinned,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Open => write!(f, "Open"),
            Status::InProgress => write!(f, "In Progress"),
            Status::Blocked => write!(f, "Blocked"),
            Status::Deferred => write!(f, "Deferred"),
            Status::Closed => write!(f, "Closed"),
            Status::Tombstone => write!(f, "Tombstone"),
            Status::Pinned => write!(f, "Pinned"),
        }
    }
}

impl Status {
    /// Returns the snake_case string representation used by Beads CLI/API
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Open => "open",
            Status::InProgress => "in_progress",
            Status::Blocked => "blocked",
            Status::Deferred => "deferred",
            Status::Closed => "closed",
            Status::Tombstone => "tombstone",
            Status::Pinned => "pinned",
        }
    }

    /// Returns true if this is a valid status variant
    /// All defined variants are considered valid by definition
    pub fn is_valid(&self) -> bool {
        true
    }

    /// Returns sort order (lower = higher priority in list)
    /// Active work items first, then planning, then resolved/archived
    pub fn sort_order(&self) -> u8 {
        match self {
            Status::InProgress => 0,
            Status::Blocked => 1,
            Status::Pinned => 2, // Persistent items should stay visible
            Status::Open => 3,
            Status::Deferred => 4,
            Status::Closed => 5,
            Status::Tombstone => 6,
        }
    }
}

/// Categorizes the kind of work an issue represents.
///
/// This enum mirrors the Go Beads IssueType type from `internal/types/types.go`.
/// Each variant represents a specific category of work with distinct semantics:
///
/// - `Bug`: Defect or error that needs fixing
/// - `Feature`: New functionality to be added
/// - `Task`: General work item that doesn't fit other categories
/// - `Epic`: Large work item that encompasses multiple sub-tasks
/// - `Chore`: Maintenance or housekeeping work
/// - `Message`: Ephemeral communication between workers
/// - `MergeRequest`: Merge queue entry for refinery processing
/// - `Molecule`: Template molecule for issue hierarchies (beads-1ra)
/// - `Gate`: Async coordination gate (bd-udsi)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum IssueType {
    /// Defect or error that needs fixing
    Bug,
    /// New functionality to be added
    Feature,
    /// General work item that doesn't fit other categories
    #[default]
    Task,
    /// Large work item that encompasses multiple sub-tasks
    Epic,
    /// Maintenance or housekeeping work
    Chore,
    /// Ephemeral communication between workers
    Message,
    /// Merge queue entry for refinery processing
    #[serde(rename = "merge-request")]
    MergeRequest,
    /// Template molecule for issue hierarchies (beads-1ra)
    Molecule,
    /// Async coordination gate (bd-udsi)
    Gate,
}

impl fmt::Display for IssueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueType::Task => write!(f, "Task"),
            IssueType::Bug => write!(f, "Bug"),
            IssueType::Feature => write!(f, "Feature"),
            IssueType::Epic => write!(f, "Epic"),
            IssueType::Chore => write!(f, "Chore"),
            IssueType::Message => write!(f, "Message"),
            IssueType::MergeRequest => write!(f, "Merge Request"),
            IssueType::Molecule => write!(f, "Molecule"),
            IssueType::Gate => write!(f, "Gate"),
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
            IssueType::Message => "message",
            IssueType::MergeRequest => "merge-request",
            IssueType::Molecule => "molecule",
            IssueType::Gate => "gate",
        }
    }

    /// Returns the kebab-case string representation used by Beads CLI/API
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueType::Task => "task",
            IssueType::Bug => "bug",
            IssueType::Feature => "feature",
            IssueType::Epic => "epic",
            IssueType::Chore => "chore",
            IssueType::Message => "message",
            IssueType::MergeRequest => "merge-request",
            IssueType::Molecule => "molecule",
            IssueType::Gate => "gate",
        }
    }

    /// Returns true if this is a valid issue type variant
    /// All defined variants are considered valid by definition
    pub fn is_valid(&self) -> bool {
        true
    }
}

#[derive(Clone)]
pub struct Client {
    bin_path: String,
    db_path: Option<String>,
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
    pub priority: Option<u8>,
    pub description: Option<String>,
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
        let db_path = std::env::var("BEADS_DB").ok();
        Self { bin_path, db_path }
    }

    pub fn with_db(mut self, path: String) -> Self {
        self.db_path = Some(path);
        self
    }

    fn base_command(&self) -> Command {
        let mut cmd = Command::new(&self.bin_path);
        if let Some(db) = &self.db_path {
            cmd.arg("--db").arg(db);
        }
        cmd
    }

    pub fn list_issues(&self) -> Result<Vec<Issue>> {
        let output = self.base_command().args(["list", "--json"]).output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(BeadsError::CommandError(error_msg.to_string()));
        }

        let issues: Vec<Issue> = serde_json::from_slice(&output.stdout)?;
        Ok(issues)
    }

    /// List all issues including closed (for stats and metrics)
    pub fn list_all_issues(&self) -> Result<Vec<Issue>> {
        let output = self
            .base_command()
            .args(["list", "--json", "--all", "--limit", "0"])
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(BeadsError::CommandError(error_msg.to_string()));
        }

        let issues: Vec<Issue> = serde_json::from_slice(&output.stdout)?;
        Ok(issues)
    }

    pub fn get_issue(&self, id: &str) -> Result<Issue> {
        let output = self
            .base_command()
            .arg("show")
            .arg(id)
            .arg("--json")
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            if error_msg.to_lowercase().contains("not found") {
                return Err(BeadsError::NotFound(id.to_string()));
            }
            return Err(BeadsError::CommandError(error_msg.to_string()));
        }

        #[derive(Deserialize)]
        struct IssueFromShow {
            pub id: String,
            pub title: String,
            pub status: Status,
            pub priority: Option<u8>,
            pub issue_type: IssueType,
            #[serde(with = "time::serde::rfc3339")]
            pub created_at: OffsetDateTime,
            #[serde(with = "time::serde::rfc3339")]
            pub updated_at: OffsetDateTime,
            #[serde(default, with = "time::serde::rfc3339::option")]
            pub closed_at: Option<OffsetDateTime>,
            pub assignee: Option<String>,
            pub labels: Option<Vec<String>>,
            pub description: Option<String>,
            pub acceptance_criteria: Option<String>,
            pub close_reason: Option<String>,
            pub estimate: Option<u32>,
            #[serde(default)]
            #[allow(dead_code)]
            pub dependencies: Vec<serde_json::Value>,
        }

        let issues: Vec<IssueFromShow> = serde_json::from_slice(&output.stdout)?;
        let issue_show = issues
            .into_iter()
            .next()
            .ok_or_else(|| BeadsError::NotFound(id.to_string()))?;

        Ok(Issue {
            id: issue_show.id,
            title: issue_show.title,
            status: issue_show.status,
            priority: issue_show.priority,
            issue_type: issue_show.issue_type,
            created_at: issue_show.created_at,
            updated_at: issue_show.updated_at,
            closed_at: issue_show.closed_at,
            assignee: issue_show.assignee,
            labels: issue_show.labels,
            description: issue_show.description,
            acceptance_criteria: issue_show.acceptance_criteria,
            close_reason: issue_show.close_reason,
            estimate: issue_show.estimate,
            dependencies: vec![],
        })
    }

    pub fn update_issue(&self, id: &str, update: IssueUpdate) -> Result<()> {
        let mut cmd = self.base_command();
        cmd.arg("update").arg(id);

        if let Some(title) = &update.title {
            cmd.arg("--title").arg(title);
        }
        if let Some(status) = &update.status {
            cmd.arg("--status").arg(status.as_str());
        }
        if let Some(priority) = update.priority {
            cmd.arg("--priority").arg(priority.to_string());
        }
        if let Some(description) = &update.description {
            cmd.arg("--description").arg(description);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(BeadsError::CommandError(error_msg.to_string()));
        }

        Ok(())
    }

    pub fn create_issue(&self, create: IssueCreate) -> Result<String> {
        let mut cmd = self.base_command();
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
        // Use a high limit to ensure we get all InProgress transitions needed for cycle time
        let output = self
            .base_command()
            .arg("activity")
            .arg("--json")
            .arg("--limit")
            .arg("10000")
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(BeadsError::CommandError(error_msg.to_string()));
        }

        let activities: Vec<Activity> = serde_json::from_slice(&output.stdout)?;
        Ok(activities)
    }

    pub fn get_status_summary(&self) -> Result<serde_json::Value> {
        let output = self.base_command().arg("status").arg("--json").output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(BeadsError::CommandError(error_msg.to_string()));
        }

        let summary: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        Ok(summary)
    }

    /// List all dependencies by reading the JSONL file directly.
    ///
    /// This is necessary because `bd list --json` does not include dependencies.
    /// The JSONL file contains the full dependency information for each issue.
    pub fn list_all_dependencies(&self) -> Result<Vec<Dependency>> {
        // Find the .beads directory
        let beads_dir = self.find_beads_dir()?;
        let jsonl_path = beads_dir.join("issues.jsonl");

        if !jsonl_path.exists() {
            return Ok(vec![]);
        }

        let content = std::fs::read_to_string(&jsonl_path)
            .map_err(|e| BeadsError::CommandError(format!("Failed to read JSONL: {}", e)))?;

        let mut all_dependencies = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Parse each line as a JSON object with dependencies
            #[derive(Deserialize)]
            struct IssueWithDeps {
                #[serde(default)]
                dependencies: Vec<Dependency>,
            }

            if let Ok(issue) = serde_json::from_str::<IssueWithDeps>(line) {
                all_dependencies.extend(issue.dependencies);
            }
        }

        Ok(all_dependencies)
    }

    /// Find the .beads directory by walking up from the current directory
    fn find_beads_dir(&self) -> Result<std::path::PathBuf> {
        if let Some(db_path) = &self.db_path {
            let path = std::path::Path::new(db_path);
            if let Some(parent) = path.parent() {
                return Ok(parent.to_path_buf());
            }
        }

        let mut current = std::env::current_dir()
            .map_err(|e| BeadsError::CommandError(format!("Failed to get current dir: {}", e)))?;

        loop {
            let beads_dir = current.join(".beads");
            if beads_dir.is_dir() {
                return Ok(beads_dir);
            }

            if !current.pop() {
                return Err(BeadsError::CommandError(
                    "No .beads directory found".to_string(),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    // Status enum tests
    #[test]
    fn test_status_serialization() {
        let status = Status::InProgress;
        let serialized = serde_json::to_string(&status).unwrap();
        assert_eq!(serialized, "\"in_progress\"");
    }

    #[test]
    fn test_status_deserialization() {
        let json = "\"in_progress\"";
        let status: Status = serde_json::from_str(json).unwrap();
        assert_eq!(status, Status::InProgress);
    }

    #[test]
    fn test_status_display() {
        assert_eq!(Status::Open.to_string(), "Open");
        assert_eq!(Status::InProgress.to_string(), "In Progress");
        assert_eq!(Status::Blocked.to_string(), "Blocked");
        assert_eq!(Status::Deferred.to_string(), "Deferred");
        assert_eq!(Status::Closed.to_string(), "Closed");
        assert_eq!(Status::Tombstone.to_string(), "Tombstone");
        assert_eq!(Status::Pinned.to_string(), "Pinned");
    }

    #[test]
    fn test_status_as_str() {
        assert_eq!(Status::Open.as_str(), "open");
        assert_eq!(Status::InProgress.as_str(), "in_progress");
        assert_eq!(Status::Blocked.as_str(), "blocked");
        assert_eq!(Status::Deferred.as_str(), "deferred");
        assert_eq!(Status::Closed.as_str(), "closed");
        assert_eq!(Status::Tombstone.as_str(), "tombstone");
        assert_eq!(Status::Pinned.as_str(), "pinned");
    }

    #[test]
    fn test_status_sort_order() {
        assert_eq!(Status::InProgress.sort_order(), 0);
        assert_eq!(Status::Blocked.sort_order(), 1);
        assert_eq!(Status::Pinned.sort_order(), 2);
        assert_eq!(Status::Open.sort_order(), 3);
        assert_eq!(Status::Deferred.sort_order(), 4);
        assert_eq!(Status::Closed.sort_order(), 5);
        assert_eq!(Status::Tombstone.sort_order(), 6);
    }

    #[test]
    fn test_status_default() {
        assert_eq!(Status::default(), Status::Open);
    }

    #[test]
    fn test_status_is_valid() {
        assert!(Status::Open.is_valid());
        assert!(Status::InProgress.is_valid());
        assert!(Status::Blocked.is_valid());
        assert!(Status::Deferred.is_valid());
        assert!(Status::Closed.is_valid());
        assert!(Status::Tombstone.is_valid());
        assert!(Status::Pinned.is_valid());
    }

    // IssueType enum tests
    #[test]
    fn test_issue_type_serialization() {
        let issue_type = IssueType::MergeRequest;
        let serialized = serde_json::to_string(&issue_type).unwrap();
        assert_eq!(serialized, "\"merge-request\"");
    }

    #[test]
    fn test_issue_type_deserialization() {
        let json = "\"merge-request\"";
        let issue_type: IssueType = serde_json::from_str(json).unwrap();
        assert_eq!(issue_type, IssueType::MergeRequest);
    }

    #[test]
    fn test_issue_type_display() {
        assert_eq!(IssueType::Task.to_string(), "Task");
        assert_eq!(IssueType::Bug.to_string(), "Bug");
        assert_eq!(IssueType::Feature.to_string(), "Feature");
        assert_eq!(IssueType::Epic.to_string(), "Epic");
        assert_eq!(IssueType::Chore.to_string(), "Chore");
        assert_eq!(IssueType::Message.to_string(), "Message");
        assert_eq!(IssueType::MergeRequest.to_string(), "Merge Request");
        assert_eq!(IssueType::Molecule.to_string(), "Molecule");
        assert_eq!(IssueType::Gate.to_string(), "Gate");
    }

    #[test]
    fn test_issue_type_as_str() {
        assert_eq!(IssueType::Task.as_str(), "task");
        assert_eq!(IssueType::Bug.as_str(), "bug");
        assert_eq!(IssueType::Feature.as_str(), "feature");
        assert_eq!(IssueType::Epic.as_str(), "epic");
        assert_eq!(IssueType::Chore.as_str(), "chore");
        assert_eq!(IssueType::Message.as_str(), "message");
        assert_eq!(IssueType::MergeRequest.as_str(), "merge-request");
        assert_eq!(IssueType::Molecule.as_str(), "molecule");
        assert_eq!(IssueType::Gate.as_str(), "gate");
    }

    #[test]
    fn test_issue_type_css_class() {
        assert_eq!(IssueType::Task.as_css_class(), "task");
        assert_eq!(IssueType::Bug.as_css_class(), "bug");
        assert_eq!(IssueType::Feature.as_css_class(), "feature");
        assert_eq!(IssueType::Epic.as_css_class(), "epic");
        assert_eq!(IssueType::Chore.as_css_class(), "chore");
        assert_eq!(IssueType::Message.as_css_class(), "message");
        assert_eq!(IssueType::MergeRequest.as_css_class(), "merge-request");
        assert_eq!(IssueType::Molecule.as_css_class(), "molecule");
        assert_eq!(IssueType::Gate.as_css_class(), "gate");
    }

    #[test]
    fn test_issue_type_default() {
        assert_eq!(IssueType::default(), IssueType::Task);
    }

    #[test]
    fn test_issue_type_is_valid() {
        assert!(IssueType::Task.is_valid());
        assert!(IssueType::Bug.is_valid());
        assert!(IssueType::Feature.is_valid());
        assert!(IssueType::Epic.is_valid());
        assert!(IssueType::Chore.is_valid());
        assert!(IssueType::Message.is_valid());
        assert!(IssueType::MergeRequest.is_valid());
        assert!(IssueType::Molecule.is_valid());
        assert!(IssueType::Gate.is_valid());
    }

    // DependencyType enum tests
    #[test]
    fn test_dependency_type_serialization() {
        let dep_type = DependencyType::ConditionalBlocks;
        let serialized = serde_json::to_string(&dep_type).unwrap();
        assert_eq!(serialized, "\"conditional-blocks\"");
    }

    #[test]
    fn test_dependency_type_deserialization() {
        let json = "\"conditional-blocks\"";
        let dep_type: DependencyType = serde_json::from_str(json).unwrap();
        assert_eq!(dep_type, DependencyType::ConditionalBlocks);
    }

    #[test]
    fn test_dependency_type_affects_workflow() {
        assert!(DependencyType::Blocks.affects_workflow());
        assert!(DependencyType::ParentChild.affects_workflow());
        assert!(DependencyType::ConditionalBlocks.affects_workflow());
        assert!(DependencyType::WaitsFor.affects_workflow());

        assert!(!DependencyType::Related.affects_workflow());
        assert!(!DependencyType::DiscoveredFrom.affects_workflow());
        assert!(!DependencyType::RepliesTo.affects_workflow());
        assert!(!DependencyType::RelatesTo.affects_workflow());
        assert!(!DependencyType::Duplicates.affects_workflow());
        assert!(!DependencyType::Supersedes.affects_workflow());
        assert!(!DependencyType::AuthoredBy.affects_workflow());
        assert!(!DependencyType::AssignedTo.affects_workflow());
        assert!(!DependencyType::ApprovedBy.affects_workflow());
    }

    #[test]
    fn test_dependency_type_default() {
        assert_eq!(DependencyType::default(), DependencyType::Blocks);
    }

    #[test]
    fn test_dependency_type_is_valid() {
        assert!(DependencyType::Blocks.is_valid());
        assert!(DependencyType::ParentChild.is_valid());
        assert!(DependencyType::ConditionalBlocks.is_valid());
        assert!(DependencyType::WaitsFor.is_valid());
        assert!(DependencyType::Related.is_valid());
        assert!(DependencyType::DiscoveredFrom.is_valid());
        assert!(DependencyType::RepliesTo.is_valid());
        assert!(DependencyType::RelatesTo.is_valid());
        assert!(DependencyType::Duplicates.is_valid());
        assert!(DependencyType::Supersedes.is_valid());
        assert!(DependencyType::AuthoredBy.is_valid());
        assert!(DependencyType::AssignedTo.is_valid());
        assert!(DependencyType::ApprovedBy.is_valid());
    }

    // EventType enum tests
    #[test]
    fn test_event_type_serialization() {
        let event_type = EventType::StatusChanged;
        let serialized = serde_json::to_string(&event_type).unwrap();
        assert_eq!(serialized, "\"status\"");
    }

    #[test]
    fn test_event_type_deserialization() {
        let json = "\"status\"";
        let event_type: EventType = serde_json::from_str(json).unwrap();
        assert_eq!(event_type, EventType::StatusChanged);
    }

    #[test]
    fn test_event_type_default() {
        assert_eq!(EventType::default(), EventType::Created);
    }

    #[test]
    fn test_event_type_is_valid() {
        assert!(EventType::Created.is_valid());
        assert!(EventType::Updated.is_valid());
        assert!(EventType::StatusChanged.is_valid());
        assert!(EventType::Commented.is_valid());
        assert!(EventType::Closed.is_valid());
        assert!(EventType::Reopened.is_valid());
        assert!(EventType::DependencyAdded.is_valid());
        assert!(EventType::DependencyRemoved.is_valid());
        assert!(EventType::LabelAdded.is_valid());
        assert!(EventType::LabelRemoved.is_valid());
        assert!(EventType::Compacted.is_valid());
    }

    // Integration tests for complete round-trip
    #[test]
    fn test_complete_issue_serialization_roundtrip() {
        let issue = Issue {
            id: "test-123".to_string(),
            title: "Test Issue".to_string(),
            status: Status::InProgress,
            priority: Some(2),
            issue_type: IssueType::Feature,
            created_at: time::macros::datetime!(2023-01-01 00:00:00 UTC),
            updated_at: time::macros::datetime!(2023-01-02 00:00:00 UTC),
            closed_at: None,
            assignee: Some("test-user".to_string()),
            labels: Some(vec!["urgent".to_string(), "backend".to_string()]),
            description: Some("Test description".to_string()),
            acceptance_criteria: Some("Test criteria".to_string()),
            close_reason: None,
            estimate: Some(8),
            dependencies: vec![],
        };

        let serialized = serde_json::to_string(&issue).unwrap();
        let deserialized: Issue = serde_json::from_str(&serialized).unwrap();

        assert_eq!(issue.id, deserialized.id);
        assert_eq!(issue.status, deserialized.status);
        assert_eq!(issue.issue_type, deserialized.issue_type);
    }

    #[test]
    fn test_activity_serialization_roundtrip() {
        let activity = Activity {
            timestamp: time::macros::datetime!(2023-01-01 12:00:00 UTC),
            r#type: EventType::StatusChanged,
            issue_id: "test-123".to_string(),
            message: "Status changed from Open to In Progress".to_string(),
            old_status: Some(Status::Open),
            new_status: Some(Status::InProgress),
        };

        let serialized = serde_json::to_string(&activity).unwrap();
        let deserialized: Activity = serde_json::from_str(&serialized).unwrap();

        assert_eq!(activity.r#type, deserialized.r#type);
        assert_eq!(activity.old_status, deserialized.old_status);
        assert_eq!(activity.new_status, deserialized.new_status);
    }

    #[test]
    fn test_activity_parse_real_json() {
        // Test parsing actual JSON from bd activity --json
        let json = r#"{"timestamp":"2026-01-01T11:00:56.004281+01:00","type":"status","issue_id":"nacre-eue.1","symbol":"→","message":"nacre-eue.1 started","old_status":"open","new_status":"in_progress"}"#;

        let activity: Activity = serde_json::from_str(json).expect("Failed to parse activity JSON");
        assert_eq!(activity.issue_id, "nacre-eue.1");
        assert_eq!(activity.r#type, EventType::StatusChanged);
        assert_eq!(activity.old_status, Some(Status::Open));
        assert_eq!(activity.new_status, Some(Status::InProgress));
    }

    #[test]
    fn test_activity_parse_array() {
        // Test parsing array of activities as returned by bd activity --json
        let json = r#"[
            {"timestamp":"2026-01-01T10:35:34.504723+01:00","type":"create","issue_id":"nacre-eue","symbol":"+","message":"nacre-eue created"},
            {"timestamp":"2026-01-01T11:00:56.004281+01:00","type":"status","issue_id":"nacre-eue.1","symbol":"→","message":"started","old_status":"open","new_status":"in_progress"},
            {"timestamp":"2026-01-01T11:07:46.028596+01:00","type":"status","issue_id":"nacre-eue.1","symbol":"✓","message":"completed","old_status":"in_progress","new_status":"closed"}
        ]"#;

        let activities: Vec<Activity> =
            serde_json::from_str(json).expect("Failed to parse activities array");
        assert_eq!(activities.len(), 3);

        // Find the InProgress activity
        let in_progress_activities: Vec<_> = activities
            .iter()
            .filter(|a| a.new_status == Some(Status::InProgress))
            .collect();
        assert_eq!(in_progress_activities.len(), 1);
        assert_eq!(in_progress_activities[0].issue_id, "nacre-eue.1");
    }

    #[test]
    fn test_dependency_serialization_roundtrip() {
        let dependency = Dependency {
            issue_id: "child-123".to_string(),
            depends_on_id: "parent-456".to_string(),
            dep_type: DependencyType::Blocks,
            created_at: Some(time::macros::datetime!(2023-01-01 12:00:00 UTC)),
            created_by: Some("test-user".to_string()),
        };

        let serialized = serde_json::to_string(&dependency).unwrap();
        let deserialized: Dependency = serde_json::from_str(&serialized).unwrap();

        assert_eq!(dependency.dep_type, deserialized.dep_type);
        assert_eq!(dependency.created_by, deserialized.created_by);
        assert_eq!(dependency.created_at, deserialized.created_at);
    }
}
