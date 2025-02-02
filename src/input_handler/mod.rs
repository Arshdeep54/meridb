mod editor;
mod history;
mod terminal;

pub use editor::Editor;
pub use history::History;

// Re-export the main input handler
pub use editor::InputHandler;
