use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header, HeaderMap},
    response::IntoResponse,
};
use chrono::Utc;
use std::collections::{HashMap, HashSet};

use crate::beads;
use crate::templates::{EditIssueTemplate, EpicWithProgress, NewIssueTemplate, TaskDetailTemplate, TasksTemplate, TreeNode};

pub async fn tasks_list(State(state): State<crate::SharedAppState>) -> crate::AppResult<TasksTemplate> {
    let all_issues = state.client.list_issues()?;
    let nodes = build_issue_tree(&all_issues);

    Ok(TasksTemplate {
        project_name: state.project_name.clone(),
        page_title: "Tasks".to_string(),
        active_nav: "tasks",
        app_version: state.app_version.clone(),
        nodes,
    })
}

pub async fn task_detail(
    State(state): State<crate::SharedAppState>,
    Path(id): Path<String>,
) -> crate::AppResult<TaskDetailTemplate> {
    let all_issues = state.client.list_issues()?;

    // Find the issue (any type, not just epics)
    let issue = all_issues
        .iter()
        .find(|i| i.id == id)
        .ok_or_else(|| crate::AppError::NotFound(format!("Task {}", id)))?;

    // Build tree for just this task and its descendants
    let prefix = format!("{}.", id);
    let descendants: Vec<beads::Issue> = all_issues
        .iter()
        .filter(|i| {
            i.id == id || // Include self to root the tree
            i.dependencies.iter().any(|d| d.depends_on_id == id) || // Direct dependencies
            i.id.starts_with(&prefix) // Dot-notation descendants
        })
        .cloned()
        .collect();

    // build_issue_tree returns a list starting with roots.
    // Since we included 'id', it should be the first root.
    // We want to skip it and take the rest (which are its children/descendants).
    let mut tree_nodes = build_issue_tree(&descendants);
    
    // Remove the root node (the task itself) if present
    if !tree_nodes.is_empty() && tree_nodes[0].id == id {
        tree_nodes.remove(0);
    }
    
    // Adjust depths and parents for the detail view context
    for node in &mut tree_nodes {
        if node.depth > 0 {
            node.depth -= 1;
        }
        
        // If the parent is the current task, treat it as a root in this view
        if node.parent_id.as_deref() == Some(&id) {
            node.parent_id = None;
        }
    }

    let can_expand = tree_nodes.iter().any(|n| n.has_children);

    Ok(TaskDetailTemplate {
        project_name: state.project_name.clone(),
        page_title: id.clone(),
        active_nav: "tasks-detail",
        app_version: state.app_version.clone(),
        task: EpicWithProgress::from_epic(issue, &all_issues, false),
        children_tree: tree_nodes,
        can_expand,
    })
}

/// Build a hierarchical tree of issues for display
fn build_issue_tree(all_issues: &[beads::Issue]) -> Vec<TreeNode> {
    // Build ID set for O(1) parent lookups (optimization from O(n²) to O(n))
    let id_set: HashSet<&str> = all_issues.iter().map(|i| i.id.as_str()).collect();

    // Build parent-child relationships
    let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut parent_map: HashMap<String, String> = HashMap::new();

    for issue in all_issues {
        // Check explicit parent-child dependency
        for dep in &issue.dependencies {
            if dep.dep_type == beads::DependencyType::ParentChild {
                children_map
                    .entry(dep.depends_on_id.clone())
                    .or_default()
                    .push(issue.id.clone());
                parent_map.insert(issue.id.clone(), dep.depends_on_id.clone());
            }
        }

        // Check dot-notation for implicit parent-child (e.g., nacre-3hd.1 -> nacre-3hd)
        if !parent_map.contains_key(&issue.id)
            && let Some(dot_pos) = issue.id.rfind('.')
        {
            let potential_parent = &issue.id[..dot_pos];
            if id_set.contains(potential_parent) {
                children_map
                    .entry(potential_parent.to_string())
                    .or_default()
                    .push(issue.id.clone());
                parent_map.insert(issue.id.clone(), potential_parent.to_string());
            }
        }
    }

    // Count blocking dependencies (non-parent-child) for each issue
    let mut blocked_by_count: HashMap<String, usize> = HashMap::new();
    for issue in all_issues {
        let count = issue
            .dependencies
            .iter()
            .filter(|d| d.dep_type != beads::DependencyType::ParentChild)
            .count();
        blocked_by_count.insert(issue.id.clone(), count);
    }

    // Build issue lookup
    let issue_map: HashMap<String, &beads::Issue> =
        all_issues.iter().map(|i| (i.id.clone(), i)).collect();

    // Recursive function to build tree nodes
    fn build_tree(
        issue_id: &str,
        issue_map: &HashMap<String, &beads::Issue>,
        children_map: &HashMap<String, Vec<String>>,
        blocked_by_count: &HashMap<String, usize>,
        depth: usize,
        nodes: &mut Vec<TreeNode>,
    ) {
        let Some(issue) = issue_map.get(issue_id) else {
            return;
        };

        let children_ids = children_map.get(issue_id);
        let has_children = children_ids.map(|c| !c.is_empty()).unwrap_or(false);

        // Determine parent_id for this node
        let parent_id = if depth > 0 {
            issue
                .id
                .rfind('.')
                .map(|dot_pos| issue.id[..dot_pos].to_string())
        } else {
            None
        };

        nodes.push(TreeNode {
            id: issue.id.clone(),
            title: issue.title.clone(),
            status: issue.status.as_str().to_string(),
            issue_type: issue.issue_type.as_css_class().to_string(),
            priority: issue.priority.unwrap_or(2),
            blocked_by_count: blocked_by_count.get(&issue.id).copied().unwrap_or(0),
            has_children,
            depth,
            parent_id,
        });

        // Recursively add children
        if let Some(children) = children_ids {
            let mut sorted_children: Vec<_> = children
                .iter()
                .filter_map(|id| issue_map.get(id).map(|i| (id, *i)))
                .collect();
            sorted_children.sort_by(|a, b| {
                a.1.status
                    .sort_order()
                    .cmp(&b.1.status.sort_order())
                    .then_with(|| a.0.cmp(b.0))
            });

            for (child_id, _) in sorted_children {
                build_tree(
                    child_id,
                    issue_map,
                    children_map,
                    blocked_by_count,
                    depth + 1,
                    nodes,
                );
            }
        }
    }

    // Find top-level nodes (no parent)
    let mut top_level: Vec<&beads::Issue> = all_issues
        .iter()
        .filter(|i| !parent_map.contains_key(&i.id))
        .collect();

    // Sort top-level: epics first, then by status, then by id
    top_level.sort_by(|a, b| {
        let a_is_epic = a.issue_type == beads::IssueType::Epic;
        let b_is_epic = b.issue_type == beads::IssueType::Epic;
        b_is_epic
            .cmp(&a_is_epic)
            .then_with(|| a.status.sort_order().cmp(&b.status.sort_order()))
            .then_with(|| a.id.cmp(&b.id))
    });

    // Build flat tree
    let mut nodes = Vec::new();
    for issue in top_level {
        build_tree(
            &issue.id,
            &issue_map,
            &children_map,
            &blocked_by_count,
            0,
            &mut nodes,
        );
    }

    nodes
}

// Form handlers

pub async fn edit_task(
    State(state): State<crate::SharedAppState>,
    Path(id): Path<String>,
) -> crate::AppResult<EditIssueTemplate> {
    let issue = state.client.get_issue(&id)?;
    Ok(EditIssueTemplate {
        project_name: state.project_name.clone(),
        page_title: format!("Edit {}", id),
        active_nav: "tasks-edit",
        app_version: state.app_version.clone(),
        issue,
    })
}

pub async fn new_task_form(State(state): State<crate::SharedAppState>) -> NewIssueTemplate {
    NewIssueTemplate {
        project_name: state.project_name.clone(),
        page_title: "New Task".to_string(),
        active_nav: "tasks-new",
        app_version: state.app_version.clone(),
    }
}

// API handlers

pub async fn list_tasks(
    State(state): State<crate::SharedAppState>,
    headers: HeaderMap,
) -> crate::AppResult<impl IntoResponse> {
    let issues = state.client.list_issues()?;

    let max_updated_at = issues.iter().map(|i| i.updated_at).max();

    let etag = if let Some(last_mod) = max_updated_at {
        format!("\"{:x}-{}\"", last_mod.timestamp(), issues.len())
    } else {
        format!("\"{}\"", issues.len())
    };

    if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH) {
        if if_none_match == etag.as_str() {
            return Ok(StatusCode::NOT_MODIFIED.into_response());
        }
    }

    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::CACHE_CONTROL, "no-cache".parse().unwrap());
    response_headers.insert(header::ETAG, etag.parse().unwrap());

    if let Some(last_mod) = max_updated_at {
        let last_mod_str = last_mod
            .with_timezone(&Utc)
            .format("%a, %d %b %Y %H:%M:%S GMT")
            .to_string();
        response_headers.insert(header::LAST_MODIFIED, last_mod_str.parse().unwrap());
    }

    Ok((response_headers, Json(issues)).into_response())
}

pub async fn update_task(
    State(state): State<crate::SharedAppState>,
    Path(id): Path<String>,
    Json(update): Json<beads::IssueUpdate>,
) -> crate::AppResult<StatusCode> {
    state.client.update_issue(&id, update)?;
    Ok(StatusCode::OK)
}

pub async fn create_task(
    State(state): State<crate::SharedAppState>,
    Json(create): Json<beads::IssueCreate>,
) -> crate::AppResult<Json<serde_json::Value>> {
    let id = state.client.create_issue(create)?;
    Ok(Json(serde_json::json!({ "id": id })))
}
