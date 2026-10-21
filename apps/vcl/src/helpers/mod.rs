//! Host-side helpers used by the IPC handlers in `app.rs`.

mod game;
mod ipc;
mod logs;
mod paths;
mod profiles;

pub use game::game_status;
pub use ipc::ipc_reply;
pub use logs::read_log;
pub use paths::{app_dir, normalize, profile_from_dir, write_app_file};
pub use profiles::{delete_profile_dir, load_profiles, save_profiles};
