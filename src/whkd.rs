pub mod unparser;

pub use whkd_core::{HotkeyBinding, Shell, Whkdrc};

pub static DEFAULT_WHKDRC: Whkdrc = Whkdrc {
    shell: Shell::Pwsh,
    app_bindings: Vec::new(),
    bindings: Vec::new(),
    pause_binding: None,
    pause_hook: None,
};
