use criterion::{Criterion, black_box, criterion_group, criterion_main};

// Simple template performance benchmark comparing Askama vs Minijinja
// This focuses on core rendering performance with realistic data

use serde_json;

// Test data structure
#[derive(serde::Serialize, Clone)]
struct TestIssue {
    id: String,
    title: String,
    status: String,
    issue_type: String,
    priority: u8,
}

fn create_test_issues(count: usize) -> Vec<TestIssue> {
    let mut issues = Vec::with_capacity(count);

    for i in 0..count {
        let status = match i % 6 {
            0 => "open",
            1 => "in-progress",
            2 => "blocked",
            3 => "deferred",
            4 => "closed",
            _ => "pinned",
        };

        let issue_type = match i % 9 {
            0 => "task",
            1 => "bug",
            2 => "feature",
            3 => "epic",
            4 => "chore",
            5 => "message",
            6 => "merge-request",
            7 => "molecule",
            _ => "gate",
        };

        issues.push(TestIssue {
            id: format!("nacre-{:03x}", i),
            title: format!("Test Issue {}", i),
            status: status.to_string(),
            issue_type: issue_type.to_string(),
            priority: (i % 4) as u8 + 1,
        });
    }

    issues
}

// Askama template tests
mod askama_tests {
    use super::*;
    use askama::Template;

    #[derive(Template)]
    #[template(
        source = r#"
<!DOCTYPE html>
<html>
<head>
    <title>{{ project_name }}</title>
</head>
<body>
    <h1>{{ project_name }}</h1>
    <ul>
        {% for issue in issues %}
        <li class="issue {{ issue.issue_type }}">
            <span class="id">{{ issue.id }}</span>
            <span class="title">{{ issue.title }}</span>
            <span class="status">{{ issue.status }}</span>
        </li>
        {% endfor %}
    </ul>
</body>
</html>
    "#,
        ext = "html"
    )]
    struct ListTemplate {
        project_name: String,
        issues: Vec<TestIssue>,
    }

    pub fn bench_simple_list(c: &mut Criterion) {
        let issues = create_test_issues(100);
        let template = ListTemplate {
            project_name: "Test Project".to_string(),
            issues,
        };

        c.bench_function("askama_simple_list_100", |b| {
            b.iter(|| template.render().unwrap())
        });
    }

    pub fn bench_complex_list(c: &mut Criterion) {
        let issues = create_test_issues(500);
        let template = ListTemplate {
            project_name: "Test Project".to_string(),
            issues,
        };

        c.bench_function("askama_complex_list_500", |b| {
            b.iter(|| template.render().unwrap())
        });
    }
}

// Minijinja template tests
mod minijinja_tests {
    use super::*;

    pub fn bench_simple_list(c: &mut Criterion) {
        let mut env = minijinja::Environment::new();

        let template_str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>{{ project_name }}</title>
</head>
<body>
    <h1>{{ project_name }}</h1>
    <ul>
        {% for issue in issues %}
        <li class="issue {{ issue.issue_type }}">
            <span class="id">{{ issue.id }}</span>
            <span class="title">{{ issue.title }}</span>
            <span class="status">{{ issue.status }}</span>
        </li>
        {% endfor %}
    </ul>
</body>
</html>
        "#;

        env.add_template("list", template_str).unwrap();
        let template = env.get_template("list").unwrap();

        let issues = create_test_issues(100);
        let ctx = serde_json::json!({
            "project_name": "Test Project",
            "issues": issues
        });

        c.bench_function("minijinja_simple_list_100", |b| {
            b.iter(|| template.render(black_box(&ctx)).unwrap())
        });
    }

    pub fn bench_complex_list(c: &mut Criterion) {
        let mut env = minijinja::Environment::new();

        let template_str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>{{ project_name }}</title>
</head>
<body>
    <h1>{{ project_name }}</h1>
    <ul>
        {% for issue in issues %}
        <li class="issue {{ issue.issue_type }}">
            <span class="id">{{ issue.id }}</span>
            <span class="title">{{ issue.title }}</span>
            <span class="status">{{ issue.status }}</span>
        </li>
        {% endfor %}
    </ul>
</body>
</html>
        "#;

        env.add_template("list", template_str).unwrap();
        let template = env.get_template("list").unwrap();

        let issues = create_test_issues(500);
        let ctx = serde_json::json!({
            "project_name": "Test Project",
            "issues": issues
        });

        c.bench_function("minijinja_complex_list_500", |b| {
            b.iter(|| template.render(black_box(&ctx)).unwrap())
        });
    }
}

// Template compilation overhead test
fn bench_template_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("template_compilation");

    // Minijinja compilation (runtime)
    group.bench_function("minijinja_compile", |b| {
        b.iter(|| {
            let mut env = minijinja::Environment::new();
            env.add_template("test", "<h1>{{ title }}</h1><p>{{ content }}</p>")
                .unwrap();
        })
    });

    // Askama template creation (no compilation needed at runtime)
    group.bench_function("askama_create", |b| {
        b.iter(|| {
            #[derive(askama::Template)]
            #[template(source = "<h1>{{ title }}</h1><p>{{ content }}</p>", ext = "html")]
            struct SimpleTemplate {
                title: String,
                content: String,
            }

            let _template = SimpleTemplate {
                title: "Test".to_string(),
                content: "Content".to_string(),
            };
        })
    });

    group.finish();
}

// Scalability test with different data sizes
fn bench_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");

    for &size in &[10, 50, 100, 500, 1000] {
        let issues = create_test_issues(size);

        // Askama
        {
            use askama::Template;

            #[derive(Template)]
            #[template(
                source = r#"
<ul>
{% for issue in issues %}
<li>{{ issue.id }}: {{ issue.title }}</li>
{% endfor %}
</ul>
            "#,
                ext = "html"
            )]
            struct SimpleList {
                issues: Vec<TestIssue>,
            }

            let template = SimpleList {
                issues: issues.clone(),
            };

            group.bench_with_input(format!("askama_{}_items", size), &size, |b, _| {
                b.iter(|| template.render().unwrap())
            });
        }

        // Minijinja
        {
            let mut env = minijinja::Environment::new();
            env.add_template("simple", "<ul>{% for issue in issues %}<li>{{ issue.id }}: {{ issue.title }}</li>{% endfor %}</ul>").unwrap();
            let template = env.get_template("simple").unwrap();

            let ctx = serde_json::json!({
                "issues": issues
            });

            group.bench_with_input(format!("minijinja_{}_items", size), &size, |b, _| {
                b.iter(|| template.render(black_box(&ctx)).unwrap())
            });
        }
    }

    group.finish();
}

// Memory efficiency test (rendering multiple times)
fn bench_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");

    // Askama - multiple renders
    {
        use askama::Template;

        #[derive(Template)]
        #[template(
            source = "<div>{{ issue.title }} - {{ issue.status }}</div>",
            ext = "html"
        )]
        struct IssueTemplate {
            issue: TestIssue,
        }

        group.bench_function("askama_repeated_renders", |b| {
            b.iter(|| {
                for _i in 0..100 {
                    let issue = create_test_issues(1).remove(0);
                    let template = IssueTemplate { issue };
                    black_box(template.render().unwrap());
                }
            })
        });
    }

    // Minijinja - multiple renders
    {
        let mut env = minijinja::Environment::new();
        env.add_template("issue", "<div>{{ issue.title }} - {{ issue.status }}</div>")
            .unwrap();
        let template = env.get_template("issue").unwrap();

        group.bench_function("minijinja_repeated_renders", |b| {
            b.iter(|| {
                for _i in 0..100 {
                    let issue = create_test_issues(1).remove(0);
                    let ctx = serde_json::json!({ "issue": issue });
                    black_box(template.render(&ctx).unwrap());
                }
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    askama_tests::bench_simple_list,
    askama_tests::bench_complex_list,
    minijinja_tests::bench_simple_list,
    minijinja_tests::bench_complex_list,
    bench_template_compilation,
    bench_scalability,
    bench_memory_efficiency
);
criterion_main!(benches);
