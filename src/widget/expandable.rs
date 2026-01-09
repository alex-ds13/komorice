#![allow(deprecated)]
use crate::widget::opt_helpers::{
    DisableArgs, disable_checkbox, label_element_with_description, opt_box, opt_box_style_bottom,
    opt_box_style_top, reset_button,
};
use iced::{
    Center, Element, Renderer, padding,
    widget::{Column, Component, column, component, mouse_area, row, rule, text},
};

pub struct Expandable<'a, Message, F, E, I, G>
where
    F: Fn(bool, bool) -> E,
    E: Into<Element<'a, Message>>,
    I: IntoIterator<Item = Element<'a, Message>>,
    G: Fn(bool) -> Message + Clone + 'a,
{
    name: Option<text::Fragment<'a>>,
    description: Option<&'a str>,
    // top_element: F,
    // children: Box<dyn IntoIterator<Item = Element<'a, Message>>>,
    on_press: Option<Message>,
    on_default: Option<Message>,
    is_dirty: bool,
    force_expand: bool,
    disable_args: Option<DisableArgs<'a, Message, G>>,
    top_element: F,
    bottom_elements: Option<Box<dyn Fn() -> I + 'a>>,
}

impl<'a, Message, F, E, I, G> Expandable<'a, Message, F, E, I, G>
where
    F: Fn(bool, bool) -> E,
    E: Into<Element<'a, Message>>,
    I: IntoIterator<Item = Element<'a, Message>>,
    G: Fn(bool) -> Message + Clone + 'a,
{
    /// Create a new `Expandable` with the specified top element function/closure that takes
    /// in an 'hovered' bool and returns an `Element`.
    pub fn new(top_element: F) -> Self {
        Self {
            name: None,
            description: None,
            on_press: None,
            on_default: None,
            is_dirty: false,
            force_expand: false,
            disable_args: None,
            top_element,
            bottom_elements: None,
        }
    }

    /// Create a new `Expandable` with all fields specified directly
    #[allow(clippy::too_many_arguments)]
    pub fn with(
        name: Option<impl text::IntoFragment<'a>>,
        description: Option<&'a str>,
        on_press: Option<Message>,
        on_default: Option<Message>,
        is_dirty: bool,
        force_expand: bool,
        disable_args: Option<DisableArgs<'a, Message, G>>,
        top_element: F,
        bottom_elements: impl Fn() -> I + 'a,
    ) -> Self {
        Self::new(top_element)
            .name(name)
            .description(description)
            .on_press_maybe(on_press)
            .on_default_maybe(on_default)
            .dirty(is_dirty)
            .force_expand(force_expand)
            .disable_args_maybe(disable_args)
            .bottom_elements(bottom_elements)
    }

    /// Sets the name to be shown on this `Expandable`
    pub fn name(mut self, name: Option<impl text::IntoFragment<'a>>) -> Self {
        self.name = name.map(text::IntoFragment::into_fragment);
        self
    }

    /// Sets the description to be shown on this `Expandable`
    pub fn description(mut self, description: Option<&'a str>) -> Self {
        self.description = description;
        self
    }

    /// Sets the `on_press` message to be sent when this `Expandable` is pressed
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Sets the `on_press` message to be sent when this `Expandable` is pressed if message
    /// is 'Some', otherwise it sets it to 'None'.
    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }

    /// Sets the `on_default` message to be sent when this `Expandable` is reset to default
    pub fn on_default(self, message: Message) -> Self {
        self.on_default_maybe(Some(message))
    }

    /// Sets the `on_default` message to be sent when this `Expandable` is reset to default,
    /// if message is 'Some', otherwise it sets it to 'None'.
    pub fn on_default_maybe(mut self, message: Option<Message>) -> Self {
        self.on_default = message;
        self
    }

    pub fn dirty(mut self, is_dirty: bool) -> Self {
        self.is_dirty = is_dirty;
        self
    }

    pub fn force_expand(mut self, force_expand: bool) -> Self {
        self.force_expand = force_expand;
        self
    }

    pub fn disable_args(self, disable_args: DisableArgs<'a, Message, G>) -> Self {
        self.disable_args_maybe(Some(disable_args))
    }

    pub fn disable_args_maybe(mut self, disable_args: Option<DisableArgs<'a, Message, G>>) -> Self {
        self.disable_args = disable_args;
        self
    }

    /// Sets the `bottom_element` to be shown on this `Expandable`, it is a function/closure
    /// that returns an `Element`.
    pub fn bottom_elements(mut self, elements: impl Fn() -> I + 'a) -> Self {
        self.bottom_elements = Some(Box::new(elements));
        self
    }
}

#[derive(Debug, Default)]
pub struct State {
    is_hovered: bool,
    is_expanded: bool,
}

#[derive(Clone, Debug, Default)]
pub enum InternalMessage<Message> {
    #[default]
    None,
    SetHovered(bool),
    ToggleExpand,
    Message(Message),
}

impl<'a, Message, F, E, I, G> Component<'a, Message> for Expandable<'a, Message, F, E, I, G>
where
    Message: Clone + 'static,
    F: Fn(bool, bool) -> E,
    E: Into<Element<'a, Message>>,
    I: IntoIterator<Item = Element<'a, Message>>,
    G: Fn(bool) -> Message + Clone + 'a,
{
    type State = State;

    type Event = InternalMessage<Message>;

    fn size_hint(&self) -> iced::Size<iced::Length> {
        iced::Size {
            width: iced::Fill,
            height: iced::Shrink,
        }
    }

    fn update(
        &mut self,
        state: &mut Self::State,
        event: Self::Event,
        _renderer: &Renderer,
    ) -> Option<Message> {
        match event {
            InternalMessage::None => {}
            InternalMessage::SetHovered(hover) => state.is_hovered = hover,
            InternalMessage::ToggleExpand => {
                if self.force_expand {
                    // If this expandable is being forcefully expanded, then we shouldn't change the
                    // internal `is_expanded` state, so that when it stops being forced it goes back
                    // to the previous state.
                    return None;
                }
                state.is_expanded = !state.is_expanded;
                if let Some(on_press) = &self.on_press {
                    return Some(on_press.clone());
                }
            }
            InternalMessage::Message(message) => return Some(message),
        }
        None
    }

    fn view(&self, state: &Self::State) -> Element<'a, Self::Event> {
        let main = if let Some(name) = &self.name {
            let label = if self.is_dirty {
                row![text(name.clone())]
                    .push(
                        self.on_default
                            .as_ref()
                            .map(|d| reset_button(Some(InternalMessage::Message(d.clone())))),
                    )
                    .spacing(5)
                    .height(30)
                    .align_y(Center)
            } else {
                row![text(name.clone())].height(30).align_y(Center)
            };
            row![label_element_with_description(label, self.description)]
                .push(
                    disable_checkbox(self.disable_args.as_ref())
                        .map(|el| el.map(InternalMessage::Message)),
                )
                .push(
                    (self.top_element)(state.is_hovered, state.is_expanded || self.force_expand)
                        .into()
                        .map(InternalMessage::Message),
                )
                .align_y(Center)
                .padding(padding::right(10))
                .spacing(10)
        } else {
            row![
                (self.top_element)(state.is_hovered, state.is_expanded || self.force_expand)
                    .into()
                    .map(InternalMessage::Message),
            ]
            .align_y(Center)
            .padding(padding::right(10))
        };

        let area = |el| -> Element<_> {
            mouse_area(el)
                .interaction(iced::mouse::Interaction::Pointer)
                .on_press(InternalMessage::ToggleExpand)
                .on_enter(InternalMessage::SetHovered(true))
                .on_exit(InternalMessage::SetHovered(false))
                .into()
        };

        // let disable_area = |el| {
        //     mouse_area(container(el).width(Fill).height(Fill))
        //         .on_press(on_default)
        //         .interaction(iced::mouse::Interaction::NotAllowed)
        // };

        if self.bottom_elements.as_ref().is_some() && (state.is_expanded || self.force_expand) {
            let children = (self.bottom_elements.as_ref().unwrap())()
                .into_iter()
                .map(|e| e.map(InternalMessage::Message))
                .collect::<Vec<_>>();
            let (wrapped_top, wrapped_inner) = if !children.is_empty() {
                let top = opt_box(main).style(opt_box_style_top);
                let wrapped_top = area(top);
                let inner = Column::with_children(children)
                    .spacing(10)
                    .padding(padding::all(10).left(20));
                let wrapped_inner = opt_box(inner).style(opt_box_style_bottom);
                // let wrapped_inner: Element<Message> = if should_disable {
                //     iced::widget::stack([wrapped_inner.into(), disable_area("").into()]).into()
                // } else {
                //     wrapped_inner.into()
                // };
                (wrapped_top, Some(wrapped_inner))
            } else {
                (area(opt_box(main)), None)
            };
            column![]
                .push(wrapped_top)
                .push(wrapped_inner.is_some().then_some(rule::horizontal(2.0)))
                .push(wrapped_inner)
                .into()
        } else {
            column![area(opt_box(main))].into()
        }
    }
}

impl<'a, Message, F, E, I, G> From<Expandable<'a, Message, F, E, I, G>> for Element<'a, Message>
where
    Message: Clone + 'static,
    F: Fn(bool, bool) -> E + 'a,
    E: Into<Element<'a, Message>> + 'a,
    I: IntoIterator<Item = Element<'a, Message>> + 'a,
    G: Fn(bool) -> Message + Clone + 'a,
{
    fn from(value: Expandable<'a, Message, F, E, I, G>) -> Self {
        component(value)
    }
}

pub fn expandable<'a, Message, F, E, I, G>(top_element: F) -> Expandable<'a, Message, F, E, I, G>
where
    Message: Clone + 'a,
    F: Fn(bool, bool) -> E + 'a,
    E: Into<Element<'a, Message>> + 'a,
    I: IntoIterator<Item = Element<'a, Message>> + 'a,
    G: Fn(bool) -> Message + Clone + 'a,
{
    Expandable::new(top_element)
}
