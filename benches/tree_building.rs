use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::collections::{HashMap, HashSet};

/// Simulated issue for benchmarking
#[derive(Clone)]
struct Issue {
    id: String,
    status: u8,
}

/// Generate test issues with hierarchical IDs (e.g., nacre-1, nacre-1.1, nacre-1.1.1)
fn generate_issues(count: usize) -> Vec<Issue> {
    let mut issues = Vec::with_capacity(count);
    let mut id_counter = 0;

    // Create ~20% as root issues, rest as children
    let root_count = count / 5;

    for i in 0..root_count {
        issues.push(Issue {
            id: format!("nacre-{}", i),
            status: (i % 4) as u8,
        });
    }
    id_counter = root_count;

    // Add children with dot notation
    let mut depth_1_count = 0;
    while id_counter < count {
        let parent_idx = depth_1_count % root_count;
        let child_num = depth_1_count / root_count + 1;

        // First level children: nacre-X.Y
        issues.push(Issue {
            id: format!("nacre-{}.{}", parent_idx, child_num),
            status: (id_counter % 4) as u8,
        });
        id_counter += 1;
        depth_1_count += 1;

        // Add some second level children: nacre-X.Y.Z
        if id_counter < count && child_num <= 3 {
            issues.push(Issue {
                id: format!("nacre-{}.{}.1", parent_idx, child_num),
                status: (id_counter % 4) as u8,
            });
            id_counter += 1;
        }
    }

    issues
}

/// Current O(n²) implementation - scans all issues for each parent lookup
fn build_tree_current(issues: &[Issue]) -> Vec<(String, Option<String>, usize)> {
    let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut parent_map: HashMap<String, String> = HashMap::new();

    for issue in issues {
        // O(n) scan inside O(n) loop = O(n²)
        if let Some(dot_pos) = issue.id.rfind('.') {
            let potential_parent = &issue.id[..dot_pos];
            if issues.iter().any(|i| i.id == potential_parent) {
                children_map
                    .entry(potential_parent.to_string())
                    .or_default()
                    .push(issue.id.clone());
                parent_map.insert(issue.id.clone(), potential_parent.to_string());
            }
        }
    }

    // Build result with sorting at each level (current behavior)
    let issue_map: HashMap<&str, &Issue> = issues.iter().map(|i| (i.id.as_str(), i)).collect();
    let mut result = Vec::new();

    fn build_recursive(
        id: &str,
        issue_map: &HashMap<&str, &Issue>,
        children_map: &HashMap<String, Vec<String>>,
        parent_map: &HashMap<String, String>,
        depth: usize,
        result: &mut Vec<(String, Option<String>, usize)>,
    ) {
        let parent_id = parent_map.get(id).cloned();
        result.push((id.to_string(), parent_id, depth));

        if let Some(children) = children_map.get(id) {
            // Sort children at each recursive call (current behavior)
            let mut sorted: Vec<_> = children
                .iter()
                .filter_map(|c| issue_map.get(c.as_str()).map(|i| (c.as_str(), i.status)))
                .collect();
            sorted.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(b.0)));

            for (child_id, _) in sorted {
                build_recursive(
                    child_id,
                    issue_map,
                    children_map,
                    parent_map,
                    depth + 1,
                    result,
                );
            }
        }
    }

    // Find roots and sort them
    let mut roots: Vec<_> = issues
        .iter()
        .filter(|i| !parent_map.contains_key(&i.id))
        .collect();
    roots.sort_by(|a, b| a.status.cmp(&b.status).then_with(|| a.id.cmp(&b.id)));

    for root in roots {
        build_recursive(
            &root.id,
            &issue_map,
            &children_map,
            &parent_map,
            0,
            &mut result,
        );
    }

    result
}

/// Optimized O(n log n) implementation - HashSet for parent lookup, pre-sorted children
fn build_tree_optimized(issues: &[Issue]) -> Vec<(String, Option<String>, usize)> {
    // O(n) - Build ID set for O(1) parent lookups
    let id_set: HashSet<&str> = issues.iter().map(|i| i.id.as_str()).collect();

    let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut parent_map: HashMap<String, String> = HashMap::new();

    // O(n) - Single pass with O(1) lookups
    for issue in issues {
        if let Some(dot_pos) = issue.id.rfind('.') {
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

    // O(n) - Build issue map
    let issue_map: HashMap<&str, &Issue> = issues.iter().map(|i| (i.id.as_str(), i)).collect();

    // O(n log n) total - Pre-sort all children lists once
    for children in children_map.values_mut() {
        children.sort_by(|a, b| {
            let a_status = issue_map.get(a.as_str()).map(|i| i.status).unwrap_or(0);
            let b_status = issue_map.get(b.as_str()).map(|i| i.status).unwrap_or(0);
            a_status.cmp(&b_status).then_with(|| a.cmp(b))
        });
    }

    let mut result = Vec::new();

    // Simple recursive traversal - no sorting needed
    fn build_recursive(
        id: &str,
        children_map: &HashMap<String, Vec<String>>,
        parent_map: &HashMap<String, String>,
        depth: usize,
        result: &mut Vec<(String, Option<String>, usize)>,
    ) {
        let parent_id = parent_map.get(id).cloned();
        result.push((id.to_string(), parent_id, depth));

        if let Some(children) = children_map.get(id) {
            for child_id in children {
                build_recursive(child_id, children_map, parent_map, depth + 1, result);
            }
        }
    }

    // Find and sort roots
    let mut roots: Vec<_> = issues
        .iter()
        .filter(|i| !parent_map.contains_key(&i.id))
        .collect();
    roots.sort_by(|a, b| a.status.cmp(&b.status).then_with(|| a.id.cmp(&b.id)));

    for root in roots {
        build_recursive(&root.id, &children_map, &parent_map, 0, &mut result);
    }

    result
}

fn bench_tree_building(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_building");

    for size in [64, 128, 256, 512, 1024] {
        let issues = generate_issues(size);

        group.bench_with_input(
            BenchmarkId::new("current_O(n²)", size),
            &issues,
            |b, issues| b.iter(|| build_tree_current(black_box(issues))),
        );

        group.bench_with_input(
            BenchmarkId::new("optimized_O(n_log_n)", size),
            &issues,
            |b, issues| b.iter(|| build_tree_optimized(black_box(issues))),
        );
    }

    group.finish();
}

fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling_comparison");

    // Focus on 512 items as requested
    let issues_512 = generate_issues(512);

    group.bench_function("512_items_current", |b| {
        b.iter(|| build_tree_current(black_box(&issues_512)))
    });

    group.bench_function("512_items_optimized", |b| {
        b.iter(|| build_tree_optimized(black_box(&issues_512)))
    });

    group.finish();
}

criterion_group!(benches, bench_tree_building, bench_scaling);
criterion_main!(benches);
