#![allow(dead_code)]
//! Tooltips display a hint of information over some element when hovered.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } }
//! # pub type State = ();
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! use iced::widget::{container, tooltip};
//!
//! enum Message {
//!     // ...
//! }
//!
//! fn view(_state: &State) -> Element<'_, Message> {
//!     tooltip(
//!         "Hover me to display the tooltip!",
//!         container("This is the tooltip contents!")
//!             .padding(10)
//!             .style(container::rounded_box),
//!         tooltip::Position::Bottom,
//!     ).into()
//! }
//! ```
use iced_core as core;

use core::layout::{self, Layout};
use core::mouse;
use core::overlay;
use core::renderer;
use core::text;
use core::time::{Duration, Instant};
use core::widget::{self, Id, Widget};
use core::window;
use core::{
    Background, Color, Element, Event, Length, Padding, Pixels, Point, Rectangle, Shadow, Shell,
    Size, Theme, Vector,
    border::{self, Border},
    theme::palette,
};
use iced::Task;
use iced::widget::container;

/// An element to display a widget over another.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::{container, tooltip};
///
/// enum Message {
///     // ...
/// }
///
/// fn view(_state: &State) -> Element<'_, Message> {
///     tooltip(
///         "Hover me to display the tooltip!",
///         container("This is the tooltip contents!")
///             .padding(10)
///             .style(container::rounded_box),
///     )
///     .position(tooltip::Position::Bottom)
///     .into()
/// }
/// ```
pub struct Tooltip<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Theme: container::Catalog + Catalog,
    Renderer: text::Renderer,
{
    id: Option<Id>,
    content: Element<'a, Message, Theme, Renderer>,
    tooltip: Element<'a, Message, Theme, Renderer>,
    position: Position,
    open: Open,
    disabled: bool,
    gap: f32,
    content_padding: Padding,
    padding: f32,
    snap_within_viewport: bool,
    delay: Duration,
    content_class: <Theme as Catalog>::Class<'a>,
    tooltip_class: <Theme as container::Catalog>::Class<'a>,
    status: Status,
}

/// The default [`Padding`] of a [`Tooltip`] content.
pub(crate) const DEFAULT_CONTENT_PADDING: Padding = Padding {
    top: 5.0,
    bottom: 5.0,
    right: 5.0,
    left: 5.0,
};

impl<'a, Message, Theme, Renderer> Tooltip<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog + Catalog,
    Renderer: text::Renderer,
{
    /// The default padding of a [`Tooltip`] drawn by this renderer.
    const DEFAULT_PADDING: f32 = 0.0;

    /// Creates a new [`Tooltip`].
    ///
    /// [`Tooltip`]: struct.Tooltip.html
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        tooltip: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Tooltip {
            id: None,
            content: content.into(),
            tooltip: tooltip.into(),
            position: Default::default(),
            open: Default::default(),
            disabled: false,
            gap: 0.0,
            content_padding: DEFAULT_CONTENT_PADDING,
            padding: Self::DEFAULT_PADDING,
            snap_within_viewport: true,
            delay: Duration::ZERO,
            content_class: <Theme as Catalog>::default(),
            tooltip_class: <Theme as container::Catalog>::default(),
            status: Default::default(),
        }
    }

    /// Sets the id for the [`Tooltip`].
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the gap between the content and its [`Tooltip`].
    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = gap.into().0;
        self
    }

    /// Sets the padding of the [`Tooltip`] content.
    pub fn content_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.content_padding = padding.into();
        self
    }

    /// Sets the padding of the [`Tooltip`].
    pub fn padding(mut self, padding: impl Into<Pixels>) -> Self {
        self.padding = padding.into().0;
        self
    }

    /// Sets the delay before the [`Tooltip`] is shown.
    ///
    /// Set to [`Duration::ZERO`] to be shown immediately.
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Sets whether the [`Tooltip`] is snapped within the viewport.
    pub fn snap_within_viewport(mut self, snap: bool) -> Self {
        self.snap_within_viewport = snap;
        self
    }

    /// Sets how the [`Tooltip`] is opened.
    pub fn open(mut self, open: Open) -> Self {
        self.open = open;
        self
    }

    /// Sets how the [`Tooltip`] is positioned.
    pub fn position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    /// Disables/enables this [`Tooltip`] to toggle it from showing the tooltip or not.
    pub fn disable(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the style of the [`Tooltip`] content.
    #[must_use]
    pub fn content_style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        <Theme as Catalog>::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.content_class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style of the [`Tooltip`].
    #[must_use]
    pub fn tooltip_style(mut self, style: impl Fn(&Theme) -> container::Style + 'a) -> Self
    where
        <Theme as container::Catalog>::Class<'a>: From<container::StyleFn<'a, Theme>>,
    {
        self.tooltip_class = (Box::new(style) as container::StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Tooltip`] content.
    #[must_use]
    pub fn content_class(mut self, class: impl Into<<Theme as Catalog>::Class<'a>>) -> Self {
        self.content_class = class.into();
        self
    }

    /// Sets the style class of the [`Tooltip`].
    #[must_use]
    pub fn tooltip_class(
        mut self,
        class: impl Into<<Theme as container::Catalog>::Class<'a>>,
    ) -> Self {
        self.tooltip_class = class.into();
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Tooltip<'_, Message, Theme, Renderer>
where
    Theme: container::Catalog + Catalog,
    Renderer: text::Renderer,
{
    fn children(&self) -> Vec<widget::Tree> {
        vec![
            widget::Tree::new(&self.content),
            widget::Tree::new(&self.tooltip),
        ]
    }

    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(&[self.content.as_widget(), self.tooltip.as_widget()]);
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = self.content.as_widget().size();
        layout::padded(
            limits,
            size.width,
            size.height,
            self.content_padding,
            |limits| {
                self.content
                    .as_widget_mut()
                    .layout(&mut tree.children[0], renderer, limits)
            },
        )
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if let Event::Mouse(_) | Event::Window(window::Event::RedrawRequested(_)) = event {
            let state = tree.state.downcast_mut::<State>();
            let now = Instant::now();
            let previous_state = *state;
            let was_idle = matches!(*state, State::Idle { .. });
            let is_over = cursor.is_over(layout.bounds());

            *state = if self.disabled {
                State::default()
            } else if let State::Opened {
                cursor_position,
                over_overlay,
            } = *state
            {
                if over_overlay {
                    *state
                } else {
                    match self.open {
                        Open::Hovered => cursor
                            .position_over(layout.bounds())
                            .map(|_| State::Opened {
                                cursor_position,
                                over_overlay: false,
                            })
                            .unwrap_or_default(),
                        Open::LeftPointer => {
                            if let Event::Mouse(mouse::Event::ButtonPressed(_)) = event {
                                State::default()
                            } else {
                                *state
                            }
                        }
                        Open::RightPointer => {
                            if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) =
                                event
                            {
                                cursor
                                    .position_over(layout.bounds())
                                    .map(|cursor_position| State::Opened {
                                        cursor_position,
                                        over_overlay: false,
                                    })
                                    .unwrap_or_default()
                            } else if let Event::Mouse(mouse::Event::ButtonPressed(
                                mouse::Button::Left,
                            )) = event
                            {
                                State::default()
                            } else {
                                *state
                            }
                        }
                    }
                }
            } else if self.open == Open::Hovered {
                let cursor_position = cursor.position_over(layout.bounds());

                match (*state, cursor_position) {
                    (
                        State::Idle {
                            pressed,
                            hovered_at: None,
                        },
                        Some(cursor_position),
                    ) => {
                        if self.delay == Duration::ZERO {
                            State::Opened {
                                cursor_position,
                                over_overlay: false,
                            }
                        } else {
                            shell.request_redraw_at(now + self.delay);

                            State::Idle {
                                pressed,
                                hovered_at: Some(now),
                            }
                        }
                    }
                    (
                        State::Idle {
                            hovered_at: Some(at),
                            ..
                        },
                        _,
                    ) if at.elapsed() < self.delay => {
                        shell.request_redraw_at(now + self.delay - at.elapsed());
                        *state
                    }
                    (
                        State::Idle {
                            hovered_at: Some(_),
                            ..
                        },
                        Some(cursor_position),
                    ) => {
                        shell.invalidate_layout();
                        State::Opened {
                            cursor_position,
                            over_overlay: false,
                        }
                    }
                    _ => *state,
                }
            } else {
                if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) = event
                    && self.open == Open::LeftPointer
                    && matches!(*state, State::Idle { pressed, .. } if pressed)
                {
                    cursor
                        .position_over(layout.bounds())
                        .map(|cursor_position| State::Opened {
                            cursor_position,
                            over_overlay: false,
                        })
                        .unwrap_or_default()
                } else if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right)) =
                    event
                    && self.open == Open::RightPointer
                    && matches!(*state, State::Idle { pressed, .. } if pressed)
                {
                    cursor
                        .position_over(layout.bounds())
                        .map(|cursor_position| State::Opened {
                            cursor_position,
                            over_overlay: false,
                        })
                        .unwrap_or_default()
                } else if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event
                    && self.open == Open::LeftPointer
                {
                    cursor
                        .position_over(layout.bounds())
                        .map(|_| State::Idle {
                            pressed: true,
                            hovered_at: None,
                        })
                        .unwrap_or_default()
                } else if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) =
                    event
                    && self.open == Open::RightPointer
                {
                    cursor
                        .position_over(layout.bounds())
                        .map(|_| State::Idle {
                            pressed: true,
                            hovered_at: None,
                        })
                        .unwrap_or_default()
                } else {
                    *state
                }
            };

            let is_idle = matches!(*state, State::Idle { .. });

            if was_idle != is_idle {
                shell.invalidate_layout();
                shell.request_redraw();
            } else if self.position == Position::FollowCursor && *state != previous_state {
                shell.request_redraw();
            }

            let previous_status = self.status;
            if self.disabled {
                self.status = Status::Disabled;
            } else {
                match state {
                    State::Idle { pressed, .. } => {
                        let status = if *pressed {
                            match self.open {
                                Open::Hovered => Status::Idle,
                                Open::LeftPointer => Status::LeftPressed,
                                Open::RightPointer => Status::RightPressed,
                            }
                        } else {
                            if is_over {
                                Status::Hovered
                            } else {
                                Status::Idle
                            }
                        };
                        self.status = status;
                    }
                    State::Opened {
                        cursor_position: _,
                        over_overlay: _,
                    } => {
                        self.status = Status::Opened;
                    }
                }
            }
            if !matches!(event, Event::Window(window::Event::RedrawRequested(_)))
                && shell.redraw_request() != window::RedrawRequest::NextFrame
                && previous_status != self.status
            {
                shell.request_redraw();
            }
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let interaction = self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        );

        if cursor.is_over(layout.bounds())
            && matches!(self.open, Open::LeftPointer)
            && self.status != Status::Disabled
            && interaction == mouse::Interaction::None
        {
            mouse::Interaction::Pointer
        } else {
            interaction
        }
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let style = Catalog::style(theme, &self.content_class, self.status);
        let content_layout = layout.children().next().unwrap();

        let bounds = layout.bounds();

        if style.background.is_some() || style.border.width > 0.0 || style.shadow.color.a > 0.0 {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: style.border,
                    shadow: style.shadow,
                    snap: style.snap,
                },
                style
                    .background
                    .unwrap_or(Background::Color(Color::TRANSPARENT)),
            );
        }

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style {
                text_color: style.text_color,
            },
            content_layout,
            cursor,
            viewport,
        );
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let state = tree.state.downcast_mut::<State>();

        operation.container(self.id.as_ref(), layout.bounds());
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
        operation.custom(self.id.as_ref(), layout.bounds(), state);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_mut::<State>();

        let mut children = tree.children.iter_mut();

        let content = self.content.as_widget_mut().overlay(
            children.next().unwrap(),
            layout,
            renderer,
            viewport,
            translation,
        );

        let tooltip = match *state {
            State::Idle { .. } => None,
            State::Opened {
                cursor_position,
                over_overlay: _,
            } => Some(overlay::Element::new(Box::new(Overlay {
                position: layout.position() + translation,
                tooltip: &mut self.tooltip,
                state: children.next().unwrap(),
                tooltip_state: state,
                cursor_position,
                content_bounds: layout.bounds(),
                snap_within_viewport: self.snap_within_viewport,
                positioning: self.position,
                gap: self.gap,
                padding: self.padding,
                class: &self.tooltip_class,
            }))),
        };

        if content.is_some() && tooltip.is_some() {
            Some(
                overlay::Group::with_children(content.into_iter().chain(tooltip).collect())
                    .overlay(),
            )
        } else if tooltip.is_some() {
            tooltip
        } else {
            None
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Tooltip<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: container::Catalog + Catalog + 'a,
    Renderer: text::Renderer + 'a,
{
    fn from(
        tooltip: Tooltip<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(tooltip)
    }
}

/// The position of the tooltip. Defaults to following the cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Position {
    /// The tooltip will appear on the top of the widget.
    Top,
    /// The tooltip will appear on the bottom of the widget.
    Bottom,
    /// The tooltip will appear on the left of the widget.
    Left,
    /// The tooltip will appear on the right of the widget.
    Right,
    /// The tooltip will follow the cursor.
    #[default]
    FollowCursor,
    /// The tooltip will appear aligned to the top right of the widget.
    TopRight,
    /// The tooltip will appear aligned to the top left of the widget.
    TopLeft,
    /// The tooltip will appear aligned to the bottom right of the widget.
    BottomRight,
    /// The tooltip will appear aligned to the bottom left of the widget.
    BottomLeft,
}

/// How should the tooltip open. Defaults to hovered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Open {
    /// The tooltip will appear when hovered.
    #[default]
    Hovered,
    /// The tooltip will appear when pressing left pointer on it.
    LeftPointer,
    /// The tooltip will appear when pressing right pointer on it.
    RightPointer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Idle {
        pressed: bool,
        hovered_at: Option<Instant>,
    },
    Opened {
        cursor_position: Point,
        over_overlay: bool,
    },
}

impl Default for State {
    fn default() -> Self {
        State::Idle {
            pressed: false,
            hovered_at: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Status {
    #[default]
    /// The [`Tooltip`] content can be hovered/pressed to show the tooltip.
    Idle,
    /// The [`Tooltip`] content is being hovered.
    Hovered,
    /// The [`Tooltip`] content is being pressed with left pointer.
    LeftPressed,
    /// The [`Tooltip`] content is being pressed with right pointer.
    RightPressed,
    /// The [`Tooltip`] is opened.
    Opened,
    /// The [`Tooltip`] won't show.
    Disabled,
}

/// The style of the [`Tooltip`] content.
///
/// If not specified with [`Tooltip::style`]
/// the theme will provide the style.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The [`Background`] of the button.
    pub background: Option<Background>,
    /// The text [`Color`] of the button.
    pub text_color: Color,
    /// The [`Border`] of the button.
    pub border: Border,
    /// The [`Shadow`] of the button.
    pub shadow: Shadow,
    /// Whether the tooltip content should be snapped to the pixel grid.
    pub snap: bool,
}

impl Style {
    /// Updates the [`Style`] with the given [`Background`].
    pub fn with_background(self, background: impl Into<Background>) -> Self {
        Self {
            background: Some(background.into()),
            ..self
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background: None,
            text_color: Color::BLACK,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: true,
        }
    }
}

/// The theme catalog of a [`Tooltip`] content.
///
/// All themes that can be used with [`Tooltip`]
/// must implement this trait.
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

/// A styling function for a [`Tooltip`] content.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(text)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// A primary button; denoting a main action.
pub fn primary(theme: &Theme, status: Status) -> Style {
    let palette = theme.palette();
    let base = styled(palette.primary.base);

    match status {
        Status::Idle | Status::LeftPressed | Status::RightPressed => base,
        Status::Hovered | Status::Opened => Style {
            background: Some(Background::Color(palette.primary.strong.color)),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A secondary button; denoting a complementary action.
pub fn secondary(theme: &Theme, status: Status) -> Style {
    let palette = theme.palette();
    let base = styled(palette.secondary.base);

    match status {
        Status::Idle | Status::LeftPressed | Status::RightPressed => base,
        Status::Hovered | Status::Opened => Style {
            background: Some(Background::Color(palette.secondary.strong.color)),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A success button; denoting a good outcome.
pub fn success(theme: &Theme, status: Status) -> Style {
    let palette = theme.palette();
    let base = styled(palette.success.base);

    match status {
        Status::Idle | Status::LeftPressed | Status::RightPressed => base,
        Status::Hovered | Status::Opened => Style {
            background: Some(Background::Color(palette.success.strong.color)),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A warning button; denoting a risky action.
pub fn warning(theme: &Theme, status: Status) -> Style {
    let palette = theme.palette();
    let base = styled(palette.warning.base);

    match status {
        Status::Idle | Status::LeftPressed | Status::RightPressed => base,
        Status::Hovered | Status::Opened => Style {
            background: Some(Background::Color(palette.warning.strong.color)),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A danger button; denoting a destructive action.
pub fn danger(theme: &Theme, status: Status) -> Style {
    let palette = theme.palette();
    let base = styled(palette.danger.base);

    match status {
        Status::Idle | Status::LeftPressed | Status::RightPressed => base,
        Status::Hovered | Status::Opened => Style {
            background: Some(Background::Color(palette.danger.strong.color)),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A text button; useful for links.
pub fn text(theme: &Theme, status: Status) -> Style {
    let palette = theme.palette();

    let base = Style {
        text_color: palette.background.base.text,
        ..Style::default()
    };

    match status {
        Status::Idle | Status::LeftPressed | Status::RightPressed => base,
        Status::Hovered | Status::Opened => Style {
            text_color: palette.background.base.text.scale_alpha(0.8),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A button using background shades.
pub fn background(theme: &Theme, status: Status) -> Style {
    let palette = theme.palette();
    let base = styled(palette.background.base);

    match status {
        Status::Idle => base,
        Status::LeftPressed | Status::RightPressed => Style {
            background: Some(Background::Color(palette.background.strong.color)),
            ..base
        },
        Status::Hovered | Status::Opened => Style {
            background: Some(Background::Color(palette.background.weak.color)),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A subtle button using weak background shades.
pub fn subtle(theme: &Theme, status: Status) -> Style {
    let palette = theme.palette();
    let base = styled(palette.background.weakest);

    match status {
        Status::Idle => base,
        Status::LeftPressed | Status::RightPressed => Style {
            background: Some(Background::Color(palette.background.strong.color)),
            ..base
        },
        Status::Hovered | Status::Opened => Style {
            background: Some(Background::Color(palette.background.weaker.color)),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

fn styled(pair: palette::Pair) -> Style {
    Style {
        background: Some(Background::Color(pair.color)),
        text_color: pair.text,
        border: border::rounded(2),
        ..Style::default()
    }
}

fn disabled(style: Style) -> Style {
    Style {
        background: style
            .background
            .map(|background| background.scale_alpha(0.5)),
        text_color: style.text_color.scale_alpha(0.5),
        ..style
    }
}

struct Overlay<'a, 'b, Message, Theme, Renderer>
where
    Theme: container::Catalog,
    Renderer: text::Renderer,
{
    position: Point,
    tooltip: &'b mut Element<'a, Message, Theme, Renderer>,
    state: &'b mut widget::Tree,
    tooltip_state: &'b mut State,
    cursor_position: Point,
    content_bounds: Rectangle,
    snap_within_viewport: bool,
    positioning: Position,
    gap: f32,
    padding: f32,
    class: &'b Theme::Class<'a>,
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Overlay<'_, '_, Message, Theme, Renderer>
where
    Theme: container::Catalog,
    Renderer: text::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let viewport = Rectangle::with_size(bounds);

        let tooltip_layout = self.tooltip.as_widget_mut().layout(
            self.state,
            renderer,
            &layout::Limits::new(
                Size::ZERO,
                if self.snap_within_viewport {
                    viewport.size()
                } else {
                    Size::INFINITE
                },
            )
            .shrink(Padding::new(self.padding)),
        );

        let text_bounds = tooltip_layout.bounds();
        let x_center = self.position.x + (self.content_bounds.width - text_bounds.width) / 2.0;
        let y_center = self.position.y + (self.content_bounds.height - text_bounds.height) / 2.0;

        let mut tooltip_bounds = {
            let offset = match self.positioning {
                Position::Top => Vector::new(
                    x_center,
                    self.position.y - text_bounds.height - self.gap - self.padding,
                ),
                Position::Bottom => Vector::new(
                    x_center,
                    self.position.y + self.content_bounds.height + self.gap + self.padding,
                ),
                Position::Left => Vector::new(
                    self.position.x - text_bounds.width - self.gap - self.padding,
                    y_center,
                ),
                Position::Right => Vector::new(
                    self.position.x + self.content_bounds.width + self.gap + self.padding,
                    y_center,
                ),
                Position::FollowCursor => {
                    let translation = self.position - self.content_bounds.position();

                    Vector::new(
                        self.cursor_position.x,
                        self.cursor_position.y - text_bounds.height,
                    ) + translation
                }
                Position::TopRight => Vector::new(
                    self.position.x + (self.content_bounds.width - text_bounds.width),
                    self.position.y - text_bounds.height - self.gap - self.padding,
                ),
                Position::TopLeft => Vector::new(
                    self.position.x,
                    self.position.y - text_bounds.height - self.gap - self.padding,
                ),
                Position::BottomRight => Vector::new(
                    self.position.x + (self.content_bounds.width - text_bounds.width),
                    self.position.y + self.content_bounds.height + self.gap + self.padding,
                ),
                Position::BottomLeft => Vector::new(
                    self.position.x,
                    self.position.y + self.content_bounds.height + self.gap + self.padding,
                ),
            };

            Rectangle {
                x: offset.x - self.padding,
                y: offset.y - self.padding,
                width: text_bounds.width + self.padding * 2.0,
                height: text_bounds.height + self.padding * 2.0,
            }
        };

        if self.snap_within_viewport {
            if tooltip_bounds.x < viewport.x {
                tooltip_bounds.x = viewport.x;
            } else if viewport.x + viewport.width < tooltip_bounds.x + tooltip_bounds.width {
                tooltip_bounds.x = viewport.x + viewport.width - tooltip_bounds.width;
            }

            if tooltip_bounds.y < viewport.y {
                tooltip_bounds.y = viewport.y;
            } else if viewport.y + viewport.height < tooltip_bounds.y + tooltip_bounds.height {
                tooltip_bounds.y = viewport.y + viewport.height - tooltip_bounds.height;
            }
        }

        layout::Node::with_children(
            tooltip_bounds.size(),
            vec![tooltip_layout.translate(Vector::new(self.padding, self.padding))],
        )
        .translate(Vector::new(tooltip_bounds.x, tooltip_bounds.y))
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
    ) {
        if let Event::Mouse(_) | Event::Window(window::Event::RedrawRequested(_)) = event {
            let state = &mut *self.tooltip_state;
            let previous_state = *state;
            let was_idle = matches!(*state, State::Idle { .. });

            *state = cursor
                .position_over(layout.bounds())
                .map(|_| State::Opened {
                    cursor_position: if let State::Opened {
                        cursor_position, ..
                    } = *state
                    {
                        cursor_position
                    } else {
                        Point::default()
                    },
                    over_overlay: true,
                })
                .unwrap_or(State::Opened {
                    cursor_position: if let State::Opened {
                        cursor_position, ..
                    } = *state
                    {
                        cursor_position
                    } else {
                        Point::default()
                    },
                    over_overlay: false,
                });

            let is_idle = matches!(*state, State::Idle { .. });

            if was_idle != is_idle {
                shell.invalidate_layout();
                shell.request_redraw();
            } else if self.positioning == Position::FollowCursor && *state != previous_state {
                shell.request_redraw();
            }
        }

        self.tooltip.as_widget_mut().update(
            self.state,
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            shell,
            &Rectangle::with_size(Size::INFINITE),
        );

        if cursor.is_over(layout.bounds()) && matches!(event, Event::Mouse(_)) {
            shell.capture_event();
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
    ) {
        let style = theme.style(self.class);

        container::draw_background(renderer, &style, layout.bounds());

        let defaults = renderer::Style {
            text_color: style.text_color.unwrap_or(inherited_style.text_color),
        };

        self.tooltip.as_widget().draw(
            self.state,
            renderer,
            theme,
            &defaults,
            layout.children().next().unwrap(),
            cursor_position,
            &Rectangle::with_size(Size::INFINITE),
        );
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let interaction = self.tooltip.as_widget().mouse_interaction(
            self.state,
            layout.children().next().unwrap(),
            cursor,
            &Rectangle::with_size(Size::INFINITE),
            renderer,
        );

        if cursor.is_over(layout.bounds()) && interaction == mouse::Interaction::None {
            mouse::Interaction::Idle
        } else {
            interaction
        }
    }
}

fn close_all_operation<T>() -> impl widget::Operation<T> {
    struct Close;

    impl<T> widget::Operation<T> for Close {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn widget::Operation<T>)) {
            operate(self)
        }

        fn custom(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn std::any::Any) {
            if let Some(state) = state.downcast_mut::<State>() {
                *state = State::default();
            }
        }
    }

    Close
}

fn close_operation<T>(id: Id) -> impl widget::Operation<T> {
    struct Close {
        target: Id,
    }

    impl<T> widget::Operation<T> for Close {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn widget::Operation<T>)) {
            operate(self)
        }

        fn custom(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn std::any::Any) {
            if id == Some(&self.target)
                && let Some(state) = state.downcast_mut::<State>()
            {
                *state = State::default();
            }
        }
    }

    Close { target: id }
}

pub fn close_all<T: Send + 'static>() -> Task<T> {
    iced::advanced::widget::operate(close_all_operation::<T>()).discard()
}

pub fn close<T: Send + 'static>(id: impl Into<Id>) -> Task<T> {
    iced::advanced::widget::operate(close_operation::<T>(id.into())).discard()
}

pub fn tooltip<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    tooltip: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Tooltip<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog + Catalog,
    Renderer: text::Renderer,
{
    Tooltip::new(content, tooltip)
}
