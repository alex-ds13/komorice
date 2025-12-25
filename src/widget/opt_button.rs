#![allow(deprecated)]
use crate::widget::opt_helpers::{
    DisableArgs, disable_checkbox, label_element_with_description, opt_box, reset_button,
};
use iced::{
    Center, Element, Renderer, padding,
    widget::{Component, component, mouse_area, row, text},
};

pub struct OptButton<'a, Message, F, I>
where
    F: Fn(bool) -> I,
    I: Into<Element<'a, Message>>,
{
    name: text::Fragment<'a>,
    description: Option<&'a str>,
    on_press: Option<Message>,
    is_dirty: bool,
    reset_message: Option<Message>,
    disable_args: Option<DisableArgs<'a, Message>>,
    element: Option<F>,
}

impl<'a, Message, F, I> OptButton<'a, Message, F, I>
where
    F: Fn(bool) -> I,
    I: Into<Element<'a, Message>>,
{
    /// Create a new `OptButton` with the specified name as a title
    pub fn new(name: impl text::IntoFragment<'a>) -> Self {
        Self {
            name: name.into_fragment(),
            description: None,
            on_press: None,
            is_dirty: false,
            reset_message: None,
            disable_args: None,
            element: None,
        }
    }

    /// Create a new `OptButton` with all fields specified directly
    pub fn with(
        name: impl text::IntoFragment<'a>,
        description: Option<&'a str>,
        on_press: Message,
        element: F,
    ) -> Self {
        Self::new(name)
            .description(description)
            .on_press(on_press)
            .element(element)
    }

    /// Sets the description to be shown on this `OptButton`
    pub fn description(mut self, description: Option<&'a str>) -> Self {
        self.description = description;
        self
    }

    /// Sets the `on_press` message to be sent when this `OptButton` is pressed
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Sets the `element` to be shown on the right of this `OptButton` according
    /// to its internal hover state. This `element` is a function/closure that takes
    /// in an 'hovered' bool and returns an `Element`.
    pub fn element(mut self, element: F) -> Self {
        self.element = Some(element);
        self
    }

    /// Sets the dirtiness of this `OptButton`.
    pub fn dirty(mut self, is_dirty: bool) -> Self {
        self.is_dirty = is_dirty;
        self
    }

    /// Sets the `reset_message` to be sent when this `OptButton`'s reset button is
    /// pressed.
    pub fn reset_message(mut self, message: Option<Message>) -> Self {
        self.reset_message = message;
        self
    }

    /// Sets the `disable_args` of this `OptButton`.
    pub fn disable_args(mut self, disable_args: Option<DisableArgs<'a, Message>>) -> Self {
        self.disable_args = disable_args;
        self
    }
}

#[derive(Debug, Default)]
pub struct State {
    is_hovered: bool,
}

#[derive(Clone, Debug, Default)]
pub enum InternalMessage<Message> {
    #[default]
    None,
    SetHovered(bool),
    Message(Message),
}

impl<'a, Message, F, I> Component<'a, Message> for OptButton<'a, Message, F, I>
where
    Message: Clone + 'static,
    F: Fn(bool) -> I,
    I: Into<Element<'a, Message>>,
{
    type State = State;

    type Event = InternalMessage<Message>;

    fn update(
        &mut self,
        state: &mut Self::State,
        event: Self::Event,
        _renderer: &Renderer,
    ) -> Option<Message> {
        match event {
            InternalMessage::None => {}
            InternalMessage::SetHovered(hover) => state.is_hovered = hover,
            InternalMessage::Message(message) => return Some(message),
        }
        None
    }

    fn view(&self, state: &Self::State) -> Element<'a, Self::Event> {
        let label = if self.is_dirty {
            row![
                text(self.name.clone()),
                Element::from(reset_button(self.reset_message.clone()))
                    .map(InternalMessage::Message)
            ]
            .spacing(5)
            .height(30)
            .align_y(Center)
        } else {
            row![text(self.name.clone())].height(30).align_y(Center)
        };
        let main = row![label_element_with_description(label, self.description)]
            .push(
                disable_checkbox(self.disable_args.as_ref())
                    .map(|el| el.map(InternalMessage::Message)),
            )
            .push(
                self.element
                    .as_ref()
                    .map(|f| f(state.is_hovered).into().map(InternalMessage::Message)),
            )
            .align_y(Center)
            .spacing(10)
            .padding(padding::right(10));

        let is_disabled = self
            .disable_args
            .as_ref()
            .map(|da| da.disable)
            .unwrap_or_default();
        let interaction = if is_disabled {
            iced::mouse::Interaction::Idle
        } else {
            iced::mouse::Interaction::Pointer
        };
        let mut area = mouse_area(opt_box(main))
            .interaction(interaction)
            .on_enter(InternalMessage::SetHovered(true))
            .on_exit(InternalMessage::SetHovered(false));

        if let Some(on_press) = &self.on_press {
            area = area.on_press(InternalMessage::Message(on_press.clone()));
        }

        area.into()
    }
}

impl<'a, Message, F, I> From<OptButton<'a, Message, F, I>> for Element<'a, Message>
where
    Message: Clone + 'static,
    F: Fn(bool) -> I + 'a,
    I: Into<Element<'a, Message>> + 'a,
{
    fn from(value: OptButton<'a, Message, F, I>) -> Self {
        component(value)
    }
}

pub fn opt_button<'a, Message, F, I>(
    name: impl text::IntoFragment<'a>,
) -> OptButton<'a, Message, F, I>
where
    Message: Clone + 'static,
    F: Fn(bool) -> I + 'a,
    I: Into<Element<'a, Message>> + 'a,
{
    OptButton::new(name)
}
