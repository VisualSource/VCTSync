//! Host-side helpers used by the IPC handlers in `app.rs`.

mod game;
mod ipc;
mod paths;
mod profiles;

pub use game::game_status;
pub use ipc::ipc_reply;
pub use paths::{app_dir, profile_from_dir, write_app_file};
pub use profiles::{delete_profile_dir, load_profiles, save_profiles};
