// This widget is a modification of the original `TextInput` widget from [`iced`]
//
// [`iced`]: https://github.com/iced-rs/iced
//
// Copyright 2019 Héctor Ramón, Iced contributors
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
// COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

mod cursor;
mod editor;
mod value;

use std::fmt::Display;
use std::str::FromStr;

use cursor::Cursor;
use editor::Editor;
use value::Value;

use iced::advanced::clipboard::{self, Clipboard};
use iced::advanced::layout::{self, Limits};
use iced::advanced::mouse::{self, click};
use iced::advanced::renderer;
use iced::advanced::text::paragraph::{self, Paragraph as _};
use iced::advanced::text::{self, Text};
use iced::advanced::widget;
use iced::advanced::widget::operation::{self, Operation};
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{Layout, Shell, Widget};
use iced::alignment;
use iced::keyboard;
use iced::keyboard::key;
use iced::time::{Duration, Instant};
use iced::touch;
use iced::widget::{button, Button};
use iced::window;
use iced::{
    border::{self, Border},
    Background, Color, Element, Event, Length, Padding, Pixels, Point, Rectangle, Size, Task,
    Theme, Vector,
};
use iced_core::input_method::{self, InputMethod};
use num_traits::{Bounded, Num, NumAssignOps};

/// A field that can be filled with numbers.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// #
/// use iced::widget::number_input;
///
/// struct State {
///    content: i32,
/// }
///
/// #[derive(Debug, Clone)]
/// enum Message {
///     ContentChanged(i32)
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     number_input("Type some number here...", &state.content)
///         .on_input(Message::ContentChanged)
///         .into()
/// }
///
/// fn update(state: &mut State, message: Message) {
///     match message {
///         Message::ContentChanged(content) => {
///             state.content = content;
///         }
///     }
/// }
/// ```
#[allow(missing_debug_implementations)]
pub struct NumberInput<'a, T, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog + button::Catalog + iced::widget::text::Catalog,
    Renderer: text::Renderer,
{
    id: Option<Id>,
    placeholder: String,
    value: Value,
    _number_value: T,
    max: T,
    min: T,
    is_secure: bool,
    font: Option<Renderer::Font>,
    width: Length,
    padding: Padding,
    size: Option<Pixels>,
    line_height: text::LineHeight,
    alignment: alignment::Horizontal,
    on_input: Option<Box<dyn Fn(T) -> Message + 'a>>,
    on_paste: Option<Box<dyn Fn(T) -> Message + 'a>>,
    on_submit: Option<Message>,
    icon: Option<Icon<Renderer::Font>>,
    class: <Theme as Catalog>::Class<'a>,
    last_status: Option<Status>,
    increment_button: Button<'a, ButtonMessage, Theme, Renderer>,
    decrement_button: Button<'a, ButtonMessage, Theme, Renderer>,
    buttons_width: Pixels,
}

/// The default [`Padding`] of a [`NumberInput`].
pub const DEFAULT_PADDING: Padding = Padding::new(5.0);

/// The default [`Pixels`] width of a [`NumberInput`] increment/decrement buttons.
pub const DEFAULT_BUTTON_WIDTH: Pixels = Pixels(16.0);

impl<'a, T, Message, Theme, Renderer> NumberInput<'a, T, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog + button::Catalog + widget::text::Catalog + 'a,
    Renderer: text::Renderer + 'a,
    T: Num + NumAssignOps + PartialOrd + Display + FromStr + Clone + Default + Bounded + 'a,
{
    /// Creates a new [`NumberInput`] with the given placeholder and
    /// its current value.
    pub fn new(placeholder: &str, value: T) -> Self {
        let v_str = value.to_string();
        let _number_value = value;
        let value = Value::new(&v_str);
        let text_size = Pixels(16.0);
        let line_height = text::LineHeight::default();
        let height = line_height.to_absolute(text_size) + DEFAULT_PADDING.vertical();
        let increment_button: Button<ButtonMessage, Theme, Renderer> =
            button(iced::widget::text("^").size(13.0).center())
                .width(DEFAULT_BUTTON_WIDTH)
                .height(height)
                .class(<Theme as Catalog>::default_increment_button())
                .padding(0);
        let decrement_button: Button<ButtonMessage, Theme, Renderer> =
            button(iced::widget::text("v").size(11.0).center())
                .width(DEFAULT_BUTTON_WIDTH)
                .height(height)
                .class(<Theme as Catalog>::default_decrement_button())
                .padding(0);

        NumberInput {
            id: None,
            placeholder: String::from(placeholder),
            value,
            _number_value,
            max: T::max_value(),
            min: T::min_value(),
            is_secure: false,
            font: None,
            width: Length::Fixed(125.0),
            padding: DEFAULT_PADDING,
            size: None,
            line_height,
            alignment: alignment::Horizontal::Left,
            on_input: None,
            on_paste: None,
            on_submit: None,
            icon: None,
            class: <Theme as Catalog>::default(),
            last_status: None,
            increment_button,
            decrement_button,
            buttons_width: DEFAULT_BUTTON_WIDTH,
        }
    }

    /// Sets the [`Id`] of the [`NumberInput`].
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Converts the [`NumberInput`] into a secure password input.
    pub fn secure(mut self, is_secure: bool) -> Self {
        self.is_secure = is_secure;
        self
    }

    /// Sets the [`NumberInput`] max bound.
    pub fn max(mut self, max: T) -> Self {
        self.max = max;
        self
    }

    /// Sets the [`NumberInput`] min bound.
    pub fn min(mut self, min: T) -> Self {
        self.min = min;
        self
    }

    /// Sets the message that should be produced when some numbers are typed into
    /// the [`NumberInput`].
    ///
    /// If this method is not called, the [`NumberInput`] will be disabled.
    pub fn on_input(mut self, on_input: impl Fn(T) -> Message + 'a) -> Self {
        self.on_input = Some(Box::new(on_input));
        self.increment_button = self.increment_button.on_press(ButtonMessage::Increment);
        self.decrement_button = self.decrement_button.on_press(ButtonMessage::Decrement);
        self
    }

    /// Sets the message that should be produced when some numbers are typed into
    /// the [`NumberInput`], if `Some`.
    ///
    /// If `None`, the [`NumberInput`] will be disabled.
    pub fn on_input_maybe(mut self, on_input: Option<impl Fn(T) -> Message + 'a>) -> Self {
        self.on_input = on_input.map(|f| Box::new(f) as _);
        if self.on_input.is_some() {
            self.increment_button = self.increment_button.on_press(ButtonMessage::Increment);
            self.decrement_button = self.decrement_button.on_press(ButtonMessage::Decrement);
        }
        self
    }

    /// Sets the message that should be produced when the [`NumberInput`] is
    /// focused and the enter key is pressed.
    pub fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }

    /// Sets the message that should be produced when the [`NumberInput`] is
    /// focused and the enter key is pressed, if `Some`.
    pub fn on_submit_maybe(mut self, on_submit: Option<Message>) -> Self {
        self.on_submit = on_submit;
        self
    }

    /// Sets the message that should be produced when some numbers are pasted into
    /// the [`NumberInput`].
    pub fn on_paste(mut self, on_paste: impl Fn(T) -> Message + 'a) -> Self {
        self.on_paste = Some(Box::new(on_paste));
        self
    }

    /// Sets the message that should be produced when some numbers are pasted into
    /// the [`NumberInput`], if `Some`.
    pub fn on_paste_maybe(mut self, on_paste: Option<impl Fn(T) -> Message + 'a>) -> Self {
        self.on_paste = on_paste.map(|f| Box::new(f) as _);
        self
    }

    /// Sets the [`Font`] of the [`NumberInput`].
    ///
    /// [`Font`]: text::Renderer::Font
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = Some(font);
        self
    }

    /// Sets the [`Icon`] of the [`NumberInput`].
    pub fn icon(mut self, icon: Icon<Renderer::Font>) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the width of the [`NumberInput`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the [`Padding`] of the [`NumberInput`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the [`Pixels`] width of the [`NumberInput`] increment/decrement buttons.
    pub fn buttons_width<P: Into<Pixels>>(mut self, width: P) -> Self {
        self.buttons_width = width.into();
        self.increment_button = self.increment_button.width(self.buttons_width);
        self.decrement_button = self.decrement_button.width(self.buttons_width);
        self
    }

    /// Sets the text size of the [`NumberInput`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Sets the [`text::LineHeight`] of the [`NumberInput`].
    pub fn line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
        self.line_height = line_height.into();
        self
    }

    /// Sets the horizontal alignment of the [`NumberInput`].
    pub fn align_x(mut self, alignment: impl Into<alignment::Horizontal>) -> Self {
        self.alignment = alignment.into();
        self
    }

    /// Sets the style of the [`NumberInput`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        <Theme as Catalog>::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`NumberInput`].
    #[must_use]
    pub fn class(mut self, class: impl Into<<Theme as Catalog>::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Sets the style for the increment button of the [`NumberInput`].
    #[must_use]
    pub fn style_increment(
        mut self,
        style: impl Fn(&Theme, button::Status) -> button::Style + 'a,
    ) -> Self
    where
        <Theme as button::Catalog>::Class<'a>: From<button::StyleFn<'a, Theme>>,
    {
        self.increment_button = self
            .increment_button
            .class(Box::new(style) as button::StyleFn<'a, Theme>);
        self
    }

    /// Sets the style class for the increment button of the [`NumberInput`].
    #[must_use]
    pub fn class_increment(
        mut self,
        class: impl Into<<Theme as button::Catalog>::Class<'a>>,
    ) -> Self {
        self.increment_button = self.increment_button.class(class);
        self
    }

    /// Sets the style for the increment button of the [`NumberInput`].
    #[must_use]
    pub fn style_decrement(
        mut self,
        style: impl Fn(&Theme, button::Status) -> button::Style + 'a,
    ) -> Self
    where
        <Theme as button::Catalog>::Class<'a>: From<button::StyleFn<'a, Theme>>,
    {
        self.decrement_button = self
            .decrement_button
            .class(Box::new(style) as button::StyleFn<'a, Theme>);
        self
    }

    /// Sets the style class for the decrement button of the [`NumberInput`].
    #[must_use]
    pub fn class_decrement(
        mut self,
        class: impl Into<<Theme as button::Catalog>::Class<'a>>,
    ) -> Self {
        self.decrement_button = self.decrement_button.class(class);
        self
    }

    /// Lays out the [`NumberInput`], overriding its [`Value`] if provided.
    ///
    /// [`Renderer`]: text::Renderer
    pub fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
        value: Option<&Value>,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        let value = value.unwrap_or(&self.value);

        let font = self.font.unwrap_or_else(|| renderer.default_font());
        let text_size = self.size.unwrap_or_else(|| renderer.default_size());
        let padding = self.padding.fit(Size::ZERO, limits.max());
        let height = self.line_height.to_absolute(text_size);

        let limits = limits
            .width(self.width)
            .shrink(padding)
            .shrink(Size::new(self.buttons_width.into(), 0.0));
        let text_bounds = limits.resolve(self.width, height, Size::ZERO);

        let placeholder_text = Text {
            font,
            line_height: self.line_height,
            content: self.placeholder.as_str(),
            bounds: Size::new(f32::INFINITY, text_bounds.height),
            size: text_size,
            align_x: text::Alignment::Default,
            align_y: alignment::Vertical::Center,
            shaping: text::Shaping::Advanced,
            wrapping: text::Wrapping::default(),
        };

        state.placeholder.update(placeholder_text);

        let secure_value = self.is_secure.then(|| value.secure());
        let value = secure_value.as_ref().unwrap_or(value);

        state.value.update(Text {
            content: &value.to_string(),
            ..placeholder_text
        });

        let button_padding = 1.0;
        let button_limits = Limits::new(
            Size::ZERO,
            Size::new(
                f32::from(self.buttons_width) - 2.0 * button_padding,
                (f32::from(height) + padding.vertical()) / 2.0 - button_padding,
            ),
        );

        let increment_node =
            self.increment_button
                .layout(&mut tree.children[0], renderer, &button_limits);
        let decrement_node =
            self.decrement_button
                .layout(&mut tree.children[1], renderer, &button_limits);

        if let Some(icon) = &self.icon {
            let mut content = [0; 4];

            let icon_text = Text {
                line_height: self.line_height,
                content: icon.code_point.encode_utf8(&mut content) as &_,
                font: icon.font,
                size: icon.size.unwrap_or_else(|| renderer.default_size()),
                bounds: Size::new(f32::INFINITY, text_bounds.height),
                align_x: text::Alignment::Center,
                align_y: alignment::Vertical::Center,
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::default(),
            };

            state.icon.update(icon_text);

            let icon_width = state.icon.min_width();

            let (text_position, icon_position, increment_position, decrement_position) =
                match icon.side {
                    Side::Left => (
                        Point::new(padding.left + icon_width + icon.spacing, padding.top),
                        Point::new(padding.left, padding.top),
                        Point::new(
                            padding.left
                                + icon_width
                                + icon.spacing
                                + text_bounds.width
                                + padding.right
                                + button_padding,
                            button_padding,
                        ),
                        Point::new(
                            padding.left
                                + icon_width
                                + icon.spacing
                                + text_bounds.width
                                + padding.right
                                + button_padding,
                            button_limits.max().height + button_padding,
                        ),
                    ),
                    Side::Right => (
                        Point::new(padding.left, padding.top),
                        Point::new(padding.left + text_bounds.width - icon_width, padding.top),
                        Point::new(
                            padding.left
                                + text_bounds.width
                                + icon_width
                                + padding.right
                                + button_padding,
                            button_padding,
                        ),
                        Point::new(
                            padding.left
                                + text_bounds.width
                                + icon_width
                                + padding.right
                                + button_padding,
                            button_limits.max().height + button_padding,
                        ),
                    ),
                };

            let text_node =
                layout::Node::new(text_bounds - Size::new(icon_width + icon.spacing, 0.0))
                    .move_to(text_position);

            let icon_node =
                layout::Node::new(Size::new(icon_width, text_bounds.height)).move_to(icon_position);

            let increment_node = increment_node.move_to(increment_position);
            let decrement_node = decrement_node.move_to(decrement_position);

            layout::Node::with_children(
                text_bounds
                    .expand(padding)
                    .expand(Size::new(self.buttons_width.into(), 0.0)),
                vec![text_node, icon_node, increment_node, decrement_node],
            )
        } else {
            let text_node =
                layout::Node::new(text_bounds).move_to(Point::new(padding.left, padding.top));
            let increment_node = increment_node.move_to(Point::new(
                text_bounds.width + padding.horizontal() + button_padding,
                button_padding,
            ));
            let decrement_node = decrement_node.move_to(Point::new(
                text_bounds.width + padding.horizontal() + button_padding,
                button_limits.max().height + button_padding,
            ));

            layout::Node::with_children(
                text_bounds
                    .expand(padding)
                    .expand(Size::new(self.buttons_width.into(), 0.0)),
                vec![text_node, increment_node, decrement_node],
            )
        }
    }

    fn input_method<'c>(
        &self,
        state: &'c State<Renderer::Paragraph>,
        layout: Layout<'_>,
        value: &Value,
    ) -> InputMethod<&'c str> {
        let Some(Focus {
            is_window_focused: true,
            ..
        }) = &state.is_focused
        else {
            return InputMethod::Disabled;
        };

        let secure_value = self.is_secure.then(|| value.secure());
        let value = secure_value.as_ref().unwrap_or(value);

        let text_bounds = layout.children().next().unwrap().bounds();

        let caret_index = match state.cursor.state(value) {
            cursor::State::Index(position) => position,
            cursor::State::Selection { start, end } => start.min(end),
        };

        let text = state.value.raw();
        let (cursor_x, scroll_offset) =
            measure_cursor_and_scroll_offset(text, text_bounds, caret_index);

        let alignment_offset =
            alignment_offset(text_bounds.width, text.min_width(), self.alignment);

        let x = (text_bounds.x + cursor_x).floor() - scroll_offset + alignment_offset;

        InputMethod::Enabled {
            position: Point::new(x, text_bounds.y + text_bounds.height),
            purpose: if self.is_secure {
                input_method::Purpose::Secure
            } else {
                input_method::Purpose::Normal
            },
            preedit: state.preedit.as_ref().map(input_method::Preedit::as_ref),
        }
    }

    /// Draws the [`NumberInput`] with the given [`Renderer`], overriding its
    /// [`Value`] if provided.
    ///
    /// [`Renderer`]: text::Renderer
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        value: Option<&Value>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        let value = value.unwrap_or(&self.value);
        let is_disabled = self.on_input.is_none();

        let secure_value = self.is_secure.then(|| value.secure());
        let value = secure_value.as_ref().unwrap_or(value);

        let bounds = layout.bounds();

        let mut children_layout = layout.children();
        let text_bounds = children_layout.next().unwrap().bounds();

        let style = <Theme as Catalog>::style(
            theme,
            &self.class,
            self.last_status.unwrap_or(Status::Disabled),
        );

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.border,
                ..renderer::Quad::default()
            },
            style.background,
        );

        if self.icon.is_some() {
            let icon_layout = children_layout.next().unwrap();

            renderer.fill_paragraph(
                state.icon.raw(),
                icon_layout.bounds().center(),
                style.icon,
                *viewport,
            );
        }

        let increment_layout = children_layout.next().unwrap();
        self.increment_button.draw(
            &tree.children[0],
            renderer,
            theme,
            renderer_style,
            increment_layout,
            cursor,
            viewport,
        );

        let decrement_layout = children_layout.next().unwrap();
        self.decrement_button.draw(
            &tree.children[1],
            renderer,
            theme,
            renderer_style,
            decrement_layout,
            cursor,
            viewport,
        );

        let text = value.to_string();

        let (cursor, offset, is_selecting) = if let Some(focus) = state
            .is_focused
            .as_ref()
            .filter(|focus| focus.is_window_focused)
        {
            match state.cursor.state(value) {
                cursor::State::Index(position) => {
                    let (text_value_width, offset) =
                        measure_cursor_and_scroll_offset(state.value.raw(), text_bounds, position);

                    let is_cursor_visible = !is_disabled
                        && ((focus.now - focus.updated_at).as_millis()
                            / CURSOR_BLINK_INTERVAL_MILLIS)
                            % 2
                            == 0;

                    let cursor = if is_cursor_visible {
                        Some((
                            renderer::Quad {
                                bounds: Rectangle {
                                    x: (text_bounds.x + text_value_width).floor(),
                                    y: text_bounds.y,
                                    width: 1.0,
                                    height: text_bounds.height,
                                },
                                ..renderer::Quad::default()
                            },
                            style.value,
                        ))
                    } else {
                        None
                    };

                    (cursor, offset, false)
                }
                cursor::State::Selection { start, end } => {
                    let left = start.min(end);
                    let right = end.max(start);

                    let (left_position, left_offset) =
                        measure_cursor_and_scroll_offset(state.value.raw(), text_bounds, left);

                    let (right_position, right_offset) =
                        measure_cursor_and_scroll_offset(state.value.raw(), text_bounds, right);

                    let width = right_position - left_position;

                    (
                        Some((
                            renderer::Quad {
                                bounds: Rectangle {
                                    x: text_bounds.x + left_position,
                                    y: text_bounds.y,
                                    width,
                                    height: text_bounds.height,
                                },
                                ..renderer::Quad::default()
                            },
                            style.selection,
                        )),
                        if end == right {
                            right_offset
                        } else {
                            left_offset
                        },
                        true,
                    )
                }
            }
        } else {
            (None, 0.0, false)
        };

        let draw = |renderer: &mut Renderer, viewport| {
            let paragraph = if text.is_empty()
                && state
                    .preedit
                    .as_ref()
                    .map(|preedit| preedit.content.is_empty())
                    .unwrap_or(true)
            {
                state.placeholder.raw()
            } else {
                state.value.raw()
            };

            let alignment_offset =
                alignment_offset(text_bounds.width, paragraph.min_width(), self.alignment);

            if let Some((cursor, color)) = cursor {
                renderer.with_translation(
                    Vector::new(alignment_offset - offset, 0.0),
                    |renderer| {
                        renderer.fill_quad(cursor, color);
                    },
                );
            } else {
                renderer.with_translation(Vector::ZERO, |_| {});
            }

            renderer.fill_paragraph(
                paragraph,
                Point::new(text_bounds.x, text_bounds.center_y())
                    + Vector::new(alignment_offset - offset, 0.0),
                if text.is_empty() {
                    style.placeholder
                } else {
                    style.value
                },
                viewport,
            );
        };

        if is_selecting {
            renderer.with_layer(text_bounds, |renderer| draw(renderer, *viewport));
        } else {
            draw(renderer, text_bounds);
        }
    }
}

impl<'a, T, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for NumberInput<'a, T, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog + button::Catalog + widget::text::Catalog + 'a,
    Renderer: text::Renderer + 'a,
    T: Num + NumAssignOps + PartialOrd + Display + FromStr + Clone + Default + Bounded + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Renderer::Paragraph>::new())
    }

    fn children(&self) -> Vec<Tree> {
        vec![
            Tree::new(&self.increment_button as &dyn Widget<_, _, _>),
            Tree::new(&self.decrement_button as &dyn Widget<_, _, _>),
        ]
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        // Stop pasting if input becomes disabled
        if self.on_input.is_none() {
            state.is_pasting = None;
        }
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.layout(tree, renderer, limits, None)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        operation.focusable(self.id.as_ref().map(|id| &id.0), layout.bounds(), state);

        operation.text_input(self.id.as_ref().map(|id| &id.0), layout.bounds(), state);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let (increment_layout, decrement_layout) = if self.icon.is_some() {
            (
                layout.children().nth(2).unwrap(),
                layout.children().nth(3).unwrap(),
            )
        } else {
            (
                layout.children().nth(1).unwrap(),
                layout.children().nth(2).unwrap(),
            )
        };
        let mut buttons_messages = Vec::<ButtonMessage>::new();
        let mut buttons_shell = Shell::new(&mut buttons_messages);

        // Reconcile buttons shell with main shell:
        if shell.is_event_captured() {
            buttons_shell.capture_event();
        }
        buttons_shell.request_redraw_at(shell.redraw_request());
        buttons_shell.request_input_method(shell.input_method());

        self.increment_button.update(
            &mut tree.children[0],
            event,
            increment_layout,
            cursor,
            renderer,
            clipboard,
            &mut buttons_shell,
            viewport,
        );
        self.decrement_button.update(
            &mut tree.children[1],
            event,
            decrement_layout,
            cursor,
            renderer,
            clipboard,
            &mut buttons_shell,
            viewport,
        );

        // Reconcile main shell with buttons shell
        shell.request_redraw_at(buttons_shell.redraw_request());
        shell.request_input_method(buttons_shell.input_method());
        if buttons_shell.is_event_captured() {
            shell.capture_event();
        }

        if !buttons_messages.is_empty() {
            if let Some(on_input) = &self.on_input {
                for message in buttons_messages {
                    match message {
                        ButtonMessage::Increment => {
                            if let Ok(mut parsed) = self.value.to_string().parse() {
                                if parsed < self.max {
                                    parsed += T::one();
                                    if parsed > self.max {
                                        parsed = self.max.clone();
                                    }
                                    let message = (on_input)(parsed);
                                    shell.publish(message);
                                }
                            }
                        }
                        ButtonMessage::Decrement => {
                            if let Ok(mut parsed) = self.value.to_string().parse() {
                                if parsed > self.min {
                                    parsed -= T::one();
                                    if parsed < self.min {
                                        parsed = self.min.clone();
                                    }
                                    let message = (on_input)(parsed);
                                    shell.publish(message);
                                }
                            }
                        }
                    }
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
        }

        if shell.is_event_captured() {
            // if the buttons handled the event we can simply return
            return;
        }

        let update_cache = |state, value| {
            replace_paragraph(
                renderer,
                state,
                layout,
                value,
                self.font,
                self.size,
                self.line_height,
            );
        };

        let state = state::<Renderer>(tree);
        match &event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let cursor_before = state.cursor;

                let click_position = cursor.position_over(layout.bounds());

                let was_previously_focused = state.is_focused();

                state.is_focused = if click_position.is_some() {
                    let now = Instant::now();

                    Some(Focus {
                        updated_at: now,
                        now,
                        is_window_focused: true,
                    })
                } else {
                    None
                };

                if was_previously_focused {
                    if let Some(cursor_position) = click_position {
                        let text_layout = layout.children().next().unwrap();

                        let target = {
                            let text_bounds = text_layout.bounds();

                            let alignment_offset = alignment_offset(
                                text_bounds.width,
                                state.value.raw().min_width(),
                                self.alignment,
                            );

                            cursor_position.x - text_bounds.x - alignment_offset
                        };

                        let click = mouse::Click::new(
                            cursor_position,
                            mouse::Button::Left,
                            state.last_click,
                        );

                        match click.kind() {
                            click::Kind::Single => {
                                let position = if target > 0.0 {
                                    let value = if self.is_secure {
                                        self.value.secure()
                                    } else {
                                        self.value.clone()
                                    };

                                    find_cursor_position(
                                        text_layout.bounds(),
                                        &value,
                                        state,
                                        target,
                                    )
                                } else {
                                    None
                                }
                                .unwrap_or(0);

                                if state.keyboard_modifiers.shift() {
                                    state
                                        .cursor
                                        .select_range(state.cursor.start(&self.value), position);
                                } else {
                                    state.cursor.move_to(position);
                                }
                                state.is_dragging = true;
                            }
                            click::Kind::Double => {
                                if self.is_secure {
                                    state.cursor.select_all(&self.value);
                                } else {
                                    let position = find_cursor_position(
                                        text_layout.bounds(),
                                        &self.value,
                                        state,
                                        target,
                                    )
                                    .unwrap_or(0);

                                    state.cursor.select_range(
                                        self.value.previous_start_of_word(position),
                                        self.value.next_end_of_word(position),
                                    );
                                }

                                state.is_dragging = false;
                            }
                            click::Kind::Triple => {
                                state.cursor.select_all(&self.value);
                                state.is_dragging = false;
                            }
                        }

                        state.last_click = Some(click);

                        if cursor_before != state.cursor {
                            shell.request_redraw();
                        }

                        shell.capture_event();
                    } else {
                        // Widget was unfocused, lets check if the current value is valid, if it
                        // isn't we send the default value
                        if let Some(on_input) = &self.on_input {
                            if self.value.to_string().parse::<T>().is_err() {
                                let message = on_input(T::default());
                                shell.publish(message);
                                state.is_empty = false;
                                state.is_empty_neg = false;
                                shell.request_redraw();
                                shell.capture_event();
                            }
                        }
                    }
                } else if state.is_focused() {
                    state.cursor.select_all(&self.value);
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                state.is_dragging = false;
            }
            Event::Mouse(mouse::Event::CursorMoved { position })
            | Event::Touch(touch::Event::FingerMoved { position, .. }) => {
                if state.is_dragging {
                    let text_layout = layout.children().next().unwrap();

                    let target = {
                        let text_bounds = text_layout.bounds();

                        let alignment_offset = alignment_offset(
                            text_bounds.width,
                            state.value.raw().min_width(),
                            self.alignment,
                        );

                        position.x - text_bounds.x - alignment_offset
                    };

                    let value = if self.is_secure {
                        self.value.secure()
                    } else {
                        self.value.clone()
                    };

                    let position =
                        find_cursor_position(text_layout.bounds(), &value, state, target)
                            .unwrap_or(0);

                    let selection_before = state.cursor.selection(&value);

                    state
                        .cursor
                        .select_range(state.cursor.start(&value), position);

                    if let Some(focus) = &mut state.is_focused {
                        focus.updated_at = Instant::now();
                    }

                    if selection_before != state.cursor.selection(&value) {
                        shell.request_redraw();
                    }

                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, text, .. }) => {
                if let Some(focus) = &mut state.is_focused {
                    let modifiers = state.keyboard_modifiers;

                    match key.as_ref() {
                        keyboard::Key::Character("c")
                            if state.keyboard_modifiers.command() && !self.is_secure =>
                        {
                            if let Some((start, end)) = state.cursor.selection(&self.value) {
                                clipboard.write(
                                    clipboard::Kind::Standard,
                                    self.value.select(start, end).to_string(),
                                );
                            }

                            shell.capture_event();
                            return;
                        }
                        keyboard::Key::Character("x")
                            if state.keyboard_modifiers.command() && !self.is_secure =>
                        {
                            let Some(on_input) = &self.on_input else {
                                return;
                            };

                            if let Some((start, end)) = state.cursor.selection(&self.value) {
                                clipboard.write(
                                    clipboard::Kind::Standard,
                                    self.value.select(start, end).to_string(),
                                );
                            }

                            let mut editor = Editor::new(&mut self.value, &mut state.cursor);
                            editor.delete();

                            if let Ok(mut parsed) = editor.contents().parse() {
                                if parsed > self.max {
                                    parsed = self.max.clone();
                                } else if parsed < self.min {
                                    parsed = self.min.clone();
                                }
                                let message = (on_input)(parsed);
                                shell.publish(message);
                            } else {
                                if self.value.is_empty() {
                                    state.is_empty = true;
                                    state.is_empty_neg = false;
                                } else if self.value.to_string() == *"-" {
                                    state.is_empty = false;
                                    state.is_empty_neg = true;
                                }
                                shell.request_redraw();
                            }
                            shell.capture_event();

                            focus.updated_at = Instant::now();
                            update_cache(state, &self.value);
                            return;
                        }
                        keyboard::Key::Character("v")
                            if state.keyboard_modifiers.command()
                                && !state.keyboard_modifiers.alt() =>
                        {
                            let Some(on_input) = &self.on_input else {
                                return;
                            };

                            let content = match state.is_pasting.take() {
                                Some(content) => content,
                                None => {
                                    let content: String = clipboard
                                        .read(clipboard::Kind::Standard)
                                        .unwrap_or_default()
                                        .chars()
                                        .filter(|c| !c.is_control())
                                        .collect();

                                    if content.parse::<T>().is_ok() {
                                        Value::new(&content)
                                    } else {
                                        Value::new("")
                                    }
                                }
                            };

                            let mut editor = Editor::new(&mut self.value, &mut state.cursor);
                            if !content.is_empty() {
                                editor.paste(content.clone());
                            }

                            if let Ok(mut parsed) = editor.contents().parse() {
                                if parsed > self.max {
                                    parsed = self.max.clone();
                                } else if parsed < self.min {
                                    parsed = self.min.clone();
                                }
                                let message = if let Some(paste) = &self.on_paste {
                                    (paste)(parsed)
                                } else {
                                    (on_input)(parsed)
                                };
                                shell.publish(message);
                                shell.capture_event();

                                state.is_empty = false;
                                state.is_empty_neg = false;
                                state.is_pasting = Some(content);
                                focus.updated_at = Instant::now();
                                update_cache(state, &self.value);
                            } else {
                                for _ in 0..content.len() {
                                    editor.backspace();
                                }
                            }
                            return;
                        }
                        keyboard::Key::Character("a") if state.keyboard_modifiers.command() => {
                            let cursor_before = state.cursor;

                            state.cursor.select_all(&self.value);

                            if cursor_before != state.cursor {
                                focus.updated_at = Instant::now();

                                shell.request_redraw();
                            }

                            shell.capture_event();
                            return;
                        }
                        _ => {}
                    }

                    if let Some(text) = text {
                        let Some(on_input) = &self.on_input else {
                            return;
                        };

                        state.is_pasting = None;

                        if let Some(c) = text.chars().next().filter(|c| !c.is_control()) {
                            let mut editor = Editor::new(&mut self.value, &mut state.cursor);

                            if c.is_ascii_digit() || c == '-' || c == '.' || c == ',' {
                                editor.insert(c);
                            }

                            if let Ok(mut parsed) = editor.contents().parse() {
                                if parsed > self.max {
                                    parsed = self.max.clone();
                                } else if parsed < self.min {
                                    parsed = self.min.clone();
                                }
                                let message = (on_input)(parsed);
                                shell.publish(message);
                                state.is_empty = false;
                                state.is_empty_neg = false;
                            } else if c == '-' && &editor.contents() == "-" {
                                state.is_empty = false;
                                state.is_empty_neg = true;
                                shell.request_redraw();
                            } else {
                                editor.backspace();
                            }
                            shell.capture_event();

                            focus.updated_at = Instant::now();
                            update_cache(state, &self.value);
                            return;
                        }
                    }

                    match key.as_ref() {
                        keyboard::Key::Named(key::Named::Enter) => {
                            if let Some(on_submit) = self.on_submit.clone() {
                                shell.publish(on_submit);
                                shell.capture_event();
                            }
                        }
                        keyboard::Key::Named(key::Named::Backspace) => {
                            let Some(on_input) = &self.on_input else {
                                return;
                            };

                            let cursor_before = state.cursor;

                            if modifiers.jump() && state.cursor.selection(&self.value).is_none() {
                                if self.is_secure {
                                    let cursor_pos = state.cursor.end(&self.value);
                                    state.cursor.select_range(0, cursor_pos);
                                } else {
                                    state.cursor.select_left_by_words(&self.value);
                                }
                            }

                            let mut editor = Editor::new(&mut self.value, &mut state.cursor);
                            editor.backspace();

                            if let Ok(mut parsed) = editor.contents().parse() {
                                if parsed > self.max {
                                    parsed = self.max.clone();
                                } else if parsed < self.min {
                                    parsed = self.min.clone();
                                }
                                let message = (on_input)(parsed);
                                shell.publish(message);

                                if cursor_before != state.cursor {
                                    shell.request_redraw();
                                }
                            } else {
                                if self.value.is_empty() {
                                    state.is_empty = true;
                                    state.is_empty_neg = false;
                                } else if self.value.to_string() == *"-" {
                                    state.is_empty = false;
                                    state.is_empty_neg = true;
                                }
                                shell.request_redraw();
                            }
                            shell.capture_event();

                            focus.updated_at = Instant::now();
                            update_cache(state, &self.value);
                        }
                        keyboard::Key::Named(key::Named::Delete) => {
                            let Some(on_input) = &self.on_input else {
                                return;
                            };

                            if modifiers.jump() && state.cursor.selection(&self.value).is_none() {
                                if self.is_secure {
                                    let cursor_pos = state.cursor.end(&self.value);
                                    state.cursor.select_range(cursor_pos, self.value.len());
                                } else {
                                    state.cursor.select_right_by_words(&self.value);
                                }
                            }

                            let mut editor = Editor::new(&mut self.value, &mut state.cursor);
                            editor.delete();

                            if let Ok(mut parsed) = editor.contents().parse() {
                                if parsed > self.max {
                                    parsed = self.max.clone();
                                } else if parsed < self.min {
                                    parsed = self.min.clone();
                                }
                                let message = (on_input)(parsed);
                                shell.publish(message);
                            } else {
                                if self.value.is_empty() {
                                    state.is_empty = true;
                                    state.is_empty_neg = false;
                                } else if self.value.to_string() == *"-" {
                                    state.is_empty = false;
                                    state.is_empty_neg = true;
                                }
                                shell.request_redraw();
                            }
                            shell.capture_event();

                            focus.updated_at = Instant::now();
                            update_cache(state, &self.value);
                        }
                        keyboard::Key::Named(key::Named::Home) => {
                            let cursor_before = state.cursor;

                            if modifiers.shift() {
                                state
                                    .cursor
                                    .select_range(state.cursor.start(&self.value), 0);
                            } else {
                                state.cursor.move_to(0);
                            }

                            if cursor_before != state.cursor {
                                focus.updated_at = Instant::now();

                                shell.request_redraw();
                            }

                            shell.capture_event();
                        }
                        keyboard::Key::Named(key::Named::End) => {
                            let cursor_before = state.cursor;

                            if modifiers.shift() {
                                state.cursor.select_range(
                                    state.cursor.start(&self.value),
                                    self.value.len(),
                                );
                            } else {
                                state.cursor.move_to(self.value.len());
                            }

                            if cursor_before != state.cursor {
                                focus.updated_at = Instant::now();

                                shell.request_redraw();
                            }

                            shell.capture_event();
                        }
                        keyboard::Key::Named(key::Named::ArrowLeft)
                            if modifiers.macos_command() =>
                        {
                            let cursor_before = state.cursor;

                            if modifiers.shift() {
                                state
                                    .cursor
                                    .select_range(state.cursor.start(&self.value), 0);
                            } else {
                                state.cursor.move_to(0);
                            }

                            if cursor_before != state.cursor {
                                focus.updated_at = Instant::now();

                                shell.request_redraw();
                            }

                            shell.capture_event();
                        }
                        keyboard::Key::Named(key::Named::ArrowRight)
                            if modifiers.macos_command() =>
                        {
                            let cursor_before = state.cursor;

                            if modifiers.shift() {
                                state.cursor.select_range(
                                    state.cursor.start(&self.value),
                                    self.value.len(),
                                );
                            } else {
                                state.cursor.move_to(self.value.len());
                            }

                            if cursor_before != state.cursor {
                                focus.updated_at = Instant::now();

                                shell.request_redraw();
                            }

                            shell.capture_event();
                        }
                        keyboard::Key::Named(key::Named::ArrowLeft) => {
                            let cursor_before = state.cursor;

                            if modifiers.jump() && !self.is_secure {
                                if modifiers.shift() {
                                    state.cursor.select_left_by_words(&self.value);
                                } else {
                                    state.cursor.move_left_by_words(&self.value);
                                }
                            } else if modifiers.shift() {
                                state.cursor.select_left(&self.value);
                            } else {
                                state.cursor.move_left(&self.value);
                            }

                            if cursor_before != state.cursor {
                                focus.updated_at = Instant::now();

                                shell.request_redraw();
                            }

                            shell.capture_event();
                        }
                        keyboard::Key::Named(key::Named::ArrowRight) => {
                            let cursor_before = state.cursor;

                            if modifiers.jump() && !self.is_secure {
                                if modifiers.shift() {
                                    state.cursor.select_right_by_words(&self.value);
                                } else {
                                    state.cursor.move_right_by_words(&self.value);
                                }
                            } else if modifiers.shift() {
                                state.cursor.select_right(&self.value);
                            } else {
                                state.cursor.move_right(&self.value);
                            }

                            if cursor_before != state.cursor {
                                focus.updated_at = Instant::now();

                                shell.request_redraw();
                            }

                            shell.capture_event();
                        }
                        keyboard::Key::Named(key::Named::Escape) => {
                            state.is_focused = None;
                            state.is_dragging = false;
                            state.is_pasting = None;

                            state.keyboard_modifiers = keyboard::Modifiers::default();

                            // Widget was unfocused, lets check if the current value is valid, if it
                            // isn't we send the default value
                            if let Some(on_input) = &self.on_input {
                                if self.value.to_string().parse::<T>().is_err() {
                                    let message = on_input(T::default());
                                    shell.publish(message);
                                    state.is_empty = false;
                                    state.is_empty_neg = false;
                                    shell.request_redraw();
                                }
                            }

                            shell.capture_event();
                        }
                        _ => {}
                    }
                }
            }
            Event::Keyboard(keyboard::Event::KeyReleased { key, .. }) => {
                if state.is_focused.is_some() {
                    if let keyboard::Key::Character("v") = key.as_ref() {
                        state.is_pasting = None;

                        shell.capture_event();
                    }
                }

                state.is_pasting = None;
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.keyboard_modifiers = *modifiers;
            }
            Event::InputMethod(event) => match event {
                input_method::Event::Opened | input_method::Event::Closed => {
                    state.preedit = matches!(event, input_method::Event::Opened)
                        .then(input_method::Preedit::new);

                    shell.request_redraw();
                }
                input_method::Event::Preedit(content, selection) => {
                    if state.is_focused.is_some() {
                        state.preedit = Some(input_method::Preedit {
                            content: content.to_owned(),
                            selection: selection.clone(),
                            text_size: self.size,
                        });

                        shell.request_redraw();
                    }
                }
                input_method::Event::Commit(text) => {
                    if let Some(focus) = &mut state.is_focused {
                        let Some(on_input) = &self.on_input else {
                            return;
                        };

                        let mut editor = Editor::new(&mut self.value, &mut state.cursor);
                        if text.parse::<T>().is_ok() {
                            editor.paste(Value::new(text));
                        }

                        focus.updated_at = Instant::now();
                        state.is_pasting = None;

                        if let Ok(mut parsed) = editor.contents().parse() {
                            if parsed > self.max {
                                parsed = self.max.clone();
                            } else if parsed < self.min {
                                parsed = self.min.clone();
                            }
                            let message = (on_input)(parsed);
                            shell.publish(message);
                            shell.capture_event();
                            state.is_empty = false;
                            state.is_empty_neg = false;

                            update_cache(state, &self.value);
                        } else {
                            for _ in 0..text.len() {
                                editor.backspace();
                            }
                            if self.value.is_empty() {
                                state.is_empty = true;
                                state.is_empty_neg = false;
                            } else if self.value.to_string() == *"-" {
                                state.is_empty = false;
                                state.is_empty_neg = true;
                            }
                            shell.request_redraw();
                            update_cache(state, &self.value);
                        }
                    }
                }
            },
            Event::Window(window::Event::Unfocused) => {
                if let Some(focus) = &mut state.is_focused {
                    focus.is_window_focused = false;
                }
            }
            Event::Window(window::Event::Focused) => {
                if let Some(focus) = &mut state.is_focused {
                    focus.is_window_focused = true;
                    focus.updated_at = Instant::now();

                    shell.request_redraw();
                }
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                if let Some(focus) = &mut state.is_focused {
                    if focus.is_window_focused {
                        if matches!(state.cursor.state(&self.value), cursor::State::Index(_)) {
                            focus.now = *now;

                            let millis_until_redraw = CURSOR_BLINK_INTERVAL_MILLIS
                                - (*now - focus.updated_at).as_millis()
                                    % CURSOR_BLINK_INTERVAL_MILLIS;

                            shell.request_redraw_at(
                                *now + Duration::from_millis(millis_until_redraw as u64),
                            );
                        }

                        shell.request_input_method(&self.input_method(state, layout, &self.value));
                    }
                }
            }
            _ => {}
        }

        let is_disabled = self.on_input.is_none();

        let status = if is_disabled {
            Status::Disabled
        } else if state.is_focused() {
            Status::Focused {
                is_hovered: cursor.is_over(layout.bounds()),
            }
        } else if cursor.is_over(layout.bounds()) {
            Status::Hovered
        } else {
            Status::Active
        };

        if let Event::Window(window::Event::RedrawRequested(_now)) = event {
            self.last_status = Some(status);
        } else if self
            .last_status
            .is_some_and(|last_status| status != last_status)
        {
            shell.request_redraw();
        }

        if state.is_empty && !self.value.is_empty() {
            self.value = Value::new("");
            shell.request_redraw();
        } else if state.is_empty_neg && self.value.to_string() != *"-" {
            self.value = Value::new("-");
            replace_paragraph(
                renderer,
                state,
                layout,
                &self.value,
                self.font,
                self.size,
                self.line_height,
            );
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.draw(tree, renderer, theme, style, layout, cursor, None, viewport);
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            if self.on_input.is_none() {
                mouse::Interaction::Idle
            } else {
                let (increment_layout, decrement_layout) = if self.icon.is_some() {
                    (
                        layout.children().nth(2).unwrap(),
                        layout.children().nth(3).unwrap(),
                    )
                } else {
                    (
                        layout.children().nth(1).unwrap(),
                        layout.children().nth(2).unwrap(),
                    )
                };
                if cursor.is_over(increment_layout.bounds())
                    || cursor.is_over(decrement_layout.bounds())
                {
                    mouse::Interaction::Pointer
                } else {
                    mouse::Interaction::Text
                }
            }
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, T, Message, Theme, Renderer> From<NumberInput<'a, T, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + button::Catalog + widget::text::Catalog + 'a,
    Renderer: text::Renderer + 'a,
    T: Num + NumAssignOps + PartialOrd + Display + FromStr + Clone + Default + Bounded + 'a,
{
    fn from(
        number_input: NumberInput<'a, T, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(number_input)
    }
}

/// The content of the [`Icon`].
#[derive(Debug, Clone)]
pub struct Icon<Font> {
    /// The font that will be used to display the `code_point`.
    pub font: Font,
    /// The unicode code point that will be used as the icon.
    pub code_point: char,
    /// The font size of the content.
    pub size: Option<Pixels>,
    /// The spacing between the [`Icon`] and the numbers in a [`NumberInput`].
    pub spacing: f32,
    /// The side of a [`NumberInput`] where to display the [`Icon`].
    pub side: Side,
}

/// The side of a [`NumberInput`].
#[derive(Debug, Clone)]
pub enum Side {
    /// The left side of a [`NumberInput`].
    Left,
    /// The right side of a [`NumberInput`].
    Right,
}

/// The identifier of a [`NumberInput`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(widget::Id);

impl Id {
    /// Creates a custom [`Id`].
    pub fn new(id: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        Self(widget::Id::new(id))
    }

    /// Creates a unique [`Id`].
    ///
    /// This function produces a different [`Id`] every time it is called.
    pub fn unique() -> Self {
        Self(widget::Id::unique())
    }
}

impl From<Id> for widget::Id {
    fn from(id: Id) -> Self {
        id.0
    }
}

impl From<&'static str> for Id {
    fn from(id: &'static str) -> Self {
        Self::new(id)
    }
}

impl From<String> for Id {
    fn from(id: String) -> Self {
        Self::new(id)
    }
}

/// Produces a [`Task`] that returns whether the [`NumberInput`] with the given [`Id`] is focused or not.
pub fn is_focused(id: impl Into<Id>) -> Task<bool> {
    widget::operate(operation::focusable::is_focused(id.into().0))
}

/// Produces a [`Task`] that focuses the [`NumberInput`] with the given [`Id`].
pub fn focus<T>(id: impl Into<Id>) -> Task<T>
where
    T: Send + 'static,
{
    widget::operate(operation::focusable::focus(id.into().0))
}

/// Produces a [`Task`] that moves the cursor of the [`NumberInput`] with the given [`Id`] to the
/// end.
pub fn move_cursor_to_end<T>(id: impl Into<Id>) -> Task<T>
where
    T: Send + 'static,
{
    widget::operate(operation::text_input::move_cursor_to_end(id.into().0))
}

/// Produces a [`Task`] that moves the cursor of the [`NumberInput`] with the given [`Id`] to the
/// front.
pub fn move_cursor_to_front<T>(id: impl Into<Id>) -> Task<T>
where
    T: Send + 'static,
{
    widget::operate(operation::text_input::move_cursor_to_front(id.into().0))
}

/// Produces a [`Task`] that moves the cursor of the [`NumberInput`] with the given [`Id`] to the
/// provided position.
pub fn move_cursor_to<T>(id: impl Into<Id>, position: usize) -> Task<T>
where
    T: Send + 'static,
{
    widget::operate(operation::text_input::move_cursor_to(id.into().0, position))
}

/// Produces a [`Task`] that selects all the content of the [`NumberInput`] with the given [`Id`].
pub fn select_all<T>(id: impl Into<Id>) -> Task<T>
where
    T: Send + 'static,
{
    widget::operate(operation::text_input::select_all(id.into().0))
}

/// The state of a [`NumberInput`].
#[derive(Debug, Default, Clone)]
pub struct State<P: text::Paragraph> {
    value: paragraph::Plain<P>,
    placeholder: paragraph::Plain<P>,
    icon: paragraph::Plain<P>,
    is_focused: Option<Focus>,
    is_dragging: bool,
    is_pasting: Option<Value>,
    is_empty: bool,
    is_empty_neg: bool,
    preedit: Option<input_method::Preedit>,
    last_click: Option<mouse::Click>,
    cursor: Cursor,
    keyboard_modifiers: keyboard::Modifiers,
    // TODO: Add stateful horizontal scrolling offset
}

fn state<Renderer: text::Renderer>(tree: &mut Tree) -> &mut State<Renderer::Paragraph> {
    tree.state.downcast_mut::<State<Renderer::Paragraph>>()
}

#[derive(Debug, Clone)]
struct Focus {
    updated_at: Instant,
    now: Instant,
    is_window_focused: bool,
}

impl<P: text::Paragraph> State<P> {
    /// Creates a new [`State`], representing an unfocused [`NumberInput`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether the [`NumberInput`] is currently focused or not.
    pub fn is_focused(&self) -> bool {
        self.is_focused.is_some()
    }

    /// Returns the [`Cursor`] of the [`NumberInput`].
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// Focuses the [`NumberInput`].
    pub fn focus(&mut self) {
        let now = Instant::now();

        self.is_focused = Some(Focus {
            updated_at: now,
            now,
            is_window_focused: true,
        });

        self.move_cursor_to_end();
    }

    /// Unfocuses the [`NumberInput`].
    pub fn unfocus(&mut self) {
        self.is_focused = None;
    }

    /// Moves the [`Cursor`] of the [`NumberInput`] to the front of the input value.
    pub fn move_cursor_to_front(&mut self) {
        self.cursor.move_to(0);
    }

    /// Moves the [`Cursor`] of the [`NumberInput`] to the end of the input value.
    pub fn move_cursor_to_end(&mut self) {
        self.cursor.move_to(usize::MAX);
    }

    /// Moves the [`Cursor`] of the [`NumberInput`] to an arbitrary location.
    pub fn move_cursor_to(&mut self, position: usize) {
        self.cursor.move_to(position);
    }

    /// Selects all the content of the [`NumberInput`].
    pub fn select_all(&mut self) {
        self.cursor.select_range(0, usize::MAX);
    }
}

impl<P: text::Paragraph> operation::Focusable for State<P> {
    fn is_focused(&self) -> bool {
        State::is_focused(self)
    }

    fn focus(&mut self) {
        State::focus(self);
    }

    fn unfocus(&mut self) {
        State::unfocus(self);
    }
}

impl<P: text::Paragraph> operation::TextInput for State<P> {
    fn move_cursor_to_front(&mut self) {
        State::move_cursor_to_front(self);
    }

    fn move_cursor_to_end(&mut self) {
        State::move_cursor_to_end(self);
    }

    fn move_cursor_to(&mut self, position: usize) {
        State::move_cursor_to(self, position);
    }

    fn select_all(&mut self) {
        State::select_all(self);
    }
}

fn offset<P: text::Paragraph>(text_bounds: Rectangle, value: &Value, state: &State<P>) -> f32 {
    if state.is_focused() {
        let cursor = state.cursor();

        let focus_position = match cursor.state(value) {
            cursor::State::Index(i) => i,
            cursor::State::Selection { end, .. } => end,
        };

        let (_, offset) =
            measure_cursor_and_scroll_offset(state.value.raw(), text_bounds, focus_position);

        offset
    } else {
        0.0
    }
}

fn measure_cursor_and_scroll_offset(
    paragraph: &impl text::Paragraph,
    text_bounds: Rectangle,
    cursor_index: usize,
) -> (f32, f32) {
    let grapheme_position = paragraph
        .grapheme_position(0, cursor_index)
        .unwrap_or(Point::ORIGIN);

    let offset = ((grapheme_position.x + 5.0) - text_bounds.width).max(0.0);

    (grapheme_position.x, offset)
}

/// Computes the position of the cursor at the given X coordinate of
/// a [`NumberInput`].
fn find_cursor_position<P: text::Paragraph>(
    text_bounds: Rectangle,
    value: &Value,
    state: &State<P>,
    x: f32,
) -> Option<usize> {
    let offset = offset(text_bounds, value, state);
    let value = value.to_string();

    let char_offset = state
        .value
        .raw()
        .hit_test(Point::new(x + offset, text_bounds.height / 2.0))
        .map(text::Hit::cursor)?;

    Some(
        unicode_segmentation::UnicodeSegmentation::graphemes(
            &value[..char_offset.min(value.len())],
            true,
        )
        .count(),
    )
}

fn replace_paragraph<Renderer>(
    renderer: &Renderer,
    state: &mut State<Renderer::Paragraph>,
    layout: Layout<'_>,
    value: &Value,
    font: Option<Renderer::Font>,
    text_size: Option<Pixels>,
    line_height: text::LineHeight,
) where
    Renderer: text::Renderer,
{
    let font = font.unwrap_or_else(|| renderer.default_font());
    let text_size = text_size.unwrap_or_else(|| renderer.default_size());

    let mut children_layout = layout.children();
    let text_bounds = children_layout.next().unwrap().bounds();

    state.value = paragraph::Plain::new(Text {
        font,
        line_height,
        content: &value.to_string(),
        bounds: Size::new(f32::INFINITY, text_bounds.height),
        size: text_size,
        align_x: text::Alignment::Default,
        align_y: alignment::Vertical::Center,
        shaping: text::Shaping::Advanced,
        wrapping: text::Wrapping::default(),
    });
}

const CURSOR_BLINK_INTERVAL_MILLIS: u128 = 500;

/// The possible status of a [`NumberInput`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`NumberInput`] can be interacted with.
    Active,
    /// The [`NumberInput`] is being hovered.
    Hovered,
    /// The [`NumberInput`] is focused.
    Focused {
        /// Whether the [`NumberInput`] is hovered, while focused.
        is_hovered: bool,
    },
    /// The [`NumberInput`] cannot be interacted with.
    Disabled,
}

/// The appearance of a number input.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The [`Background`] of the number input.
    pub background: Background,
    /// The [`Border`] of the number input.
    pub border: Border,
    /// The [`Color`] of the icon of the number input.
    pub icon: Color,
    /// The [`Color`] of the placeholder of the number input.
    pub placeholder: Color,
    /// The [`Color`] of the value of the number input.
    pub value: Color,
    /// The [`Color`] of the selection of the number input.
    pub selection: Color,
}

/// The theme catalog of a [`NumberInput`].
pub trait Catalog: Sized + button::Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> <Self as Catalog>::Class<'a>;

    /// The default class for the increment buttons of the [`NumberInput`].
    fn default_increment_button<'a>() -> <Self as button::Catalog>::Class<'a> {
        <Self as button::Catalog>::default()
    }

    /// The default class for the decrement buttons of the [`NumberInput`].
    fn default_decrement_button<'a>() -> <Self as button::Catalog>::Class<'a> {
        <Self as button::Catalog>::default()
    }

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &<Self as Catalog>::Class<'_>, status: Status) -> Style;

    /// The button [`button::Style`] of a button class with the given button status.
    fn style_button(
        &self,
        class: &<Self as button::Catalog>::Class<'_>,
        status: button::Status,
    ) -> button::Style;
}

/// A styling function for a [`NumberInput`].
///
/// This is just a boxed closure: `Fn(&Theme, Status) -> Style`.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> <Self as Catalog>::Class<'a> {
        Box::new(default)
    }

    fn default_increment_button<'a>() -> <Self as button::Catalog>::Class<'a> {
        Box::new(top_button) as button::StyleFn<'_, Theme>
    }

    fn default_decrement_button<'a>() -> <Self as button::Catalog>::Class<'a> {
        Box::new(bottom_button) as button::StyleFn<'_, Theme>
    }

    fn style(&self, class: &<Self as Catalog>::Class<'_>, status: Status) -> Style {
        class(self, status)
    }

    fn style_button(
        &self,
        class: &<Self as button::Catalog>::Class<'_>,
        status: button::Status,
    ) -> button::Style {
        class(self, status)
    }
}

/// The default style of a [`NumberInput`].
pub fn default(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();

    let active = Style {
        background: Background::Color(palette.background.base.color),
        border: Border {
            radius: 2.0.into(),
            width: 1.0,
            color: palette.background.strongest.color,
        },
        icon: palette.background.weak.text,
        placeholder: palette.background.strongest.color,
        value: palette.background.base.text,
        selection: palette.primary.weak.color,
    };

    match status {
        Status::Active => active,
        Status::Hovered => Style {
            border: Border {
                color: palette.background.base.text,
                ..active.border
            },
            ..active
        },
        Status::Focused { .. } => Style {
            border: Border {
                color: palette.primary.strong.color,
                ..active.border
            },
            ..active
        },
        Status::Disabled => Style {
            background: Background::Color(palette.background.weak.color),
            value: active.placeholder,
            ..active
        },
    }
}

/// The default style for the top button of a [`NumberInput`].
pub fn top_button(theme: &Theme, status: button::Status) -> button::Style {
    let base = button::secondary(theme, status);
    button::Style {
        border: border::rounded(border::top(2)),
        ..base
    }
}

/// The default style for the bottom button of a [`NumberInput`].
pub fn bottom_button(theme: &Theme, status: button::Status) -> button::Style {
    let base = button::secondary(theme, status);
    button::Style {
        border: border::rounded(border::bottom(2)),
        ..base
    }
}

fn alignment_offset(
    text_bounds_width: f32,
    text_min_width: f32,
    alignment: alignment::Horizontal,
) -> f32 {
    if text_min_width > text_bounds_width {
        0.0
    } else {
        match alignment {
            alignment::Horizontal::Left => 0.0,
            alignment::Horizontal::Center => (text_bounds_width - text_min_width) / 2.0,
            alignment::Horizontal::Right => text_bounds_width - text_min_width,
        }
    }
}

#[derive(Debug, Clone)]
enum ButtonMessage {
    Increment,
    Decrement,
}
