pub mod board;
pub mod general;
pub mod landing;
pub mod metrics;
pub mod prds;
pub mod tasks;

pub use board::board;
pub use general::{graph, health_check, serve_css, serve_favicon, serve_js, palette};
pub use landing::landing;
pub use metrics::metrics_handler;
pub use prds::{prd_view, prds_list};
pub use tasks::{
    create_task, edit_task, list_tasks, new_task_form, task_detail, tasks_list, update_task,
};
