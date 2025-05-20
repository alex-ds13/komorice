use whkd_core::Whkdrc;

// Define a function to generate a configuration file from a Whkdrc instance
pub fn unparse_whkdrc(whkdrc: &Whkdrc) -> String {
    let mut contents = String::new();

    let delimiter = " : ";

    // Write shell name
    contents.push_str(".shell ");
    contents.push_str(&format!("{}", whkdrc.shell));
    contents.push('\n');

    // Write .pause binding and hook
    if let Some(pause_binding) = &whkdrc.pause_binding {
        contents.push_str(".pause ");
        contents.push_str(&pause_binding.join(" + "));
        contents.push('\n');
    }
    if let Some(pause_hook) = &whkdrc.pause_hook {
        contents.push_str(".pause_hook ");
        contents.push_str(pause_hook);
        contents.push('\n');
    }

    // Blank line for separation
    contents.push('\n');

    // Add app bindings
    for (keys, process_bindings) in &whkdrc.app_bindings {
        let mut lines = Vec::new();
        let mut keys_str = keys.join(" + ");
        keys_str.push_str(" [");
        lines.push(keys_str);
        for process_binding in process_bindings {
            let mut line = String::from("    ");
            if let Some(process_name) = &process_binding.process_name {
                line.push_str(process_name);
                line.push_str(delimiter);
                line.push_str(&process_binding.command);
            }
            lines.push(line);
        }
        lines.push(String::from("]"));

        contents.push_str(&lines.join("\n"));
        contents.push('\n');
    }

    // Blank line for separation
    contents.push('\n');

    // Add bindings
    for binding in &whkdrc.bindings {
        contents.push_str(&binding.keys.join(" + "));
        contents.push_str(delimiter);
        contents.push_str(&binding.command);
        contents.push('\n');
    }

    println!("{contents}");
    contents
}

#[cfg(test)]
#[test]
fn test() {
    use whkd_core::{HotkeyBinding, Shell, Whkdrc};

    let whkdrc = Whkdrc {
        shell: Shell::Cmd,
        app_bindings: vec![(
            vec![String::from("alt"), String::from("n")],
            vec![
                HotkeyBinding {
                    keys: vec![String::from("alt"), String::from("n")],
                    command: String::from(r#"echo "hello firefox""#),
                    process_name: Option::from("Firefox".to_string()),
                },
                HotkeyBinding {
                    keys: vec![String::from("alt"), String::from("n")],
                    command: String::from(r#"echo "hello chrome""#),
                    process_name: Option::from("Google Chrome".to_string()),
                },
            ],
        )],
        bindings: vec![
            HotkeyBinding {
                keys: vec![String::from("alt"), String::from("h")],
                command: String::from("komorebic focus left"),
                process_name: None,
            },
            HotkeyBinding {
                keys: vec![String::from("alt"), String::from("j")],
                command: String::from("komorebic focus down"),
                process_name: None,
            },
            HotkeyBinding {
                keys: vec![String::from("alt"), String::from("k")],
                command: String::from("komorebic focus up"),
                process_name: None,
            },
            HotkeyBinding {
                keys: vec![String::from("alt"), String::from("l")],
                command: String::from("komorebic focus right"),
                process_name: None,
            },
            HotkeyBinding {
                keys: vec![String::from("alt"), String::from("1")],
                command: String::from("komorebic focus-workspace 0"),
                process_name: None,
            },
        ],
        pause_binding: Some(vec![
            "ctrl".to_string(),
            "shift".to_string(),
            "esc".to_string(),
        ]),
        pause_hook: Some("komorebic toggle-pause".to_string()),
    };

    let expected = r#".shell cmd
.pause ctrl + shift + esc
.pause_hook komorebic toggle-pause

alt + n [
    Firefox : echo "hello firefox"
    Google Chrome : echo "hello chrome"
]

alt + h : komorebic focus left
alt + j : komorebic focus down
alt + k : komorebic focus up
alt + l : komorebic focus right
alt + 1 : komorebic focus-workspace 0
"#;

    let res = unparse_whkdrc(&whkdrc);

    assert_eq!(&res, expected);
}
