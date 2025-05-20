pub mod unparser;

use crate::{
    apperror::{AppError, AppErrorKind},
    screen::{self, Screen},
};

use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use async_compat::Compat;
use iced::{
    futures::{channel::mpsc, SinkExt},
    widget::markdown,
    Element, Subscription, Task, Theme,
};
use notify_debouncer_mini::{
    new_debouncer,
    notify::{ReadDirectoryChangesWatcher, RecursiveMode},
    DebounceEventResult, DebouncedEvent, DebouncedEventKind, Debouncer,
};
use smol::channel::{self, Receiver, Sender};
pub use whkd_core::{HotkeyBinding, Shell, Whkdrc};

pub static DEFAULT_WHKDRC: Whkdrc = Whkdrc {
    shell: Shell::Pwsh,
    app_bindings: Vec::new(),
    bindings: Vec::new(),
    pause_binding: None,
    pause_hook: None,
};

#[derive(Debug, Clone)]
pub enum Message {
    WhkdrcFileWatcherTx(Sender<Input>),
    LoadedWhkdrc(Arc<Whkdrc>),
    FailedToLoadWhkdrc(AppError),
    SavedWhkdrc,
    AppError(AppError),

    // Messages related to CLI commands fetch
    LoadedCommands(Vec<String>),
    FailedToLoadCommands(String),
    LoadedCommandDescription(String, String),
    FailedToLoadCommandsDescription(String),

    // Messages related to screens
    Whkd(screen::whkd::Message),
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    LoadedWhkdrc,
    AppError(AppError),
}

#[derive(Debug)]
pub struct Whkd {
    whkdrc_watcher_tx: Option<Sender<Input>>,
    pub whkdrc: Whkdrc,
    pub loaded_whkdrc: Arc<Whkdrc>,
    pub is_dirty: bool,
    pub whkd: screen::whkd::Whkd,
    pub screen: Screen,
    pub loaded_commands: bool,
    commands: Vec<String>,
    commands_desc: HashMap<String, Vec<markdown::Item>>,
}

impl Default for Whkd {
    fn default() -> Self {
        Self {
            whkdrc_watcher_tx: None,
            whkdrc: DEFAULT_WHKDRC.clone(),
            loaded_whkdrc: Arc::new(DEFAULT_WHKDRC.clone()),
            is_dirty: false,
            whkd: Default::default(),
            screen: Screen::Whkd,
            loaded_commands: false,
            commands: Default::default(),
            commands_desc: Default::default(),
        }
    }
}

impl Whkd {
    pub fn update(&mut self, message: Message) -> (Action, Task<Message>) {
        match message {
            Message::WhkdrcFileWatcherTx(sender) => self.whkdrc_watcher_tx = Some(sender),
            Message::LoadedWhkdrc(whkdrc) => {
                if let Some(whkdrc) = Arc::into_inner(whkdrc) {
                    println!("Whkdrc Loaded: {whkdrc:#?}");
                    // let whkdrc = whkdrc::merge_default(whkdrc);
                    self.whkdrc = whkdrc.clone();
                    // self.home.has_loaded_whkdrc = true;
                    self.loaded_whkdrc = Arc::new(whkdrc);
                    self.whkd = screen::whkd::Whkd::new(&self.whkdrc);
                    //TODO: show message on app to load external changes
                    return (Action::LoadedWhkdrc, Task::none());
                }
            }
            Message::FailedToLoadWhkdrc(app_error) => {
                return (Action::AppError(app_error), Task::none());
            }
            Message::SavedWhkdrc => {
                if let Some(sender) = &self.whkdrc_watcher_tx {
                    let _ = sender.try_send(Input::IgnoreNextEvent);
                }
                self.loaded_whkdrc = Arc::new(self.whkdrc.clone());
                self.is_dirty = false;
            }
            Message::AppError(app_error) => {
                return (Action::AppError(app_error), Task::none());
            }
            Message::Whkd(message) => {
                let (action, task) = self.whkd.update(message, &mut self.whkdrc);
                let action_task = match action {
                    screen::whkd::Action::None => Task::none(),
                };
                self.check_changes();
                return (
                    Action::None,
                    Task::batch([task, action_task]).map(Message::Whkd),
                );
            }
            Message::LoadedCommands(commands) => {
                // println!("{commands:?}");
                self.commands = commands;
                self.loaded_commands = true;
                return (Action::None, self.load_commands_description());
            }
            Message::FailedToLoadCommands(error) => {
                println!("WHKD -> Failed to load commands: {error}");
            }
            Message::LoadedCommandDescription(command, description) => {
                println!("received description for command: {command}");
                let md = markdown::parse(&description).collect();
                self.commands_desc.insert(command, md);
            }
            Message::FailedToLoadCommandsDescription(error) => {
                println!("WHKD -> Failed to load commands: {error}");
            }
        }
        (Action::None, Task::none())
    }

    pub fn view<'a>(&'a self, theme: &'a Theme) -> Element<'a, Message> {
        self.whkd
            .view(&self.whkdrc, &self.commands, &self.commands_desc, theme)
            .map(Message::Whkd)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let screen_subscription = match self.screen {
            Screen::Whkd => self.whkd.subscription().map(Message::Whkd),
            _ => Subscription::none(),
        };

        Subscription::batch([worker(), screen_subscription])
    }

    pub fn discard_changes(&mut self) {
        self.whkdrc = (*self.loaded_whkdrc).clone();
        self.is_dirty = false;
    }

    fn check_changes(&mut self) {
        self.is_dirty = self.whkdrc != *self.loaded_whkdrc;
    }

    pub fn load_commands(&self) -> Task<Message> {
        Task::future(Compat::new(async {
            static APP_USER_AGENT: &str =
                concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

            println!("Running GET request: {}", APP_USER_AGENT);

            let client = reqwest::Client::builder()
                .user_agent(APP_USER_AGENT)
                .build()?;
            client
                .get("https://api.github.com/repos/lgug2z/komorebi/contents/docs/cli")
                .send()
                .await
        }))
        .then(|res| match res {
            Ok(response) => Task::perform(
                Compat::new(async {
                    #[derive(serde::Deserialize)]
                    struct Command {
                        name: String,
                    }
                    response.json::<Vec<Command>>().await
                }),
                |res| match res {
                    Ok(commands) => Message::LoadedCommands(
                        commands
                            .into_iter()
                            .flat_map(|c| c.name.strip_suffix(".md").map(|v| v.to_string()))
                            .collect(),
                    ),
                    Err(error) => Message::FailedToLoadCommands(error.to_string()),
                },
            ),
            Err(error) => Task::done(Message::FailedToLoadCommands(error.to_string())),
        })
    }

    pub fn load_commands_description(&self) -> Task<Message> {
        Task::batch(self.commands.iter().map(|command| {
            let command_c = command.clone();
            let command_c1 = command.clone();
            Task::future(Compat::new(async move {
                static APP_USER_AGENT: &str =
                    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

                println!(
                    "Running GET request for command {} description: {}",
                    &command_c1, APP_USER_AGENT
                );

                let client = reqwest::Client::builder()
                    .user_agent(APP_USER_AGENT)
                    .build()?;
                client
                    .get(format!(
                        "https://raw.githubusercontent.com/lgug2z/komorebi/master/docs/cli/{}.md",
                        &command_c1,
                    ))
                    .send()
                    .await
            }))
            .then(move |res| {
                let command_c = command_c.clone();
                match res {
                    Ok(response) => {
                        Task::perform(Compat::new(async { response.text().await }), move |res| {
                            match res {
                                Ok(description) => {
                                    Message::LoadedCommandDescription(command_c, description)
                                }
                                Err(error) => {
                                    Message::FailedToLoadCommandsDescription(error.to_string())
                                }
                            }
                        })
                    }
                    Err(error) => {
                        Task::done(Message::FailedToLoadCommandsDescription(error.to_string()))
                    }
                }
            })
        }))
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
        iced::stream::channel(10, move |mut output: mpsc::Sender<Message>| async move {
            let mut state = State::Starting;

            loop {
                match state {
                    State::Starting => {
                        let (sender, receiver) = channel::bounded(10);

                        let sender_clone = sender.clone();
                        match output
                            .send(Message::WhkdrcFileWatcherTx(sender_clone))
                            .await
                        {
                            Ok(_) => {}
                            Err(e) => {
                                println!("Error trying to send the options watcher sender:\n{e:?}");
                            }
                        }

                        let mut debouncer = new_debouncer(Duration::from_millis(250), move |res| {
                            smol::block_on(async {
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

                        let path = config_path();
                        if matches!(std::fs::exists(&path), Ok(false) | Err(_)) {
                            if let Err(apperror) = save(DEFAULT_WHKDRC.clone()).await {
                                match output.send(Message::AppError(apperror)).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        println!("Error trying to send error:\n{e:?}");
                                    }
                                }
                            }
                        }
                        debouncer
                            .watcher()
                            .watch(&path, RecursiveMode::NonRecursive)
                            .unwrap();

                        state = State::Ready(Data {
                            debouncer,
                            receiver,
                            ignore_event: 0,
                        });
                    }
                    State::Ready(data) => {
                        let Data {
                            receiver,
                            debouncer,
                            mut ignore_event,
                        } = data;
                        let input = receiver.recv().await;

                        match input {
                            Ok(Input::IgnoreNextEvent) => {
                                println!("IgnoreNextEvent");
                                state = State::Ready(Data {
                                    debouncer,
                                    receiver,
                                    ignore_event: ignore_event + 1,
                                });
                            }
                            Ok(Input::DebouncerRes(res)) => {
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
                            Err(error) => {
                                println!("Error from file watcher: {error:?}");

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
            println!("FileWatcher: loading whkdrc");
            smol::block_on(async {
                match load().await {
                    Ok(loaded_whkdrc) => {
                        let _ = output
                            .send(Message::LoadedWhkdrc(Arc::new(loaded_whkdrc)))
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

pub fn load_task() -> Task<Message> {
    Task::perform(load(), |res| match res {
        Ok(whkdrc) => Message::LoadedWhkdrc(Arc::new(whkdrc)),
        Err(apperror) => Message::FailedToLoadWhkdrc(apperror),
    })
}

pub async fn load() -> Result<Whkdrc, AppError> {
    use smol::prelude::*;
    use whkd_parser::chumsky::Parser;

    let mut contents = String::new();

    let file_open_res = smol::fs::File::open(config_path()).await;

    let mut file = match file_open_res {
        Ok(file) => file,
        Err(error) => {
            println!("Failed to find 'whkdrc' file.\nError: {}", error);
            return Err(AppError {
                title: "Failed to find 'whkdrc' file.".into(),
                description: None,
                kind: AppErrorKind::Info,
            });
        }
    };

    file.read_to_string(&mut contents)
        .await
        .map_err(|e| AppError {
            title: "Error opening 'whkdrc' file.".into(),
            description: Some(e.to_string()),
            kind: AppErrorKind::Error,
        })?;

    smol::unblock(|| {
        whkd_parser::parser().parse(contents).map_err(|e| AppError {
            title: "Error reading 'whkdrc' file.".into(),
            description: Some(format!("{e:#?}")),
            kind: AppErrorKind::Error,
        })
    })
    .await
}

pub fn save_task(whkdrc: Whkdrc) -> Task<Message> {
    Task::future(save(whkdrc)).map(|res| match res {
        Ok(_) => Message::SavedWhkdrc,
        Err(apperror) => Message::AppError(apperror),
    })
}

pub async fn save(whkdrc: Whkdrc) -> Result<(), AppError> {
    use smol::prelude::*;

    let str = smol::unblock(move || unparser::unparse_whkdrc(&whkdrc)).await;

    let path = config_path();

    if let Some(dir) = path.parent() {
        smol::fs::create_dir_all(dir).await.map_err(|e| AppError {
            title: "Error creating folder for 'whkdrc' file".into(),
            description: Some(e.to_string()),
            kind: AppErrorKind::Error,
        })?;
    }

    let mut file = smol::fs::File::create(path).await.map_err(|e| AppError {
        title: "Error creating 'whkdrc' file.".into(),
        description: Some(e.to_string()),
        kind: AppErrorKind::Error,
    })?;

    file.write_all(str.as_bytes()).await.map_err(|e| AppError {
        title: "Error saving 'whkdrc' file".into(),
        description: Some(e.to_string()),
        kind: AppErrorKind::Error,
    })?;

    // This is a simple way to save at most once every couple seconds
    // smol::Timer::after(std::time::Duration::from_secs(2)).await;

    Ok(())
}

pub fn home_path() -> PathBuf {
    std::env::var("WHKD_CONFIG_HOME").map_or_else(
        |_| {
            dirs::home_dir()
                .expect("there is no home directory")
                .join(".config/")
        },
        |home_path| {
            let home = PathBuf::from(&home_path);

            if home.as_path().is_dir() {
                home
            } else {
                panic!(
                    "$Env:WHKD_CONFIG_HOME is set to '{home_path}', which is not a valid directory"
                );
            }
        },
    )
}

pub fn config_path() -> PathBuf {
    let home_dir = home_path();

    home_dir.join("whkdrc")
}
