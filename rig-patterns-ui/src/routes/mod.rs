//! API routes for the rig-patterns UI

pub mod execute;
pub mod compare;
pub mod presets;
pub mod websocket;

pub use execute::execute_pattern;
pub use compare::compare_patterns;
pub use presets::get_presets;
pub use websocket::ws_handler;
