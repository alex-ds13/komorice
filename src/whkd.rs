pub mod unparser;

use crate::{
    apperror::{AppError, AppErrorKind},
    screen::{self, ConfigState, ConfigType, Configuration, Screen, View},
};

use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use async_compat::Compat;
use iced::{
    Subscription, Task, Theme,
    futures::{SinkExt, channel::mpsc},
    widget::{markdown, space},
};
use notify_debouncer_mini::{
    DebounceEventResult, DebouncedEvent, DebouncedEventKind, Debouncer, new_debouncer,
    notify::{ReadDirectoryChangesWatcher, RecursiveMode},
};
use smol::channel::{self, Receiver, Sender};
use smol::process::{Command, ExitStatus, Output, Stdio, windows::CommandExt};
pub use whkd_core::{HotkeyBinding, Shell, Whkdrc};

pub static MODIFIERS: [&str; 4] = ["CTRL", "SHIFT", "ALT", "WIN"];

const CREATE_NO_WINDOW: u32 = 0x08000000;
pub const SEPARATOR: &str = " + ";
pub const UNPADDED_SEPARATOR: &str = "+";

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
    FileWatcherError(AppError),
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
    Bindings(screen::whkd::bindings::Message),

    // Messages related to whkd binary
    WhkdFoundOnPath,
    WhkdNotFoundOnPath,
    StoppedWhkd,
    FailedToStopWhkd,
    StartedWhkd,
    FailedToStartWhkd,
    WhkdStatus(bool),
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    SavedWhkdrc,
    LoadedWhkdrc,
    FailedToLoadWhkdrc(AppError),
    AppError(AppError),
}

#[derive(Debug)]
pub struct Whkd {
    whkdrc_watcher_tx: Option<Sender<Input>>,
    pub whkdrc: Whkdrc,
    pub loaded_whkdrc: Arc<Whkdrc>,
    pub is_dirty: bool,
    pub whkd: screen::whkd::Whkd,
    pub bindings: screen::whkd::Bindings,
    pub screen: Screen,
    pub loaded_commands: bool,
    commands: Vec<String>,
    commands_desc: HashMap<String, Vec<markdown::Item>>,
    whkd_bin: WhkdBinary,
}

impl Default for Whkd {
    fn default() -> Self {
        Self {
            whkdrc_watcher_tx: None,
            whkdrc: DEFAULT_WHKDRC.clone(),
            loaded_whkdrc: Arc::new(DEFAULT_WHKDRC.clone()),
            is_dirty: false,
            whkd: Default::default(),
            bindings: Default::default(),
            screen: Screen::Whkd,
            loaded_commands: false,
            commands: Default::default(),
            commands_desc: Default::default(),
            whkd_bin: Default::default(),
        }
    }
}

impl Whkd {
    pub fn init() -> (Self, Task<Message>) {
        (Self::default(), Task::batch([find_whkd(), whkd_status()]))
    }

    pub fn update(&mut self, message: Message) -> (Action, Task<Message>) {
        match message {
            Message::WhkdrcFileWatcherTx(sender) => self.whkdrc_watcher_tx = Some(sender),
            Message::FileWatcherError(app_error) => {
                return (Action::AppError(app_error), Task::none());
            }
            Message::LoadedWhkdrc(whkdrc) => {
                if let Some(whkdrc) = Arc::into_inner(whkdrc) {
                    // println!("Whkdrc Loaded: {whkdrc:#?}");
                    self.whkdrc = whkdrc.clone();
                    self.loaded_whkdrc = Arc::new(whkdrc);
                    self.refresh();
                    //TODO: show message on app to load external changes
                    return (Action::LoadedWhkdrc, Task::none());
                }
            }
            Message::FailedToLoadWhkdrc(app_error) => {
                return (Action::FailedToLoadWhkdrc(app_error), Task::none());
            }
            Message::SavedWhkdrc => {
                if let Some(sender) = &self.whkdrc_watcher_tx {
                    let _ = sender.try_send(Input::IgnoreNextEvent);
                }
                self.loaded_whkdrc = Arc::new(self.whkdrc.clone());
                self.is_dirty = false;
                return (Action::SavedWhkdrc, Task::none());
            }
            Message::AppError(app_error) => {
                return (Action::AppError(app_error), Task::none());
            }
            Message::Whkd(message) => {
                let (action, task) = self.whkd.update(message, &mut self.whkdrc);
                let action_task = match action {
                    screen::whkd::Action::None => Task::none(),
                    screen::whkd::Action::StopWhkd => {
                        if self.whkd_bin.found
                            && self.whkd_bin.running_initial
                            && self.whkd_bin.running_current
                        {
                            stop_whkd()
                        } else {
                            Task::none()
                        }
                    }
                    screen::whkd::Action::StartWhkd => {
                        if self.whkd_bin.found
                            && self.whkd_bin.running_initial
                            && !self.whkd_bin.running_current
                        {
                            restart_whkd()
                        } else {
                            Task::none()
                        }
                    }
                };
                self.check_changes();
                return (
                    Action::None,
                    Task::batch([task.map(Message::Whkd), action_task]),
                );
            }
            Message::Bindings(message) => {
                let (action, task) =
                    self.bindings
                        .update(message, &mut self.whkdrc, &self.commands);
                let action_task = match action {
                    screen::whkd::bindings::Action::None => Task::none(),
                    screen::whkd::bindings::Action::StopWhkd => {
                        if self.whkd_bin.found
                            && self.whkd_bin.running_initial
                            && self.whkd_bin.running_current
                        {
                            stop_whkd()
                        } else {
                            Task::none()
                        }
                    }
                    screen::whkd::bindings::Action::StartWhkd => {
                        if self.whkd_bin.found
                            && self.whkd_bin.running_initial
                            && !self.whkd_bin.running_current
                        {
                            restart_whkd()
                        } else {
                            Task::none()
                        }
                    }
                };
                self.check_changes();
                return (
                    Action::None,
                    Task::batch([task.map(Message::Bindings), action_task]),
                );
            }
            Message::LoadedCommands(commands) => {
                // println!("{commands:?}");
                self.commands = commands;
                self.whkd.load_new_commands(&self.commands);
                self.bindings.load_new_commands(&self.commands);
                self.loaded_commands = true;
                return (Action::None, self.load_commands_description());
            }
            Message::FailedToLoadCommands(error) => {
                println!("WHKD -> Failed to load commands: {error}");
            }
            Message::LoadedCommandDescription(command, description) => {
                // println!("received description for command: {command}");
                let md = markdown::parse(&description).collect();
                self.commands_desc.insert(command, md);
            }
            Message::FailedToLoadCommandsDescription(error) => {
                println!("WHKD -> Failed to load commands: {error}");
            }
            Message::WhkdFoundOnPath => self.whkd_bin.found = true,
            Message::WhkdNotFoundOnPath => self.whkd_bin.found = false,
            Message::StoppedWhkd => self.whkd_bin.running_current = false,
            Message::FailedToStopWhkd => {}
            Message::StartedWhkd => self.whkd_bin.running_current = true,
            Message::FailedToStartWhkd => {}
            Message::WhkdStatus(running) => {
                self.whkd_bin.running_initial = running;
                self.whkd_bin.running_current = running;
            }
        }
        (Action::None, Task::none())
    }

    pub fn view<'a>(&'a self, theme: &'a Theme) -> View<'a, Message> {
        match self.screen {
            Screen::Whkd => self
                .whkd
                .view(
                    &self.whkdrc,
                    &self.whkd_bin,
                    &self.commands,
                    &self.commands_desc,
                    theme,
                )
                .map(Message::Whkd),
            Screen::WhkdBinding => self
                .bindings
                .view(
                    &self.whkdrc,
                    &self.whkd_bin,
                    &self.commands,
                    &self.commands_desc,
                    theme,
                )
                .map(Message::Bindings),
            _ => space::horizontal().into(),
        }
    }

    pub fn subscription(&self, configuration: &Configuration) -> Subscription<Message> {
        let screen_subscription = match self.screen {
            Screen::Whkd => self.whkd.subscription().map(Message::Whkd),
            Screen::WhkdBinding => self.bindings.subscription().map(Message::Bindings),
            _ => Subscription::none(),
        };

        let worker = if matches!(configuration.config_type, ConfigType::Whkd)
            && (!matches!(configuration.whkd_state, ConfigState::New(_))
                || configuration.saved_new_whkd)
        {
            // Only start the worker if has the config_type as `Whkd` and in case the whkd state is
            // `New` the worker should only run if it has already been saved once at least.
            worker(configuration.path())
        } else {
            Subscription::none()
        };

        Subscription::batch([worker, screen_subscription])
    }

    pub fn discard_changes(&mut self) {
        self.whkdrc = (*self.loaded_whkdrc).clone();
        self.is_dirty = false;
        self.refresh();
    }

    fn check_changes(&mut self) {
        self.is_dirty = self.whkdrc != *self.loaded_whkdrc;
    }

    pub fn load_default(&mut self) {
        // Clear any editing states from bindings
        self.bindings.clear_editing();

        // Set whkdrc to default
        self.whkdrc = DEFAULT_WHKDRC.clone();
        self.loaded_whkdrc = Arc::new(DEFAULT_WHKDRC.clone());
        self.is_dirty = false;

        self.refresh();
    }

    /// Refreshes the current state after some new config as been loaded. This makes sure that each
    /// screen will refresh any internal state to make use of the new loaded config.
    pub fn refresh(&mut self) {
        self.whkd.refresh(&self.whkdrc);
        self.bindings.refresh(&self.whkdrc);
    }

    pub fn load_commands_description(&self) -> Task<Message> {
        Task::batch(self.commands.iter().map(|command| {
            let command_c = command.clone();
            let command_c1 = command.clone();
            Task::future(Compat::new(async move {
                static APP_USER_AGENT: &str =
                    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

                // println!(
                //     "Running GET request for command {} description: {}",
                //     &command_c1, APP_USER_AGENT
                // );
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

#[derive(Debug, Default)]
pub struct WhkdBinary {
    pub found: bool,
    pub running_initial: bool,
    pub running_current: bool,
}

fn find_whkd() -> Task<Message> {
    Task::perform(
        async {
            Command::new("whkd.exe")
                .arg("--version")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .await
        },
        |res| match res {
            Ok(output) => {
                if output.status.success() {
                    Message::WhkdFoundOnPath
                } else {
                    Message::WhkdNotFoundOnPath
                }
            }
            _ => Message::WhkdNotFoundOnPath,
        },
    )
}

fn whkd_status() -> Task<Message> {
    Task::future(async {
        Command::new("tasklist")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("/fi")
            .raw_arg(r#""imagename eq whkd.exe""#)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
    })
    .then(|output| match output {
        Ok(output) => Task::perform(
            async {
                if let Some(stdout) = output.stdout
                    && let Ok(stdout) = stdout.into_stdio().await
                {
                    Command::new("find")
                        .args(["/I", "/N", "/C"])
                        .raw_arg(r#""whkd.exe""#)
                        .stdin(stdout)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .creation_flags(CREATE_NO_WINDOW)
                        .output()
                        .await
                } else {
                    Ok(Output {
                        status: ExitStatus::default(),
                        stdout: Vec::new(),
                        stderr: Vec::new(),
                    })
                }
            },
            |res| match res {
                Ok(output) => {
                    if output.status.success()
                        && String::from_utf8_lossy(&output.stdout).trim() == "1"
                    {
                        Message::WhkdStatus(true)
                    } else {
                        Message::WhkdStatus(false)
                    }
                }
                _ => Message::WhkdStatus(false),
            },
        ),
        Err(_e) => Task::done(Message::WhkdStatus(false)),
    })
}

fn stop_whkd() -> Task<Message> {
    Task::perform(
        async {
            Command::new("taskkill")
                .args(["/f", "/im", "whkd.exe"])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .creation_flags(CREATE_NO_WINDOW)
                .status()
                .await
        },
        |res| match res {
            Ok(status) if status.success() => Message::StoppedWhkd,
            _ => Message::FailedToStopWhkd,
        },
    )
}

fn restart_whkd() -> Task<Message> {
    Task::perform(
        async move {
            Command::new("cmd")
                .args([
                    "/b",
                    "/c",
                    "start",
                    "/b",
                    "powershell.exe",
                    "-NoProfile",
                    "-NoLogo",
                    "-C",
                    "Start-Process",
                    "whkd.exe",
                    "-WindowStyle",
                    "Hidden",
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .creation_flags(CREATE_NO_WINDOW)
                .status()
                .await
        },
        |res| match res {
            Ok(output) => {
                if output.success() {
                    Message::StartedWhkd
                } else {
                    Message::FailedToStartWhkd
                }
            }
            _ => Message::FailedToStartWhkd,
        },
    )
}

pub fn load_commands() -> Task<Message> {
    Task::future(Compat::new(async {
        static APP_USER_AGENT: &str =
            concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

        // println!("Running GET request: {}", APP_USER_AGENT);
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

pub fn worker(path: PathBuf) -> Subscription<Message> {
    Subscription::run_with(path, |path| {
        let path = path.clone();
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

                        let debouncer_res = new_debouncer(Duration::from_millis(250), move |res| {
                            smol::block_on(async {
                                let input = Input::DebouncerRes(res);
                                if let Err(error) = sender.send(input).await {
                                    println!(
                                        "Error sending a debounced event to the worker channel.\n\
                                        E: {error:?}"
                                    );
                                }
                            })
                        });

                        match debouncer_res {
                            Ok(mut debouncer) => {
                                if matches!(std::fs::exists(config_path()), Ok(false) | Err(_)) {
                                    // If the default path doesn't exist, we save the default version to create it
                                    if let Err(apperror) =
                                        save(DEFAULT_WHKDRC.clone(), config_path()).await
                                    {
                                        match output.send(Message::AppError(apperror)).await {
                                            Ok(_) => {}
                                            Err(e) => {
                                                println!("Error trying to send error:\n{e:?}");
                                            }
                                        }
                                    }
                                }

                                match debouncer
                                    .watcher()
                                    .watch(&path, RecursiveMode::NonRecursive)
                                {
                                    Ok(_) => {
                                        state = State::Ready(Data {
                                            debouncer,
                                            receiver,
                                            ignore_event: 0,
                                        });
                                    }
                                    Err(error) => {
                                        if let Err(send_error) = output
                                            .send(Message::FileWatcherError(AppError {
                                                title: String::from(
                                                    "Error trying to watch a whkdrc file",
                                                ),
                                                description: Some(error.to_string()),
                                                kind: AppErrorKind::Error,
                                            }))
                                            .await
                                        {
                                            println!("Error sending an `AppError`: {}", send_error);
                                            println!(
                                                "Actual error it was trying to send: {}",
                                                error
                                            );
                                        }
                                        smol::Timer::after(Duration::from_secs(60)).await;
                                    }
                                }
                            }
                            Err(error) => {
                                if let Err(send_error) = output
                                    .send(Message::FileWatcherError(AppError {
                                        title: String::from(
                                            "Error trying to setup a whkdrc file watcher",
                                        ),
                                        description: Some(error.to_string()),
                                        kind: AppErrorKind::Error,
                                    }))
                                    .await
                                {
                                    println!("Error sending an `AppError`: {}", send_error);
                                    println!("Actual error it was trying to send: {}", error);
                                }
                                smol::Timer::after(Duration::from_secs(60)).await;
                            }
                        }
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
                                            handle_event(
                                                event,
                                                &mut ignore_event,
                                                &mut output,
                                                path.clone(),
                                            );
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
    path: PathBuf,
) {
    // println!("FileWatcher event: {event:?}");
    if let DebouncedEventKind::Any = event.kind {
        if *ignore_event == 0 {
            println!("FileWatcher: loading whkdrc");
            smol::block_on(async {
                match load(path).await {
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

pub fn load_task(path: PathBuf) -> Task<Message> {
    Task::perform(load(path), |res| match res {
        Ok(whkdrc) => Message::LoadedWhkdrc(Arc::new(whkdrc)),
        Err(apperror) => Message::FailedToLoadWhkdrc(apperror),
    })
}

pub async fn load(path: PathBuf) -> Result<Whkdrc, AppError> {
    smol::unblock(move || {
        whkd_parser::load(&path).map_err(|e| AppError {
            title: "Error reading 'whkdrc' file.".into(),
            description: Some(format!("{e:#?}")),
            kind: AppErrorKind::Error,
        })
    })
    .await
}

pub fn save_task(whkdrc: Whkdrc, path: PathBuf) -> Task<Message> {
    Task::future(save(whkdrc, path)).map(|res| match res {
        Ok(_) => Message::SavedWhkdrc,
        Err(apperror) => Message::AppError(apperror),
    })
}

pub async fn save(whkdrc: Whkdrc, path: PathBuf) -> Result<(), AppError> {
    use smol::prelude::*;

    let str = smol::unblock(move || unparser::unparse_whkdrc(&whkdrc)).await;

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

    file.close().await.map_err(|e| AppError {
        title: "Error closing 'whkdrc' file".into(),
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
                .join(".config")
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
