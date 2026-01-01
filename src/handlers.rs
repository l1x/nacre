pub mod board;
pub mod general;
pub mod graph;
pub mod landing;
pub mod metrics;
pub mod prds;
pub mod tasks;

pub use board::board;
pub use general::{
    graph, graph_epic, health_check, palette, serve_autumnus_dark, serve_autumnus_light, serve_css,
    serve_favicon, serve_js,
};
pub use graph::graph_data;
pub use landing::landing;
pub use metrics::metrics_handler;
pub use prds::{prd_view, prds_list};
pub use tasks::{
    create_task, edit_task, list_tasks, new_task_form, task_detail, tasks_list, update_task,
};
