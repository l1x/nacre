use askama::Template;

use crate::beads;

pub mod filters {
    pub fn format_hours(hours: &f64) -> askama::Result<String> {
        Ok(format!("{:.1}h", hours))
    }
    pub fn format_minutes(mins: &f64) -> askama::Result<String> {
        Ok(format!("{:.0}m", mins))
    }
    pub fn format_decimal(val: &f64) -> askama::Result<String> {
        Ok(format!("{:.2}", val))
    }
    pub fn format_date(date: &chrono::DateTime<chrono::FixedOffset>) -> askama::Result<String> {
        Ok(date.format("%Y-%m-%d %H:%M").to_string())
    }
}

pub struct ProjectStats {
    pub total: usize,
    pub open: usize,
    pub in_progress: usize,
    pub blocked: usize,
    pub closed: usize,
    pub avg_lead_time_hours: f64,
    pub avg_cycle_time_mins: f64,
}

pub struct EpicWithProgress {
    pub issue: beads::Issue,
    pub total: usize,
    pub closed: usize,
    pub percent: f64,
    pub children: Vec<beads::Issue>,
}

impl EpicWithProgress {
    /// Create an EpicWithProgress from an epic issue and all issues.
    /// If `include_children` is true, the children vector is populated (sorted by status).
    pub fn from_epic(
        epic: &beads::Issue,
        all_issues: &[beads::Issue],
        include_children: bool,
    ) -> Self {
        let prefix = format!("{}.", epic.id);
        let mut children: Vec<beads::Issue> = all_issues
            .iter()
            .filter(|i| {
                i.dependencies.iter().any(|d| d.depends_on_id == epic.id)
                    || i.id.starts_with(&prefix)
            })
            .cloned()
            .collect();

        // Sort children by status priority
        children.sort_by_key(|i| i.status.sort_order());

        let total = children.len();
        let closed = children
            .iter()
            .filter(|i| i.status == beads::Status::Closed)
            .count();
        let percent = if total > 0 {
            (closed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Self {
            issue: epic.clone(),
            total,
            closed,
            percent,
            children: if include_children {
                children
            } else {
                Vec::new()
            },
        }
    }
}

pub struct BoardColumn {
    pub name: String,
    pub status: String,
    pub issues: Vec<beads::Issue>,
}

/// Tree node for hierarchical graph view
pub struct TreeNode {
    pub id: String,
    pub title: String,
    pub status: String,
    pub issue_type: String,
    pub priority: u8,
    pub blocked_by_count: usize,
    pub has_children: bool,
    pub depth: usize,
    pub parent_id: Option<String>,
}

#[derive(Template)]
#[template(path = "landing.html")]
pub struct LandingTemplate {
    pub project_name: String,
    pub page_title: String,
    pub active_nav: &'static str,
    pub app_version: String,
    pub stats: ProjectStats,
    pub epics: Vec<EpicWithProgress>,
    pub blocked: Vec<beads::Issue>,
    pub in_progress: Vec<beads::Issue>,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub project_name: String,
    pub page_title: String,
    pub active_nav: &'static str,
    pub app_version: String,
    pub nodes: Vec<TreeNode>,
}

#[derive(Template)]
#[template(path = "epics.html")]
pub struct EpicsTemplate {
    pub project_name: String,
    pub page_title: String,
    pub active_nav: &'static str,
    pub app_version: String,
    pub epics: Vec<EpicWithProgress>,
}

#[derive(Template)]
#[template(path = "board.html")]
pub struct BoardTemplate {
    pub project_name: String,
    pub page_title: String,
    pub active_nav: &'static str,
    pub app_version: String,
    pub columns: Vec<BoardColumn>,
}

#[derive(Template)]
#[template(path = "issue.html")]
pub struct IssueDetailTemplate {
    pub project_name: String,
    pub page_title: String,
    pub active_nav: &'static str,
    pub app_version: String,
    pub issue: beads::Issue,
}

#[derive(Template)]
#[template(path = "new_issue.html")]
pub struct NewIssueTemplate {
    pub project_name: String,
    pub page_title: String,
    pub active_nav: &'static str,
    pub app_version: String,
}

#[derive(Template)]
#[template(path = "prds.html")]
pub struct PrdsListTemplate {
    pub project_name: String,
    pub page_title: String,
    pub active_nav: &'static str,
    pub app_version: String,
    pub files: Vec<String>,
}

#[derive(Template)]
#[template(path = "prd.html")]
pub struct PrdViewTemplate {
    pub project_name: String,
    pub page_title: String,
    pub active_nav: &'static str,
    pub app_version: String,
    #[allow(dead_code)]
    pub filename: String,
    pub content: String,
}

#[derive(Template)]
#[template(path = "epic_detail.html")]
pub struct EpicDetailTemplate {
    pub project_name: String,
    pub page_title: String,
    pub active_nav: &'static str,
    pub app_version: String,
    pub epic: EpicWithProgress,
}

#[derive(Template)]
#[template(path = "edit_issue.html")]
pub struct EditIssueTemplate {
    pub project_name: String,
    pub page_title: String,
    pub active_nav: &'static str,
    pub app_version: String,
    pub issue: beads::Issue,
}

#[derive(Template)]
#[template(path = "graph.html")]
pub struct GraphTemplate {
    pub project_name: String,
    pub page_title: String,
    pub active_nav: &'static str,
    pub app_version: String,
}

#[derive(Template)]
#[template(path = "metrics.html")]
pub struct MetricsTemplate {
    pub project_name: String,
    pub page_title: String,
    pub active_nav: &'static str,
    pub app_version: String,
    pub avg_lead_time_hours: f64,
    pub avg_cycle_time_mins: f64,
    pub throughput_per_day: f64,
    pub closed_last_7_days: usize,
    pub wip_count: usize,
    pub blocked_count: usize,
    pub tickets_chart_svg: String,
    pub lead_time_chart_svg: String,
    pub p50_lead_time_hours: f64,
    pub p90_lead_time_hours: f64,
    pub p100_lead_time_hours: f64,
}
