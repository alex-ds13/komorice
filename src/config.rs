use crate::apperror::{AppError, AppErrorKind};
use crate::Message;

use std::sync::Arc;
use std::time::Duration;

use async_std::channel::{self, Receiver};
use iced::futures::{SinkExt, StreamExt};
use iced::Subscription;
use komorebi_client::{
    AnimationStyle, AnimationsConfig, BorderColours, BorderImplementation, BorderStyle, Colour,
    CrossBoundaryBehaviour, HidingBehaviour, MonitorConfig, MoveBehaviour, OperationBehaviour,
    PerAnimationPrefixConfig, Rgb, StackbarConfig, StackbarLabel, StackbarMode, StaticConfig,
    TabsConfig, WindowContainerBehaviour, WorkspaceConfig,
};
use notify_debouncer_mini::{
    new_debouncer,
    notify::{ReadDirectoryChangesWatcher, RecursiveMode},
    DebounceEventResult, DebouncedEvent, DebouncedEventKind, Debouncer,
};
use lazy_static::lazy_static;

lazy_static!{
    pub static ref DEFAULT_CONFIG: StaticConfig = StaticConfig {
        invisible_borders: None,
        minimum_window_width: None,
        minimum_window_height: None,
        resize_delta: Some(50),
        window_container_behaviour: Some(WindowContainerBehaviour::Create),
        float_override: Some(false),
        cross_monitor_move_behaviour: Some(MoveBehaviour::Swap),
        cross_boundary_behaviour: Some(CrossBoundaryBehaviour::Monitor),
        unmanaged_window_operation_behaviour: Some(OperationBehaviour::Op),
        focus_follows_mouse: None,
        mouse_follows_focus: Some(true),
        app_specific_configuration_path: None,
        border_width: Some(8),
        border_offset: Some(-1),
        border: Some(false),
        border_colours: Some(BorderColours {
            single: Some(Colour::Rgb(Rgb::new(66, 165, 245))),
            stack: Some(Colour::Rgb(Rgb::new(0, 165, 66))),
            monocle: Some(Colour::Rgb(Rgb::new(255, 51, 153))),
            floating: Some(Colour::Rgb(Rgb::new(245, 245, 165))),
            unfocused: Some(Colour::Rgb(Rgb::new(128, 128, 128))),
        }),
        border_style: Some(BorderStyle::default()),
        border_z_order: None,
        border_implementation: Some(BorderImplementation::default()),
        transparency: Some(false),
        transparency_alpha: Some(200),
        transparency_ignore_rules: None,
        default_workspace_padding: Some(10),
        default_container_padding: Some(10),
        monitors: Some(Vec::new()),
        window_hiding_behaviour: Some(HidingBehaviour::Cloak),
        global_work_area_offset: None,
        ignore_rules: None,
        manage_rules: None,
        floating_applications: None,
        border_overflow_applications: None,
        tray_and_multi_window_applications: None,
        layered_applications: None,
        object_name_change_applications: None,
        monitor_index_preferences: None,
        display_index_preferences: None,
        stackbar: Some(StackbarConfig {
            height: Some(40),
            label: Some(StackbarLabel::Process),
            mode: Some(StackbarMode::OnStack),
            tabs: Some(TabsConfig {
                width: Some(200),
                focused_text: Some(Colour::Rgb(Rgb::new(0xFF, 0xFF, 0xFF))),
                unfocused_text: Some(Colour::Rgb(Rgb::new(0xB3, 0xB3, 0xB3))),
                background: Some(Colour::Rgb(Rgb::new(0x33, 0x33, 0x33))),
                font_family: None,
                font_size: None,
            }),
        }),
        animation: Some(AnimationsConfig {
            enabled: PerAnimationPrefixConfig::Global(false),
            duration: Some(PerAnimationPrefixConfig::Global(250)),
            style: Some(PerAnimationPrefixConfig::Global(AnimationStyle::Linear)),
            fps: Some(60),
        }),
        theme: None,
        slow_application_identifiers: None,
        slow_application_compensation_time: Some(20),
        bar_configurations: None,
    };
}

trait ChangeConfig {
    fn change_config(&mut self, mut f: impl FnMut(&mut Self)) {
        f(self);
    }

    fn change_monitor_config(&mut self, idx: usize, f: impl Fn(&mut MonitorConfig));

    fn change_workspace_config(
        &mut self,
        monitor_idx: usize,
        workspace_idx: usize,
        f: impl Fn(&mut WorkspaceConfig),
    );
}

impl ChangeConfig for StaticConfig {
    fn change_monitor_config(&mut self, idx: usize, f: impl Fn(&mut MonitorConfig)) {
        if let Some(monitors) = &mut self.monitors {
            if let Some(monitor) = monitors.get_mut(idx) {
                f(monitor);
            } else {
                monitors.reserve(idx + 1 - monitors.len());
                for _ in monitors.len()..(idx + 1) {
                    monitors.push(MonitorConfig {
                        workspaces: Vec::new(),
                        work_area_offset: None,
                        window_based_work_area_offset: None,
                        window_based_work_area_offset_limit: None,
                    });
                }
                f(&mut monitors[idx]);
            }
        } else {
            let mut monitors = vec![
                komorebi::MonitorConfig {
                    workspaces: Vec::new(),
                    work_area_offset: None,
                    window_based_work_area_offset: None,
                    window_based_work_area_offset_limit: None,
                };
                idx + 1
            ];
            f(&mut monitors[idx]);
            self.monitors = Some(monitors);
        }
    }

    fn change_workspace_config(
        &mut self,
        monitor_idx: usize,
        workspace_idx: usize,
        f: impl Fn(&mut WorkspaceConfig),
    ) {
        self.change_monitor_config(monitor_idx, |monitor| {
            if let Some(workspace) = monitor.workspaces.get_mut(workspace_idx) {
                f(workspace);
            } else {
                monitor
                    .workspaces
                    .reserve(workspace_idx + 1 - monitor.workspaces.len());
                for _ in monitor.workspaces.len()..(workspace_idx + 1) {
                    monitor.workspaces.push(WorkspaceConfig {
                        name: String::default(),
                        layout: None,
                        custom_layout: None,
                        layout_rules: None,
                        custom_layout_rules: None,
                        container_padding: None,
                        workspace_padding: None,
                        initial_workspace_rules: None,
                        workspace_rules: None,
                        apply_window_based_work_area_offset: None,
                        window_container_behaviour: None,
                        float_override: None,
                    });
                }
                f(&mut monitor.workspaces[workspace_idx]);
            }
        });
    }
}

enum State {
    Starting,
    Ready(Data),
}

struct Data {
    debouncer: Debouncer<ReadDirectoryChangesWatcher>,
    receiver: Receiver<Input>,
    ignore_event: usize,
}

pub enum Input {
    IgnoreNextEvent,
    DebouncerRes(DebounceEventResult),
}

pub fn worker() -> Subscription<Message> {
    Subscription::run(|| {
        iced::stream::channel(10, move |mut output| async move {
            let mut state = State::Starting;

            loop {
                match state {
                    State::Starting => {
                        let (sender, receiver) = channel::bounded(10);

                        let sender_clone = sender.clone();
                        match output
                            .send(Message::ConfigFileWatcherTx(sender_clone))
                            .await
                        {
                            Ok(_) => {}
                            Err(e) => {
                                println!("Error trying to send the options watcher sender:\n{e:?}");
                            }
                        }

                        let mut debouncer = new_debouncer(Duration::from_millis(250), move |res| {
                            async_std::task::block_on(async {
                                let input = Input::DebouncerRes(res);
                                match sender.send(input).await {
                                    Ok(_) => {}
                                    Err(error) => {
                                        println!(
                                            "Error sending a debounced \
                                                event to the worker channel. \
                                                E: {error:?}"
                                        );
                                    }
                                }
                            })
                        })
                        .unwrap();

                        debouncer
                            .watcher()
                            .watch(&config_path(), RecursiveMode::NonRecursive)
                            .unwrap();

                        state = State::Ready(Data {
                            debouncer,
                            receiver,
                            ignore_event: 0,
                        });
                    }
                    State::Ready(data) => {
                        let Data {
                            mut receiver,
                            debouncer,
                            mut ignore_event,
                        } = data;
                        let input = receiver.select_next_some().await;

                        match input {
                            Input::IgnoreNextEvent => {
                                println!("IgnoreNextEvent");
                                state = State::Ready(Data {
                                    debouncer,
                                    receiver,
                                    ignore_event: ignore_event + 1,
                                });
                            }
                            Input::DebouncerRes(res) => {
                                match res {
                                    Ok(events) => {
                                        events.iter().for_each(|event| {
                                            handle_event(event, &mut ignore_event, &mut output);
                                        });
                                    }
                                    Err(error) => {
                                        println!("Error from file watcher: {error:?}")
                                    }
                                }

                                state = State::Ready(Data {
                                    debouncer,
                                    receiver,
                                    ignore_event,
                                });
                            }
                        }
                    }
                }
            }
        })
    })
}

fn handle_event(
    event: &DebouncedEvent,
    ignore_event: &mut usize,
    output: &mut iced::futures::channel::mpsc::Sender<Message>,
) {
    // println!("FileWatcher event: {event:?}");
    if let DebouncedEventKind::Any = event.kind {
        if *ignore_event == 0 {
            println!("FileWatcher: loading options");
            async_std::task::block_on(async {
                match load().await {
                    Ok(loaded_options) => {
                        let _ = output
                            .send(Message::LoadedConfig(Arc::new(loaded_options)))
                            .await;
                    }
                    Err(e) => {
                        let _ = output.send(Message::AppError(e)).await;
                    }
                }
            });
        } else {
            println!("FileWatcher: ignoring event");
            *ignore_event = ignore_event.saturating_sub(1);
        }
    }
}

pub async fn load() -> Result<StaticConfig, AppError> {
    use async_std::prelude::*;

    let mut contents = String::new();

    let file_open_res = async_std::fs::File::open(config_path()).await;

    let mut file = match file_open_res {
        Ok(file) => file,
        Err(error) => {
            println!("Failed to find 'komorebi.json' file.\nError: {}", error);
            return Err(AppError {
                title: "Failed to find 'komorebi.json' file.".into(),
                description: None,
                kind: AppErrorKind::Info,
            });
        }
    };

    file.read_to_string(&mut contents)
        .await
        .map_err(|e| AppError {
            title: "Error opening 'komorebi.json' file.".into(),
            description: Some(e.to_string()),
            kind: AppErrorKind::Error,
        })?;

    serde_json::from_str(&contents).map_err(|e| AppError {
        title: "Error reading 'komorebi.json' file.".into(),
        description: Some(e.to_string()),
        kind: AppErrorKind::Error,
    })
}

pub async fn save(config: StaticConfig) -> Result<(), AppError> {
    use async_std::prelude::*;

    let json = serde_json::to_string_pretty(&config).map_err(|e| AppError {
        title: "Error writing to 'komorebi.json' file".into(),
        description: Some(e.to_string()),
        kind: AppErrorKind::Error,
    })?;

    let path = config_path();

    // if let Some(dir) = path.parent() {
    //     async_std::fs::create_dir_all(dir)
    //         .await
    //         .map_err(|e| AppError {
    //             title: "Error creating folder for 'komorebi.json' file".into(),
    //             description: Some(e.to_string()),
    //             kind: AppErrorKind::Error,
    //         })?;
    // }

    let mut file = async_std::fs::File::create(path)
        .await
        .map_err(|e| AppError {
            title: "Error creating 'komorebi.json' file.".into(),
            description: Some(e.to_string()),
            kind: AppErrorKind::Error,
        })?;

    file.write_all(json.as_bytes())
        .await
        .map_err(|e| AppError {
            title: "Error saving 'komorebi.json' file".into(),
            description: Some(e.to_string()),
            kind: AppErrorKind::Error,
        })?;

    // This is a simple way to save at most once every couple seconds
    // async_std::task::sleep(std::time::Duration::from_secs(2)).await;

    Ok(())
}

pub fn config_path() -> std::path::PathBuf {
    let home_dir: std::path::PathBuf = std::env::var("KOMOREBI_CONFIG_HOME").map_or_else(
        |_| dirs::home_dir().expect("there is no home directory"),
        |home_path| {
            let home = std::path::PathBuf::from(&home_path);

            if home.as_path().is_dir() {
                home
            } else {
                panic!("$Env:KOMOREBI_CONFIG_HOME is set to '{home_path}', which is not a valid directory");
            }
        },
    );

    home_dir.join("komorebi.json")
}
