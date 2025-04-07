use super::{ConfigType, Screen};

use crate::SCREENS_TO_RESET;

use iced::{
    padding,
    widget::{button, column, container, horizontal_rule, scrollable, Container, Space},
    Element,
    Length::{Fill, Shrink},
    Task,
};

#[derive(Clone, Debug)]
pub enum Message {
    SetHomeScreen,
    SelectScreen(Screen),
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
    SetHomeScreen,
    UpdateMainScreen(Screen),
}

#[derive(Debug, Clone)]
pub struct Sidebar {
    pub komorebi_selected: Screen,
    pub whkd_selected: Screen,
    pub config_type: ConfigType,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self {
            komorebi_selected: Screen::General,
            whkd_selected: Screen::Whkd,
            config_type: ConfigType::Komorebi,
        }
    }
}

impl Sidebar {
    pub fn update(&mut self, message: Message) -> (Action, Task<Message>) {
        match message {
            Message::SetHomeScreen => {
                return (Action::SetHomeScreen, Task::none());
            }
            Message::SelectScreen(screen) => match self.config_type {
                ConfigType::Komorebi => {
                    if screen != self.komorebi_selected || SCREENS_TO_RESET.contains(&screen) {
                        self.komorebi_selected = screen.clone();
                        return (Action::UpdateMainScreen(screen), Task::none());
                    }
                }
                ConfigType::Whkd => {
                    if screen != self.whkd_selected || SCREENS_TO_RESET.contains(&screen) {
                        self.whkd_selected = screen.clone();
                        return (Action::UpdateMainScreen(screen), Task::none());
                    }
                }
            },
        }
        (Action::None, Task::none())
    }

    pub fn view(&self) -> Element<Message> {
        let home = container(
            button(Screen::Home)
                .on_press(Message::SetHomeScreen)
                .style(button::secondary)
                .width(Fill),
        )
        .width(Fill);
        let screen_buttons = self.get_screens();
        let fixed_width = Space::new(120, Shrink);
        let main_content = scrollable(
            column![fixed_width]
                .extend(screen_buttons)
                .spacing(10)
                .padding(padding::right(15).left(5).bottom(0))
                .width(Shrink),
        )
        .height(Fill);
        let fixed_width_wider = Space::new(135, Shrink);
        let fixed_width = Space::new(120, Shrink);
        let bottom_content = column![
            fixed_width_wider,
            container(horizontal_rule(2.0)).padding(padding::bottom(5)),
            column![fixed_width, home].width(Shrink),
        ]
        .width(Shrink)
        .padding(padding::left(5));

        column![main_content, bottom_content].into()
    }

    pub fn selected_screen(&self) -> Screen {
        match self.config_type {
            ConfigType::Komorebi => self.komorebi_selected.clone(),
            ConfigType::Whkd => self.whkd_selected.clone(),
        }
    }

    fn get_screens(&self) -> Vec<Element<Message>> {
        match self.config_type {
            ConfigType::Komorebi => [
                Screen::General,
                Screen::Monitors,
                Screen::Border,
                Screen::Stackbar,
                Screen::Transparency,
                Screen::Animations,
                Screen::Theme,
                Screen::Rules,
                Screen::LiveDebug,
                Screen::Settings,
            ]
            .into_iter()
            .map(|s| screen_button(s, &self.komorebi_selected).into())
            .collect(),
            ConfigType::Whkd => [Screen::Whkd, Screen::Settings]
                .into_iter()
                .map(|s| screen_button(s, &self.whkd_selected).into())
                .collect(),
        }
    }
}

fn screen_button(screen: Screen, selected: &Screen) -> Container<Message> {
    let is_selected = &screen == selected;
    container(
        button(&screen)
            .on_press(Message::SelectScreen(screen))
            .style(move |t, s| {
                if is_selected {
                    button::primary(t, s)
                } else {
                    button::secondary(t, s)
                }
            })
            .width(Fill),
    )
    .width(Fill)
}
