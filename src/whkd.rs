pub mod unparser;

use whkd_core::{Shell, Whkdrc};

static DEFAULT_WHKDRC: Whkdrc = Whkdrc {
    shell: Shell::Pwsh,
    app_bindings: Vec::new(),
    bindings: Vec::new(),
    pause_binding: None,
    pause_hook: None,
};
