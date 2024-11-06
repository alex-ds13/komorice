use crate::apperror::{AppError, AppErrorKind};
use crate::Message;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_std::channel::{self, Receiver};
use iced::futures::{SinkExt, StreamExt};
use iced::Subscription;
use komorebi_client::StaticConfig;
use notify_debouncer_mini::{
    new_debouncer,
    notify::{ReadDirectoryChangesWatcher, RecursiveMode},
    DebounceEventResult, DebouncedEvent, DebouncedEventKind, Debouncer,
};

#[derive(Clone, Debug)]
pub enum GlobalConfigChangeType {
    CrossBoundaryBehaviour(Arc<str>), // maps komorebi::CrossBoundaryBehaviour to String on GlobalConfigStrs
    CrossMonitorMoveBehaviour(komorebi::MoveBehaviour),
    DefaultContainerPadding(String), // maps i32 to String on GlobalConfigStrs
    DefaultWorkspacePadding(String), // maps i32 to String on GlobalConfigStrs
    DisplayIndexPreferences(HashMap<usize, String>),
    FloatOverride(bool),
    FocusFollowsMouse1(Option<komorebi::FocusFollowsMouseImplementation>), // maps komorebi::FocusFollowsMouseImplementation to String on GlobalConfigStrs
    FocusFollowsMouse(Arc<str>), // maps komorebi::FocusFollowsMouseImplementation to String on GlobalConfigStrs
    GlobalWorkAreaOffset(komorebi::Rect),
    MouseFollowsFocus(bool),
    ResizeDelta(String), // maps i32 to String on GlobalConfigStrs
    Transparency(bool),
    TransparencyAlpha(String), // maps u8 to String on GlobalConfigStrs
    UnmanagedWindowBehaviour(komorebi::OperationBehaviour),
    WindowContainerBehaviour(komorebi::WindowContainerBehaviour),
    WindowHidingBehaviour(komorebi::HidingBehaviour),
}

#[derive(Clone, Debug)]
pub enum MonitorConfigChangeType {
    WindowBasedWorkAreaOffset(komorebi::Rect),
    WindowBasedWorkAreaOffsetLimit(String), // maps i32 to String on MonitorConfigStrs
    WorkAreaOffset(komorebi::Rect),
}

#[derive(Clone, Debug)]
pub enum WorkspaceConfigChangeType {
    ApplyWindowBasedWorkAreaOffset(bool),
    ContainerPadding(String), // maps i32 to String on WorkspaceConfigStrs
    FloatOverride(bool),
    Layout(komorebi::Layout),
    Name(String),
    WindowContainerBehaviour(komorebi::WindowContainerBehaviour),
    WorkspacePadding(String), // maps i32 to String on WorkspaceConfigStrs
}

pub struct ConfigStrs {
    pub global_config_strs: GlobalConfigStrs,
    pub monitors_config_strs: HashMap<usize, MonitorConfigStrs>,
    pub workspaces_config_strs: HashMap<(usize, usize), WorkspaceConfigStrs>,
}

pub struct GlobalConfigStrs {
    pub cross_boundary_behaviour: Arc<str>,
    pub default_container_padding: Arc<str>,
    pub default_workspace_padding: Arc<str>,
    pub focus_follows_mouse: Arc<str>,
    pub resize_delta: Arc<str>,
    pub transparency_alpha: Arc<str>,
}

pub struct MonitorConfigStrs {
    pub window_based_work_area_offset_limit: Arc<str>,
}

pub struct WorkspaceConfigStrs {
    pub container_padding: Arc<str>,
    pub workspace_padding: Arc<str>,
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
