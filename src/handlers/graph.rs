use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use std::collections::HashSet;

use crate::beads::{Dependency, DependencyType, Issue, Status};

/// A node in the dependency graph representing an issue
#[derive(Debug, Serialize)]
pub struct GraphNode {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub issue_type: String,
    pub status: String,
    pub priority: u8,
    /// Parent ID for hierarchical positioning (dot-notation or explicit parent-child)
    pub parent: Option<String>,
}

/// An edge in the dependency graph representing a relationship between issues
#[derive(Debug, Serialize)]
pub struct GraphEdge {
    /// Source node ID (the dependent/child issue)
    pub from: String,
    /// Target node ID (the blocking/parent issue)
    pub to: String,
    /// Type of relationship
    #[serde(rename = "type")]
    pub edge_type: String,
}

/// Complete graph data for visualization
#[derive(Debug, Serialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

impl GraphNode {
    fn from_issue(issue: &Issue, parent_id: Option<String>) -> Self {
        Self {
            id: issue.id.clone(),
            title: issue.title.clone(),
            issue_type: issue.issue_type.as_css_class().to_string(),
            status: issue.status.as_str().to_string(),
            priority: issue.priority.unwrap_or(2),
            parent: parent_id,
        }
    }
}

/// Build graph data from a list of issues and their dependencies
fn build_graph_data(issues: &[Issue], all_dependencies: &[Dependency]) -> GraphData {
    // Build ID set for efficient parent lookups
    let id_set: HashSet<&str> = issues.iter().map(|i| i.id.as_str()).collect();

    let mut nodes = Vec::with_capacity(issues.len());
    let mut edges = Vec::new();
    let mut seen_edges: HashSet<(String, String, String)> = HashSet::new();

    // Build a map of issue_id -> dependencies for quick lookup
    let mut deps_by_issue: std::collections::HashMap<&str, Vec<&Dependency>> =
        std::collections::HashMap::new();
    for dep in all_dependencies {
        deps_by_issue
            .entry(dep.issue_id.as_str())
            .or_default()
            .push(dep);
    }

    for issue in issues {
        // Determine parent from dot-notation first
        let mut parent_id: Option<String> = None;

        if let Some(dot_pos) = issue.id.rfind('.') {
            let potential_parent = &issue.id[..dot_pos];
            if id_set.contains(potential_parent) {
                parent_id = Some(potential_parent.to_string());

                // Add implicit parent-child edge
                let edge_key = (
                    issue.id.clone(),
                    potential_parent.to_string(),
                    "parent-child".to_string(),
                );
                if !seen_edges.contains(&edge_key) {
                    edges.push(GraphEdge {
                        from: issue.id.clone(),
                        to: potential_parent.to_string(),
                        edge_type: "parent-child".to_string(),
                    });
                    seen_edges.insert(edge_key);
                }
            }
        }

        // Process explicit dependencies from the all_dependencies list
        if let Some(issue_deps) = deps_by_issue.get(issue.id.as_str()) {
            for dep in issue_deps {
                // Skip if target doesn't exist in our issue set
                if !id_set.contains(dep.depends_on_id.as_str()) {
                    continue;
                }

                let edge_type = dep.dep_type.as_str().to_string();

                // If this is a parent-child dependency, use it for hierarchy
                if dep.dep_type == DependencyType::ParentChild && parent_id.is_none() {
                    parent_id = Some(dep.depends_on_id.clone());
                }

                // Add edge (deduplicated)
                let edge_key = (
                    issue.id.clone(),
                    dep.depends_on_id.clone(),
                    edge_type.clone(),
                );
                if !seen_edges.contains(&edge_key) {
                    edges.push(GraphEdge {
                        from: issue.id.clone(),
                        to: dep.depends_on_id.clone(),
                        edge_type,
                    });
                    seen_edges.insert(edge_key);
                }
            }
        }

        nodes.push(GraphNode::from_issue(issue, parent_id));
    }

    GraphData { nodes, edges }
}

/// API handler for graph data
///
/// Returns JSON with nodes and edges for dependency graph visualization.
/// By default returns all non-tombstone issues.
pub async fn graph_data(
    State(state): State<crate::SharedAppState>,
) -> crate::AppResult<impl IntoResponse> {
    let all_issues = state.client.list_issues()?;
    let all_dependencies = state.client.list_all_dependencies().unwrap_or_default();

    // Filter out tombstone issues
    let active_issues: Vec<Issue> = all_issues
        .into_iter()
        .filter(|i| i.status != Status::Tombstone)
        .collect();

    let graph = build_graph_data(&active_issues, &all_dependencies);

    Ok((StatusCode::OK, Json(graph)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beads::IssueType;
    use time::OffsetDateTime;

    fn make_issue(id: &str, issue_type: IssueType, status: Status) -> Issue {
        Issue {
            id: id.to_string(),
            title: format!("Test {}", id),
            status,
            priority: Some(2),
            issue_type,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            closed_at: None,
            assignee: None,
            labels: None,
            description: None,
            acceptance_criteria: None,
            close_reason: None,
            estimate: None,
            dependencies: vec![],
        }
    }

    fn make_dependency(from: &str, to: &str, dep_type: DependencyType) -> Dependency {
        Dependency {
            issue_id: from.to_string(),
            depends_on_id: to.to_string(),
            dep_type,
            created_at: None,
            created_by: None,
        }
    }

    #[test]
    fn test_build_graph_basic() {
        let issues = vec![
            make_issue("nacre-1", IssueType::Epic, Status::Open),
            make_issue("nacre-2", IssueType::Task, Status::InProgress),
        ];

        let graph = build_graph_data(&issues, &[]);

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 0);
    }

    #[test]
    fn test_dot_notation_creates_edge() {
        let issues = vec![
            make_issue("nacre-1", IssueType::Epic, Status::Open),
            make_issue("nacre-1.1", IssueType::Task, Status::Open),
            make_issue("nacre-1.2", IssueType::Task, Status::InProgress),
        ];

        let graph = build_graph_data(&issues, &[]);

        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);

        // Check parent-child edges
        let parent_child_edges: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| e.edge_type == "parent-child")
            .collect();
        assert_eq!(parent_child_edges.len(), 2);

        // Verify the child nodes have parent set
        let child1 = graph.nodes.iter().find(|n| n.id == "nacre-1.1").unwrap();
        assert_eq!(child1.parent, Some("nacre-1".to_string()));
    }

    #[test]
    fn test_nested_dot_notation() {
        let issues = vec![
            make_issue("nacre-1", IssueType::Epic, Status::Open),
            make_issue("nacre-1.1", IssueType::Task, Status::Open),
            make_issue("nacre-1.1.1", IssueType::Task, Status::Open),
        ];

        let graph = build_graph_data(&issues, &[]);

        assert_eq!(graph.nodes.len(), 3);

        // Check hierarchy
        let grandchild = graph.nodes.iter().find(|n| n.id == "nacre-1.1.1").unwrap();
        assert_eq!(grandchild.parent, Some("nacre-1.1".to_string()));
    }

    #[test]
    fn test_orphan_dot_notation() {
        // If parent doesn't exist, no edge should be created
        let issues = vec![make_issue("nacre-1.1", IssueType::Task, Status::Open)];

        let graph = build_graph_data(&issues, &[]);

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.edges.len(), 0);
        assert_eq!(graph.nodes[0].parent, None);
    }

    #[test]
    fn test_explicit_blocking_dependency() {
        let issues = vec![
            make_issue("nacre-1", IssueType::Task, Status::Open),
            make_issue("nacre-2", IssueType::Task, Status::Open),
        ];
        let deps = vec![make_dependency("nacre-2", "nacre-1", DependencyType::Blocks)];

        let graph = build_graph_data(&issues, &deps);

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].from, "nacre-2");
        assert_eq!(graph.edges[0].to, "nacre-1");
        assert_eq!(graph.edges[0].edge_type, "blocks");
    }

    #[test]
    fn test_combined_dot_notation_and_explicit_deps() {
        let issues = vec![
            make_issue("nacre-1", IssueType::Epic, Status::Open),
            make_issue("nacre-1.1", IssueType::Task, Status::Open),
            make_issue("nacre-1.2", IssueType::Task, Status::Open),
        ];
        // nacre-1.2 blocks nacre-1.1 (in addition to both being children of nacre-1)
        let deps = vec![make_dependency(
            "nacre-1.2",
            "nacre-1.1",
            DependencyType::Blocks,
        )];

        let graph = build_graph_data(&issues, &deps);

        assert_eq!(graph.nodes.len(), 3);
        // 2 parent-child edges + 1 blocks edge
        assert_eq!(graph.edges.len(), 3);

        let blocks_edges: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| e.edge_type == "blocks")
            .collect();
        assert_eq!(blocks_edges.len(), 1);
    }
}
