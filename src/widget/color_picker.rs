// This is a reworked version of the iced_aw's ColorPicker ported to iced version
// 0.14+ and with some changes/fixes that I wanted myself.
//
// Copyright (c) 2020 Kaiden42
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::collections::HashMap;

use iced::{
    advanced::{
        graphics::geometry::Renderer as _,
        layout::{Limits, Node},
        overlay, renderer,
        text::Renderer as _,
        widget::tree::{self, Tag, Tree},
        Clipboard, Layout, Overlay, Renderer as _, Shell, Text, Widget,
    },
    alignment::{Horizontal, Vertical},
    event, keyboard,
    mouse::{self, Cursor},
    touch,
    widget::{
        canvas::{self, LineCap, Path, Stroke},
        container, row, text,
        text::Wrapping,
        text_input, Button, Column, Row, TextInput,
    },
    Alignment, Background, Border, Color, Element, Event, Length, Padding, Pixels, Point,
    Rectangle, Renderer, Shadow, Size, Theme, Vector,
};

/// An input element for picking colors.
///
/// # Example
/// ```rust
/// # use iced_aw::ColorPicker;
/// # use iced::{Color, widget::{button, Button, Text}};
/// #
/// #[derive(Clone, Debug)]
/// enum Message {
///     Open,
///     Cancel,
///     Submit(Color),
/// }
///
/// let color_picker = ColorPicker::new(
///     true,
///     Color::default(),
///     Button::new(Text::new("Pick color"))
///         .on_press(Message::Open),
///     Message::Cancel,
///     Message::Submit,
/// );
/// ```
#[allow(missing_debug_implementations)]
pub struct ColorPicker<'a, Message, Theme = iced::Theme>
where
    Message: Clone,
    Theme: Catalog + iced::widget::button::Catalog + text_input::Catalog + Clone,
{
    /// Show the picker.
    show_picker: bool,
    /// The color to show.
    color: Color,
    /// The underlying element.
    underlay: Element<'a, Message, Theme, Renderer>,
    /// The message that is sent if the cancel button of the [`ColorPickerOverlay`] is pressed.
    on_cancel: Message,
    /// The function that produces a message when the submit button of the [`ColorPickerOverlay`] is pressed.
    on_submit: Box<dyn Fn(Color) -> Message + 'a>,
    /// The style of the [`ColorPickerOverlay`].
    class: <Theme as Catalog>::Class<'a>,
    /// Overlay element to be used for the layout calculation only
    overlay_el: Element<'a, Message, Theme, Renderer>,
    /// The cancel button of the [`ColorPickerOverlay`].
    cancel_button: Element<'a, Message, Theme, Renderer>,
    /// The submit button of the [`ColorPickerOverlay`].
    submit_button: Element<'a, Message, Theme, Renderer>,
    /// The hex input of the [`ColorPickerOverlay`].
    hex_input: TextInput<'a, InternalMessage, InternalTheme<Theme>, Renderer>,
}

#[derive(Debug, Clone)]
enum InternalMessage {
    ChangeInput(String),
}

struct InternalTheme<Theme: text_input::Catalog + Clone> {
    theme: Theme,
    color: Color,
}
impl<Theme: text_input::Catalog + Clone> text_input::Catalog for InternalTheme<Theme> {
    type Class<'a> = <Theme as text_input::Catalog>::Class<'a>;

    fn default<'a>() -> Self::Class<'a> {
        <Theme as text_input::Catalog>::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: text_input::Status) -> text_input::Style {
        let hsv = Hsv::from(self.color);
        let default = <Theme as text_input::Catalog>::style(&self.theme, class, status);
        text_input::Style {
            background: self.color.into(),
            value: Color {
                a: 1.0,
                ..Hsv {
                    hue: 0,
                    saturation: 0.0,
                    value: if hsv.value < 0.5 { 1.0 } else { 0.0 },
                }
                .into()
            },
            border: Border {
                color: default.border.color,
                width: 1.0,
                radius: 5.0.into(),
            },
            ..default
        }
    }
}

impl<'a, Message, Theme> ColorPicker<'a, Message, Theme>
where
    Message: 'a + Clone,
    Theme: 'a
        + Catalog
        + iced::widget::button::Catalog
        + iced::widget::text::Catalog
        + container::Catalog
        + text_input::Catalog
        + Clone,
{
    /// Creates a new [`ColorPicker`] wrapping around the given underlay.
    ///
    /// It expects:
    ///     * if the overlay of the color picker is visible.
    ///     * the initial color to show.
    ///     * the underlay [`Element`] on which this [`ColorPicker`]
    ///         will be wrapped around.
    ///     * a message that will be send when the cancel button of the [`ColorPicker`]
    ///         is pressed.
    ///     * a function that will be called when the submit button of the [`ColorPicker`]
    ///         is pressed, which takes the picked [`Color`] value.
    pub fn new<U, F>(
        show_picker: bool,
        color: Color,
        underlay: U,
        on_cancel: Message,
        on_submit: F,
    ) -> Self
    where
        U: Into<Element<'a, Message, Theme, Renderer>>,
        F: Fn(Color) -> Message + 'a,
    {
        let cancel_button = Button::new(
            text("X")
                .align_x(Horizontal::Center)
                .font(iced::Font {
                    weight: iced::font::Weight::ExtraBold,
                    ..iced::Font::DEFAULT
                })
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .on_press(on_cancel.clone())
        .into();
        let submit_button = Button::new(
            text(Renderer::CHECKMARK_ICON)
                .font(Renderer::ICON_FONT)
                .align_x(Horizontal::Center)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .on_press(on_cancel.clone()) // Sending a fake message
        .into();
        let hex_input = text_input(&color.as_hex_string(), &color.as_hex_string())
            .on_input(InternalMessage::ChangeInput)
            .padding(PADDING)
            .align_x(iced::Alignment::Center);

        let overlay_el = overlay_element(color, on_cancel.clone());

        Self {
            show_picker,
            color,
            underlay: underlay.into(),
            on_cancel,
            on_submit: Box::new(on_submit),
            class: <Theme as Catalog>::default(),
            overlay_el,
            cancel_button,
            submit_button,
            hex_input,
        }
    }

    /// Sets the style of the [`ColorPicker`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        <Theme as Catalog>::Class<'a>: From<StyleFn<'a, Theme, Style>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme, Style>).into();
        self
    }

    /// Sets the class of the input of the [`ColorPicker`].
    #[must_use]
    pub fn class(mut self, class: impl Into<<Theme as Catalog>::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

/// The state of the [`ColorPicker`].
#[derive(Debug, Default)]
pub struct State {
    /// The state of the overlay.
    pub(crate) overlay_state: OverlayState,
}

impl State {
    /// Creates a new [`State`].
    #[must_use]
    pub fn new(color: Color) -> Self
    where
        Theme: text::Catalog + container::Catalog + iced::widget::button::Catalog,
    {
        Self {
            overlay_state: OverlayState::new(color),
        }
    }

    /// Resets the color of the state.
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.overlay_state.color = Color::from_rgb(0.5, 0.25, 0.25);
        self.overlay_state.color_bar_dragged = ColorBarDragged::None;
    }
}

impl<'a, Message, Theme> Widget<Message, Theme, Renderer> for ColorPicker<'a, Message, Theme>
where
    Message: 'a + Clone,
    Theme: 'a
        + Catalog
        + iced::widget::button::Catalog
        + iced::widget::text::Catalog
        + container::Catalog
        + text_input::Catalog
        + Clone,
{
    fn tag(&self) -> Tag {
        Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new(self.color))
    }

    fn children(&self) -> Vec<Tree> {
        let mut buttons_tree = Tree::empty();
        buttons_tree.children = vec![
            Tree::new(&self.cancel_button),
            Tree::new(&self.submit_button),
            Tree::empty(),
            Tree::new(&self.hex_input as &dyn Widget<_, _, _>),
        ];
        vec![Tree::new(&self.underlay), buttons_tree]
    }

    fn diff(&self, tree: &mut Tree) {
        let color_picker_state = &mut tree.state.downcast_mut::<State>().overlay_state;

        if color_picker_state.color != self.color {
            color_picker_state.color = self.color;
            if !matches!(self.color, Color::BLACK | Color::WHITE) {
                // Black (0x000000) and White (0xFFFFFF) colors don't have hue, so we keep it as it
                // was on those cases
                let hue = Hsv::from(self.color).hue;
                color_picker_state.previous_hue_degrees = hue;
                color_picker_state.current_hue_degrees = hue;
            }
        }

        if !self.show_picker
            && color_picker_state.current_hue_degrees != color_picker_state.previous_hue_degrees
        {
            color_picker_state.current_hue_degrees = color_picker_state.previous_hue_degrees;
        }

        tree.diff_children(&[&self.underlay, &self.overlay_el]);
    }

    fn size(&self) -> iced::Size<Length> {
        self.underlay.as_widget().size()
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        self.underlay
            .as_widget()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn update(
        &mut self,
        state: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.underlay.as_widget_mut().update(
            &mut state.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.underlay.as_widget().mouse_interaction(
            &state.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        self.underlay.as_widget().draw(
            &state.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let (underlay_tree, picker_tree) = state.children.split_at_mut(1);

        let under_overlay = self.underlay.as_widget_mut().overlay(
            &mut underlay_tree[0],
            layout,
            renderer,
            viewport,
            translation,
        );

        let picker = self.show_picker.then(|| {
            let picker_state: &mut State = state.state.downcast_mut::<State>();

            let bounds = layout.bounds();
            let position = bounds.position() + translation;
            let underlay_height = bounds.height;

            ColorPickerOverlay::new(
                &mut picker_state.overlay_state,
                &mut picker_tree[0],
                self.on_cancel.clone(),
                &self.on_submit,
                position,
                *viewport,
                underlay_height,
                &self.class,
                &mut self.cancel_button,
                &mut self.submit_button,
                &mut self.hex_input,
            )
            .overlay()
        });

        let overlays = under_overlay.into_iter().chain(picker).collect::<Vec<_>>();

        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}

impl<'a, Message, Theme> From<ColorPicker<'a, Message, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a
        + Catalog
        + iced::widget::button::Catalog
        + iced::widget::text::Catalog
        + container::Catalog
        + text_input::Catalog
        + Clone,
{
    fn from(color_picker: ColorPicker<'a, Message, Theme>) -> Self {
        Element::new(color_picker)
    }
}

/// The padding around the elements.
const PADDING: Padding = Padding::new(10.0);
/// The spacing between the element.
const SPACING: Pixels = Pixels(15.0);
/// The spacing between the buttons.
const BUTTON_SPACING: Pixels = Pixels(5.0);
/// The offset to move the overlay above/below the underlay element.
const OFFSET: f32 = 10.0;

/// The step value of the keyboard change of the sat/value color values.
const SAT_VALUE_STEP: f32 = 0.005;
/// The step value of the keyboard change of the hue color value.
const HUE_STEP: i32 = 1;
/// The step value of the keyboard change of the RGBA color values.
const RGBA_STEP: i16 = 1;

/// The overlay of the [`ColorPicker`](crate::widget::ColorPicker).
#[allow(missing_debug_implementations)]
pub struct ColorPickerOverlay<'a, 'b, Message, Theme>
where
    Message: Clone + 'a,
    Theme: Catalog + iced::widget::button::Catalog + text_input::Catalog + Clone,
    'b: 'a,
{
    /// The state of the [`ColorPickerOverlay`].
    state: &'a mut OverlayState,
    /// The cancel button of the [`ColorPickerOverlay`].
    cancel_button: &'a mut Element<'b, Message, Theme, Renderer>,
    /// The submit button of the [`ColorPickerOverlay`].
    submit_button: &'a mut Element<'b, Message, Theme, Renderer>,
    /// The hex input of the [`ColorPickerOverlay`].
    hex_input: &'a mut TextInput<'b, InternalMessage, InternalTheme<Theme>, Renderer>,
    /// The message that is sent if the cancel button of the [`ColorPickerOverlay`] is pressed or
    /// when pressed outside its bounds.
    on_cancel: Message,
    /// The function that produces a message when the submit button of the [`ColorPickerOverlay`]
    /// is pressed.
    on_submit: &'a dyn Fn(Color) -> Message,
    /// The position of the [`ColorPickerOverlay`].
    position: Point,
    /// The viewport of the [`ColorPickerOverlay`].
    viewport: Rectangle,
    /// The height of the underlay.
    underlay_height: f32,
    /// The style of the [`ColorPickerOverlay`].
    class: &'a <Theme as Catalog>::Class<'b>,
    /// The reference to the tree holding the state of this overlay.
    tree: &'a mut Tree,
}

impl<'a, 'b, Message, Theme> ColorPickerOverlay<'a, 'b, Message, Theme>
where
    Message: 'a + Clone,
    Theme: 'b
        + Catalog
        + iced::widget::button::Catalog
        + iced::widget::text::Catalog
        + container::Catalog
        + text_input::Catalog
        + Clone,
    'b: 'a,
{
    /// Creates a new [`ColorPickerOverlay`] on the given position.
    #[allow(clippy::too_many_arguments)]
    pub(self) fn new(
        state: &'a mut OverlayState,
        tree: &'a mut Tree,
        on_cancel: Message,
        on_submit: &'a dyn Fn(Color) -> Message,
        position: Point,
        viewport: Rectangle,
        underlay_height: f32,
        class: &'a <Theme as Catalog>::Class<'b>,
        cancel_button: &'a mut Element<'b, Message, Theme>,
        submit_button: &'a mut Element<'b, Message, Theme>,
        hex_input: &'a mut TextInput<'b, InternalMessage, InternalTheme<Theme>>,
    ) -> Self {
        ColorPickerOverlay {
            state,
            cancel_button,
            submit_button,
            hex_input,
            on_cancel,
            on_submit,
            position,
            viewport,
            underlay_height,
            class,
            tree,
        }
    }

    /// Turn this [`ColorPickerOverlay`] into an overlay [`Element`](overlay::Element).
    #[must_use]
    pub fn overlay(self) -> overlay::Element<'a, Message, Theme, Renderer> {
        overlay::Element::new(Box::new(self))
    }

    /// The event handling for the HSV color area.
    fn update_hsv_color(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        shell: &mut Shell<Message>,
    ) {
        let mut hsv_color_children = layout.children();

        let hsv_color: Hsv = self.state.color.into();
        let mut color_changed = false;

        let sat_value_bounds = hsv_color_children
            .next()
            .expect("widget: Layout should have a sat/value layout")
            .bounds();
        let hue_bounds = hsv_color_children
            .next()
            .expect("widget: Layout should have a hue layout")
            .bounds();

        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => match delta {
                mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => {
                    let move_value =
                        |value: u16, y: f32| ((i32::from(value) + y as i32).rem_euclid(360)) as u16;

                    if cursor.is_over(hue_bounds) {
                        self.state.current_hue_degrees = move_value(hsv_color.hue, *y);
                        self.state.color = Color {
                            a: self.state.color.a,
                            ..Hsv {
                                hue: self.state.current_hue_degrees,
                                ..hsv_color
                            }
                            .into()
                        };
                        color_changed = true;
                    }
                }
            },
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if cursor.is_over(sat_value_bounds) {
                    self.state.color_bar_dragged = ColorBarDragged::SatValue;
                    self.state.focus = Focus::SatValue;
                }
                if cursor.is_over(hue_bounds) {
                    self.state.color_bar_dragged = ColorBarDragged::Hue;
                    self.state.focus = Focus::Hue;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. }) => {
                if matches!(
                    self.state.color_bar_dragged,
                    ColorBarDragged::SatValue | ColorBarDragged::Hue
                ) {
                    shell.capture_event();
                    self.state.color_bar_dragged = ColorBarDragged::None;
                }
            }
            _ => {}
        }

        let calc_percentage_sat =
            |cursor_position: Point| (cursor_position.x.max(0.0) / sat_value_bounds.width).min(1.0);

        let calc_percentage_value = |cursor_position: Point| {
            (cursor_position.y.max(0.0) / sat_value_bounds.height).min(1.0)
        };

        let calc_hue = |cursor_position: Point| {
            ((cursor_position.x.max(0.0) / hue_bounds.width).min(1.0) * 360.0) as u16
        };

        match self.state.color_bar_dragged {
            ColorBarDragged::SatValue => {
                self.state.color = Color {
                    a: self.state.color.a,
                    ..Hsv {
                        saturation: cursor
                            .position()
                            .map(|position| {
                                calc_percentage_sat(Point {
                                    x: (position.x - sat_value_bounds.x)
                                        .max(0.0)
                                        .min(sat_value_bounds.width),
                                    y: (position.y - sat_value_bounds.y)
                                        .max(0.0)
                                        .min(sat_value_bounds.height),
                                })
                            })
                            .unwrap_or(hsv_color.saturation),
                        value: cursor
                            .position()
                            .map(|position| {
                                calc_percentage_value(Point {
                                    x: (position.x - sat_value_bounds.x)
                                        .max(0.0)
                                        .min(sat_value_bounds.width),
                                    y: (position.y - sat_value_bounds.y)
                                        .max(0.0)
                                        .min(sat_value_bounds.height),
                                })
                            })
                            .unwrap_or(hsv_color.value),
                        hue: self.state.current_hue_degrees,
                    }
                    .into()
                };
                color_changed = true;
            }
            ColorBarDragged::Hue => {
                self.state.current_hue_degrees = cursor
                    .position()
                    .map(|position| {
                        calc_hue(Point {
                            x: (position.x - hue_bounds.x).max(0.0).min(hue_bounds.width),
                            y: (position.y - hue_bounds.y).max(0.0).min(hue_bounds.height),
                        })
                    })
                    .unwrap_or(self.state.current_hue_degrees);

                self.state.color = Color {
                    a: self.state.color.a,
                    ..Hsv {
                        hue: self.state.current_hue_degrees,
                        ..hsv_color
                    }
                    .into()
                };
                color_changed = true;
            }
            _ => {}
        }

        if color_changed {
            shell.capture_event();
            shell.request_redraw();
        }
    }

    /// The event handling for the RGBA color area.
    #[allow(clippy::too_many_lines)]
    fn update_rgba_color(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        hex_input_layout: Layout<'_>,
        cursor: Cursor,
        shell: &mut Shell<Message>,
    ) {
        let mut rgba_color_children = layout.children();
        let mut color_changed = false;

        let mut red_row_children = rgba_color_children
            .next()
            .expect("widget: Layout should have a red row layout")
            .children();
        let _ = red_row_children.next();
        let red_bar_bounds = red_row_children
            .next()
            .expect("widget: Layout should have a red bar layout")
            .bounds();

        let mut green_row_children = rgba_color_children
            .next()
            .expect("widget: Layout should have a green row layout")
            .children();
        let _ = green_row_children.next();
        let green_bar_bounds = green_row_children
            .next()
            .expect("widget: Layout should have a green bar layout")
            .bounds();

        let mut blue_row_children = rgba_color_children
            .next()
            .expect("widget: Layout should have a blue row layout")
            .children();
        let _ = blue_row_children.next();
        let blue_bar_bounds = blue_row_children
            .next()
            .expect("widget: Layout should have a blue bar layout")
            .bounds();

        let mut alpha_row_children = rgba_color_children
            .next()
            .expect("widget: Layout should have an alpha row layout")
            .children();
        let _ = alpha_row_children.next();
        let alpha_bar_bounds = alpha_row_children
            .next()
            .expect("widget: Layout should have an alpha bar layout")
            .bounds();

        let hex_input_bounds = hex_input_layout
            .children()
            .next()
            .expect("widget: Layout should have an hex input layout")
            .bounds();

        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => match delta {
                mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => {
                    let move_value =
                        //|value: f32, y: f32| (value * 255.0 + y).clamp(0.0, 255.0) / 255.0;
                        |value: f32, y: f32| value.mul_add(255.0, y).clamp(0.0, 255.0) / 255.0;

                    if cursor.is_over(red_bar_bounds) {
                        self.state.color = Color {
                            r: move_value(self.state.color.r, *y),
                            ..self.state.color
                        };
                        color_changed = true;
                    }
                    if cursor.is_over(green_bar_bounds) {
                        self.state.color = Color {
                            g: move_value(self.state.color.g, *y),
                            ..self.state.color
                        };
                        color_changed = true;
                    }
                    if cursor.is_over(blue_bar_bounds) {
                        self.state.color = Color {
                            b: move_value(self.state.color.b, *y),
                            ..self.state.color
                        };
                        color_changed = true;
                    }
                    if cursor.is_over(alpha_bar_bounds) {
                        self.state.color = Color {
                            a: move_value(self.state.color.a, *y),
                            ..self.state.color
                        };
                        color_changed = true;
                    }
                }
            },
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if cursor.is_over(red_bar_bounds) {
                    self.state.color_bar_dragged = ColorBarDragged::Red;
                    self.state.focus = Focus::Red;
                }
                if cursor.is_over(green_bar_bounds) {
                    self.state.color_bar_dragged = ColorBarDragged::Green;
                    self.state.focus = Focus::Green;
                }
                if cursor.is_over(blue_bar_bounds) {
                    self.state.color_bar_dragged = ColorBarDragged::Blue;
                    self.state.focus = Focus::Blue;
                }
                if cursor.is_over(alpha_bar_bounds) {
                    self.state.color_bar_dragged = ColorBarDragged::Alpha;
                    self.state.focus = Focus::Alpha;
                }
                if cursor.is_over(hex_input_bounds) {
                    self.state.focus = Focus::None;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. }) => {
                if matches!(
                    self.state.color_bar_dragged,
                    ColorBarDragged::Red
                        | ColorBarDragged::Green
                        | ColorBarDragged::Blue
                        | ColorBarDragged::Alpha
                ) {
                    shell.capture_event();
                    self.state.color_bar_dragged = ColorBarDragged::None;
                }
            }
            _ => {}
        }

        let calc_percentage = |bounds: Rectangle, cursor_position: Point| {
            (cursor_position.x.max(0.0) / bounds.width).min(1.0)
        };

        match self.state.color_bar_dragged {
            ColorBarDragged::Red => {
                self.state.color = Color {
                    r: cursor
                        .position()
                        .map(|position| {
                            calc_percentage(
                                red_bar_bounds,
                                Point {
                                    x: (position.x - red_bar_bounds.x)
                                        .max(0.0)
                                        .min(red_bar_bounds.width),
                                    y: (position.y - red_bar_bounds.y)
                                        .max(0.0)
                                        .min(red_bar_bounds.height),
                                },
                            )
                        })
                        .unwrap_or(self.state.color.r),
                    ..self.state.color
                };
                color_changed = true;
            }
            ColorBarDragged::Green => {
                self.state.color = Color {
                    g: cursor
                        .position()
                        .map(|position| {
                            calc_percentage(
                                green_bar_bounds,
                                Point {
                                    x: (position.x - green_bar_bounds.x)
                                        .max(0.0)
                                        .min(green_bar_bounds.width),
                                    y: (position.y - green_bar_bounds.y)
                                        .max(0.0)
                                        .min(green_bar_bounds.height),
                                },
                            )
                        })
                        .unwrap_or(self.state.color.g),
                    ..self.state.color
                };
                color_changed = true;
            }
            ColorBarDragged::Blue => {
                self.state.color = Color {
                    b: cursor
                        .position()
                        .map(|position| {
                            calc_percentage(
                                blue_bar_bounds,
                                Point {
                                    x: (position.x - blue_bar_bounds.x)
                                        .max(0.0)
                                        .min(blue_bar_bounds.width),
                                    y: (position.y - blue_bar_bounds.y)
                                        .max(0.0)
                                        .min(blue_bar_bounds.height),
                                },
                            )
                        })
                        .unwrap_or(self.state.color.b),
                    ..self.state.color
                };
                color_changed = true;
            }
            ColorBarDragged::Alpha => {
                self.state.color = Color {
                    a: cursor
                        .position()
                        .map(|position| {
                            calc_percentage(
                                alpha_bar_bounds,
                                Point {
                                    x: (position.x - alpha_bar_bounds.x)
                                        .max(0.0)
                                        .min(alpha_bar_bounds.width),
                                    y: (position.y - alpha_bar_bounds.y)
                                        .max(0.0)
                                        .min(alpha_bar_bounds.height),
                                },
                            )
                        })
                        .unwrap_or(self.state.color.a),
                    ..self.state.color
                };
                color_changed = true;
            }
            _ => {}
        }

        if color_changed {
            if !matches!(self.state.color, Color::BLACK | Color::WHITE) {
                // Black (0x000000) and White (0xFFFFFF) colors don't have hue, so we keep it as it
                // was on those cases
                let hue = Hsv::from(self.state.color).hue;
                self.state.current_hue_degrees = hue;
            }
            shell.capture_event();
            shell.request_redraw();
        }
    }

    /// The even handling for the keyboard input.
    fn on_event_keyboard(&mut self, event: &Event) -> event::Status {
        if let Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) = event {
            let mut status = event::Status::Ignored;

            if matches!(key, keyboard::Key::Named(keyboard::key::Named::Tab)) {
                if self.state.keyboard_modifiers.shift() {
                    self.state.focus = self.state.focus.previous();
                } else {
                    self.state.focus = self.state.focus.next();
                }
                status = event::Status::Captured;
            } else {
                let sat_value_handle = |key_code: &keyboard::Key, color: &mut Color, hue: u16| {
                    let mut hsv_color: Hsv = (*color).into();
                    let mut status = event::Status::Ignored;

                    match key_code {
                        keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                            hsv_color.saturation -= SAT_VALUE_STEP;
                            status = event::Status::Captured;
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                            hsv_color.saturation += SAT_VALUE_STEP;
                            status = event::Status::Captured;
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                            hsv_color.value -= SAT_VALUE_STEP;
                            status = event::Status::Captured;
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                            hsv_color.value += SAT_VALUE_STEP;
                            status = event::Status::Captured;
                        }
                        _ => {}
                    }

                    hsv_color.saturation = hsv_color.saturation.clamp(0.0, 1.0);
                    hsv_color.value = hsv_color.value.clamp(0.0, 1.0);
                    hsv_color.hue = hue;

                    *color = Color {
                        a: color.a,
                        ..hsv_color.into()
                    };
                    status
                };

                let mut hue_handle = |key_code: &keyboard::Key, color: &mut Color| {
                    let mut hsv_color: Hsv = (*color).into();
                    let mut status = event::Status::Ignored;

                    let mut value = i32::from(self.state.current_hue_degrees);

                    match key_code {
                        keyboard::Key::Named(
                            keyboard::key::Named::ArrowLeft | keyboard::key::Named::ArrowDown,
                        ) => {
                            value -= HUE_STEP;
                            status = event::Status::Captured;
                        }
                        keyboard::Key::Named(
                            keyboard::key::Named::ArrowRight | keyboard::key::Named::ArrowUp,
                        ) => {
                            value += HUE_STEP;
                            status = event::Status::Captured;
                        }
                        _ => {}
                    }

                    hsv_color.hue = value.rem_euclid(360) as u16;
                    self.state.current_hue_degrees = hsv_color.hue;

                    *color = Color {
                        a: color.a,
                        ..hsv_color.into()
                    };

                    status
                };

                let rgba_bar_handle = |key_code: &keyboard::Key, value: &mut f32| {
                    let mut byte_value = (*value * 255.0) as i16;
                    let mut status = event::Status::Ignored;

                    match key_code {
                        keyboard::Key::Named(
                            keyboard::key::Named::ArrowLeft | keyboard::key::Named::ArrowDown,
                        ) => {
                            byte_value -= RGBA_STEP;
                            status = event::Status::Captured;
                        }
                        keyboard::Key::Named(
                            keyboard::key::Named::ArrowRight | keyboard::key::Named::ArrowUp,
                        ) => {
                            byte_value += RGBA_STEP;
                            status = event::Status::Captured;
                        }
                        _ => {}
                    }
                    *value = f32::from(byte_value.clamp(0, 255)) / 255.0;

                    status
                };

                match self.state.focus {
                    Focus::SatValue => {
                        status = sat_value_handle(
                            key,
                            &mut self.state.color,
                            self.state.current_hue_degrees,
                        )
                    }
                    Focus::Hue => status = hue_handle(key, &mut self.state.color),
                    Focus::Red => status = rgba_bar_handle(key, &mut self.state.color.r),
                    Focus::Green => status = rgba_bar_handle(key, &mut self.state.color.g),
                    Focus::Blue => status = rgba_bar_handle(key, &mut self.state.color.b),
                    Focus::Alpha => status = rgba_bar_handle(key, &mut self.state.color.a),
                    _ => {}
                }
            }

            if matches!(status, event::Status::Captured)
                && !matches!(self.state.color, Color::BLACK | Color::WHITE)
            {
                // Black (0x000000) and White (0xFFFFFF) colors don't have hue, so we keep it as it
                // was on those cases
                let hue = Hsv::from(self.state.color).hue;
                self.state.current_hue_degrees = hue;
            }

            status
        } else if let Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) = event {
            self.state.keyboard_modifiers = *modifiers;
            event::Status::Ignored
        } else {
            event::Status::Ignored
        }
    }

    fn update_mouse(
        &self,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let mut children = layout.children();

        let mouse_interaction = if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Idle
        } else {
            mouse::Interaction::None
        };

        if !matches!(self.state.color_bar_dragged, ColorBarDragged::None) {
            // If a bar is being dragged keep the mouse interaction the same as the one from the
            // dragged bar
            return match self.state.color_bar_dragged {
                ColorBarDragged::None => mouse::Interaction::default(),
                ColorBarDragged::SatValue | ColorBarDragged::Hue => mouse::Interaction::Pointer,
                ColorBarDragged::Red
                | ColorBarDragged::Green
                | ColorBarDragged::Blue
                | ColorBarDragged::Alpha => mouse::Interaction::ResizingHorizontally,
            };
        }

        // Block 1
        let block1_layout = children
            .next()
            .expect("Graphics: Layout should have a 1. block layout");
        let mut block1_mouse_interaction = mouse::Interaction::default();
        // HSV color
        let mut hsv_color_children = block1_layout.children();
        let sat_value_layout = hsv_color_children
            .next()
            .expect("Graphics: Layout should have a sat/value layout");
        if cursor.is_over(sat_value_layout.bounds()) {
            block1_mouse_interaction = block1_mouse_interaction.max(mouse::Interaction::Pointer);
        }
        let hue_layout = hsv_color_children
            .next()
            .expect("Graphics: Layout should have a hue layout");
        if cursor.is_over(hue_layout.bounds()) {
            block1_mouse_interaction = block1_mouse_interaction.max(mouse::Interaction::Pointer);
        }

        // Block 2
        let block2_layout = children
            .next()
            .expect("Graphics: Layout should have a 2. block layout");
        let mut block2_mouse_interaction = mouse::Interaction::default();
        let mut block2_children = block2_layout.children();
        // RGBA color
        let rgba_color_layout = block2_children
            .next()
            .expect("Graphics: Layout should have a RGBA color layout");
        let mut rgba_color_children = rgba_color_layout.children();

        let f = |layout: Layout<'_>, cursor: Cursor| {
            let mut children = layout.children();

            let _label_layout = children.next();
            let bar_layout = children
                .next()
                .expect("Graphics: Layout should have a bar layout");

            if cursor.is_over(bar_layout.bounds()) {
                mouse::Interaction::ResizingHorizontally
            } else {
                mouse::Interaction::default()
            }
        };
        let red_row_layout = rgba_color_children
            .next()
            .expect("Graphics: Layout should have a red row layout");
        block2_mouse_interaction = block2_mouse_interaction.max(f(red_row_layout, cursor));
        let green_row_layout = rgba_color_children
            .next()
            .expect("Graphics: Layout should have a green row layout");
        block2_mouse_interaction = block2_mouse_interaction.max(f(green_row_layout, cursor));
        let blue_row_layout = rgba_color_children
            .next()
            .expect("Graphics: Layout should have a blue row layout");
        block2_mouse_interaction = block2_mouse_interaction.max(f(blue_row_layout, cursor));
        let alpha_row_layout = rgba_color_children
            .next()
            .expect("Graphics: Layout should have an alpha row layout");
        block2_mouse_interaction = block2_mouse_interaction.max(f(alpha_row_layout, cursor));

        let hex_input_layout = block2_children
            .next()
            .expect("Graphics: Layout should have an hex input layout for a ColorPicker");
        let hex_input_interaction = self.hex_input.mouse_interaction(
            &self.tree.children[3],
            hex_input_layout,
            cursor,
            viewport,
            renderer,
        );

        // Buttons
        let cancel_button_layout = block2_children
            .next()
            .expect("Graphics: Layout should have a cancel button layout for a ColorPicker");
        let cancel_mouse_interaction = self.cancel_button.as_widget().mouse_interaction(
            &self.tree.children[0],
            cancel_button_layout,
            cursor,
            viewport,
            renderer,
        );

        let submit_button_layout = block2_children
            .next()
            .expect("Graphics: Layout should have a submit button layout for a ColorPicker");
        let submit_mouse_interaction = self.submit_button.as_widget().mouse_interaction(
            &self.tree.children[1],
            submit_button_layout,
            cursor,
            viewport,
            renderer,
        );

        mouse_interaction
            .max(block1_mouse_interaction)
            .max(block2_mouse_interaction)
            .max(hex_input_interaction)
            .max(cancel_mouse_interaction)
            .max(submit_mouse_interaction)
    }
}

impl<'a, 'b, Message, Theme> Overlay<Message, Theme, Renderer>
    for ColorPickerOverlay<'a, 'b, Message, Theme>
where
    Message: 'a + Clone,
    Theme: 'b
        + Catalog
        + iced::widget::button::Catalog
        + iced::widget::text::Catalog
        + container::Catalog
        + text_input::Catalog
        + Clone,
    'b: 'a,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> Node {
        let space_below = bounds.height - (self.position.y + self.underlay_height + OFFSET);
        let space_above = self.position.y - OFFSET;
        let screen_bounds = bounds;

        let mut show_below = false;
        let bounds = Size::new(
            bounds.width,
            if space_below > space_above {
                show_below = true;
                space_below
            } else {
                space_above
            },
        );

        let position = if show_below {
            self.position + Vector::new(0.0, self.underlay_height + OFFSET)
        } else {
            self.position - Vector::new(0.0, OFFSET)
        };

        let (max_width, max_height) = if bounds.width > bounds.height {
            (600.0, 300.0)
        } else {
            (300.0, 600.0)
        };

        let limits = Limits::new(Size::ZERO, bounds)
            .shrink(PADDING)
            .width(Length::Fill)
            .height(Length::Fill)
            .max_width(max_width)
            .max_height(max_height);

        let divider = if bounds.width > bounds.height {
            Row::<(), Theme, Renderer>::new()
                .spacing(SPACING)
                .push(Row::new().width(Length::Fill).height(Length::Fill))
                .push(Row::new().width(Length::Fill).height(Length::Fill))
                .layout(&mut self.tree.children[2], renderer, &limits)
        } else {
            Column::<(), Theme, Renderer>::new()
                .spacing(SPACING)
                .push(Row::new().width(Length::Fill).height(Length::Fill))
                .push(Row::new().width(Length::Fill).height(Length::Fill))
                .layout(&mut self.tree.children[2], renderer, &limits)
        };

        let mut divider_children = divider.children().iter();

        let block1_bounds = divider_children
            .next()
            .expect("Divider should have a first child")
            .bounds();
        let block2_bounds = divider_children
            .next()
            .expect("Divider should have a second child")
            .bounds();

        // ----------- Block 1 ----------------------
        let block1_node = block1_layout(self, renderer, block1_bounds);

        // ----------- Block 2 ----------------------
        let block2_node = block2_layout(self, renderer, block2_bounds);

        let (width, height) = if bounds.width > bounds.height {
            (
                block1_node.size().width + block2_node.size().width + SPACING.0, // + (2.0 * PADDING as f32),
                block2_node.size().height,
            )
        } else {
            (
                block2_node.size().width,
                block1_node.size().height + block2_node.size().height + SPACING.0,
            )
        };

        let mut node =
            Node::with_children(Size::new(width, height), vec![block1_node, block2_node]);

        node.position_and_bounce(position, screen_bounds, show_below);
        node
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<Message>,
    ) {
        if event::Status::Captured == self.on_event_keyboard(event) {
            self.state.clear();
            shell.capture_event();
            shell.request_redraw();
            return;
        }

        let mut children = layout.children();

        // ----------- Block 1 ----------------------
        let block1_layout = children
            .next()
            .expect("widget: Layout should have a 1. block layout");
        self.update_hsv_color(event, block1_layout, cursor, shell);
        // ----------- Block 1 end ------------------

        // ----------- Block 2 ----------------------
        let mut block2_children = children
            .next()
            .expect("widget: Layout should have a 2. block layout")
            .children();

        let rgba_color_layout = block2_children
            .next()
            .expect("widget: Layout should have a RGBA color layout");
        let hex_input_layout = block2_children
            .next()
            .expect("widget: Layout should have a hex text layout");
        // ----------- RGB Color -----------------------
        self.update_rgba_color(event, rgba_color_layout, hex_input_layout, cursor, shell);

        // ----------- Hex input ----------------------
        let mut fake_input_messages: Vec<InternalMessage> = Vec::new();
        let mut fake_input_shell = Shell::new(&mut fake_input_messages);

        // if event was already captured, mark this fake shell with a captured_event as well
        if shell.event_status() == event::Status::Captured {
            fake_input_shell.capture_event();
        }

        let mut should_update_input = true;
        self.hex_input.update(
            &mut self.tree.children[3],
            event,
            hex_input_layout,
            cursor,
            renderer,
            clipboard,
            &mut fake_input_shell,
            &layout.bounds(),
        );

        // Reconcile redraws and captures
        match fake_input_shell.redraw_request() {
            iced::window::RedrawRequest::NextFrame => shell.request_redraw(),
            iced::window::RedrawRequest::At(instant) => shell.request_redraw_at(instant),
            iced::window::RedrawRequest::Wait => {}
        }
        if fake_input_shell.event_status() == event::Status::Captured {
            shell.capture_event();
        }

        if !fake_input_messages.is_empty() {
            // Update the selected color if string can be parsed
            let InternalMessage::ChangeInput(color_str) = &fake_input_messages[0];

            if let Some(color) = iced::Color::parse(color_str) {
                self.state.color = color;
                if !matches!(self.state.color, Color::BLACK | Color::WHITE) {
                    // Black (0x000000) and White (0xFFFFFF) colors don't have hue, so we keep it as it
                    // was on those cases
                    let hue = Hsv::from(self.state.color).hue;
                    self.state.current_hue_degrees = hue;
                }
            } else {
                should_update_input = false;
            }
            shell.capture_event();
            shell.request_redraw();
        }
        // ----------- Buttons -------------------------
        let cancel_button_layout = block2_children
            .next()
            .expect("widget: Layout should have a cancel button layout for a ColorPicker");

        self.cancel_button.as_widget_mut().update(
            &mut self.tree.children[0],
            event,
            cancel_button_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &layout.bounds(),
        );

        let mut fake_messages: Vec<Message> = Vec::new();
        let mut fake_submit_shell = Shell::new(&mut fake_messages);

        // if event was already captured, mark this fake shell with a captured_event as well
        if shell.event_status() == event::Status::Captured {
            fake_submit_shell.capture_event();
        }

        let submit_button_layout = block2_children
            .next()
            .expect("widget: Layout should have a submit button layout for a ColorPicker");
        self.submit_button.as_widget_mut().update(
            &mut self.tree.children[1],
            event,
            submit_button_layout,
            cursor,
            renderer,
            clipboard,
            &mut fake_submit_shell,
            &layout.bounds(),
        );

        // Reconcile redraws and captures
        match fake_submit_shell.redraw_request() {
            iced::window::RedrawRequest::NextFrame => shell.request_redraw(),
            iced::window::RedrawRequest::At(instant) => shell.request_redraw_at(instant),
            iced::window::RedrawRequest::Wait => {}
        }
        if fake_submit_shell.event_status() == event::Status::Captured {
            shell.capture_event();
        }

        if !fake_messages.is_empty() {
            // Submit the selected color
            shell.publish((self.on_submit)(self.state.color));
            self.state.previous_hue_degrees = self.state.current_hue_degrees;

            shell.capture_event();
            shell.request_redraw();
        }
        // ----------- Block 2 end ------------------

        if shell.is_event_captured() {
            if should_update_input {
                let new_hex_input: TextInput<InternalMessage, InternalTheme<Theme>> = text_input(
                    &self.state.color.as_hex_string(),
                    &self.state.color.as_hex_string(),
                )
                .on_input(InternalMessage::ChangeInput)
                .padding(PADDING)
                .align_x(iced::Alignment::Center);
                self.tree.children[3].diff(&new_hex_input as &dyn Widget<_, _, _>);
                *self.hex_input = new_hex_input;
            }
            self.state.clear();
        } else if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerLifted { .. })
                | Event::Touch(touch::Event::FingerLost { .. })
                | Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. })
        ) && !cursor.is_over(layout.bounds())
        {
            // Clicked outside of bounds so lets send the `on_cancel` message to close the picker
            shell.publish(self.on_cancel.clone());
            shell.capture_event();
            self.state.clear();
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.update_mouse(layout, cursor, &self.viewport, renderer)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
    ) {
        let bounds = layout.bounds();
        let mut children = layout.children();

        let mut style_sheet: HashMap<StyleState, Style> = HashMap::new();
        let _ = style_sheet.insert(
            StyleState::Active,
            Catalog::style(theme, self.class, Status::Active),
        );
        let _ = style_sheet.insert(
            StyleState::Selected,
            Catalog::style(theme, self.class, Status::Selected),
        );
        let _ = style_sheet.insert(
            StyleState::Hovered,
            Catalog::style(theme, self.class, Status::Hovered),
        );
        let _ = style_sheet.insert(
            StyleState::Focused,
            Catalog::style(theme, self.class, Status::Focused),
        );

        let mut style_state = StyleState::Active;
        if self.state.focus == Focus::Overlay {
            style_state = style_state.max(StyleState::Focused);
        }
        if cursor.is_over(bounds) {
            style_state = style_state.max(StyleState::Hovered);
        }

        if (bounds.width > 0.) && (bounds.height > 0.) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        radius: style_sheet[&style_state].border_radius.into(),
                        width: style_sheet[&style_state].border_width,
                        color: style_sheet[&style_state].border_color,
                    },
                    shadow: Shadow::default(),
                },
                style_sheet[&style_state].background,
            );
        }

        // ----------- Block 1 ----------------------
        let block1_layout = children
            .next()
            .expect("Graphics: Layout should have a 1. block layout");
        block1(renderer, self, block1_layout, cursor, &style_sheet);

        // ----------- Block 2 ----------------------
        let block2_layout = children
            .next()
            .expect("Graphics: Layout should have a 2. block layout");
        block2(
            renderer,
            self,
            block2_layout,
            cursor,
            theme,
            style,
            &bounds,
            &style_sheet,
        );
    }
}

pub fn overlay_element<'a, Message, Theme>(
    color: Color,
    on_cancel: Message,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a
        + Catalog
        + iced::widget::button::Catalog
        + iced::widget::text::Catalog
        + container::Catalog
        + text_input::Catalog,
{
    // RGBA Colors
    let mut rgba_colors: Column<'_, Message, Theme, Renderer> =
        Column::<Message, Theme, Renderer>::new();

    for _ in 0..4 {
        rgba_colors = rgba_colors.push(
            Row::new()
                .align_y(Alignment::Center)
                .spacing(SPACING)
                .padding(PADDING)
                .height(Length::Fill)
                .push(
                    text("X: ")
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                )
                .push(
                    Row::new()
                        .width(Length::FillPortion(5))
                        .height(Length::Fill),
                )
                .push(
                    text("XXX")
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                ),
        );
    }

    let hex_input = text_input("0x00000000", "0x00000000")
        .padding(PADDING)
        .align_x(Alignment::Center);

    let cancel_button = Button::new(
        text("X")
            .align_x(Horizontal::Center)
            .font(iced::Font {
                weight: iced::font::Weight::ExtraBold,
                ..iced::Font::DEFAULT
            })
            .width(Length::Fill),
    )
    .width(Length::Fill);
    let submit_button = Button::new(
        text(Renderer::CHECKMARK_ICON)
            .font(Renderer::ICON_FONT)
            .align_x(Horizontal::Center)
            .width(Length::Fill),
    )
    .width(Length::Fill);

    let block2 = Column::new()
        .spacing(PADDING.vertical() / 2.) // Average vertical padding
        .push(rgba_colors)
        .push(hex_input)
        .push(
            Row::new()
                .spacing(PADDING.vertical() / 2.)
                .push(cancel_button)
                .push(submit_button),
        );

    let block1 = Column::<Message, Theme, Renderer>::new()
        .spacing(PADDING.vertical() / 2.) // Average vertical padding
        .push(
            Row::new().push(
                container("")
                    .width(Length::Fill)
                    .height(Length::FillPortion(7)),
            ),
        )
        .push(
            Row::new().push(
                container("")
                    .width(Length::Fill)
                    .height(Length::FillPortion(1)),
            ),
        );

    let divider = Row::<Message, Theme, Renderer>::new()
        .spacing(SPACING)
        .push(Row::new().push(container(block1).width(Length::Fill).height(Length::Fill)))
        .push(Row::new().push(container(block2).width(Length::Fill).height(Length::Fill)))
        .width(600)
        .height(300);

    let element: Element<Message, Theme, Renderer> = Element::new(divider);

    let cancel_button = Button::new(
        text("X")
            .align_x(Horizontal::Center)
            .font(iced::Font {
                weight: iced::font::Weight::ExtraBold,
                ..iced::Font::DEFAULT
            })
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .on_press(on_cancel.clone());
    let submit_button = Button::new(
        text(Renderer::CHECKMARK_ICON)
            .font(Renderer::ICON_FONT)
            .align_x(Horizontal::Center)
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .on_press(on_cancel.clone()); // Sending a fake message
    let hex_input = text_input(&color.as_hex_string(), &color.as_hex_string())
        .on_input(move |_| on_cancel.clone())
        .padding(PADDING);

    row![cancel_button, submit_button, element, hex_input].into()
}

/// Defines the layout of the 1. block of the color picker containing the HSV part.
fn block1_layout<'a, Message, Theme>(
    color_picker: &mut ColorPickerOverlay<'_, '_, Message, Theme>,
    renderer: &Renderer,
    bounds: Rectangle,
) -> Node
where
    Message: 'a + Clone,
    Theme: 'a
        + Catalog
        + iced::widget::button::Catalog
        + iced::widget::text::Catalog
        + text_input::Catalog
        + Clone,
{
    let block1_limits = Limits::new(Size::ZERO, bounds.size())
        .width(Length::Fill)
        .height(Length::Fill);

    let block1_node = Column::<(), Theme, Renderer>::new()
        .spacing(PADDING.vertical() / 2.) // Average vertical padding
        .push(
            Row::new()
                .width(Length::Fill)
                .height(Length::FillPortion(7)),
        )
        .push(
            Row::new()
                .width(Length::Fill)
                .height(Length::FillPortion(1)),
        )
        .layout(&mut color_picker.tree.children[2], renderer, &block1_limits);

    block1_node.move_to(Point::new(bounds.x + PADDING.left, bounds.y + PADDING.top))
}

/// Defines the layout of the 2. block of the color picker containing the RGBA part, Hex and buttons.
fn block2_layout<'a, Message, Theme>(
    color_picker: &mut ColorPickerOverlay<'a, '_, Message, Theme>,
    renderer: &Renderer,
    bounds: Rectangle,
) -> Node
where
    Message: 'a + Clone,
    Theme: 'a
        + Catalog
        + iced::widget::button::Catalog
        + iced::widget::text::Catalog
        + text_input::Catalog
        + Clone,
{
    let block2_limits = Limits::new(Size::ZERO, bounds.size())
        .width(Length::Fill)
        .height(Length::Fill);

    // Pre-Buttons TODO: get rid of it
    let cancel_limits = block2_limits;
    let cancel_button = color_picker.cancel_button.as_widget().layout(
        &mut color_picker.tree.children[0],
        renderer,
        &cancel_limits,
    );

    let hex_text_limits = block2_limits;

    let mut hex_text_layout = (color_picker.hex_input as &mut dyn Widget<_, _, _>).layout(
        &mut color_picker.tree.children[3],
        renderer,
        &hex_text_limits,
    );

    let block2_limits = block2_limits.shrink(Size::new(
        0.0,
        cancel_button.bounds().height + hex_text_layout.bounds().height + 2.0 * SPACING.0,
    ));

    // RGBA Colors
    let mut rgba_colors: Column<'_, Message, Theme, Renderer> =
        Column::<Message, Theme, Renderer>::new();

    for _ in 0..4 {
        rgba_colors = rgba_colors.push(
            Row::new()
                .align_y(Alignment::Center)
                .spacing(SPACING)
                .padding(PADDING)
                .height(Length::Fill)
                .push(
                    text("X: ")
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                )
                .push(
                    Row::new()
                        .width(Length::FillPortion(5))
                        .height(Length::Fill),
                )
                .push(
                    text("XXX")
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                ),
        );
    }
    let element: Element<Message, Theme, Renderer> = Element::new(rgba_colors);
    let rgba_tree = if let Some(child_tree) = color_picker.tree.children[2].children.get_mut(0) {
        child_tree.diff(element.as_widget());
        child_tree
    } else {
        let child_tree = Tree::new(element.as_widget());
        color_picker.tree.children[2].children.insert(0, child_tree);
        &mut color_picker.tree.children[2].children[0]
    };

    let mut rgba_colors = element
        .as_widget()
        .layout(rgba_tree, renderer, &block2_limits);

    let rgba_bounds = rgba_colors.bounds();
    rgba_colors = rgba_colors.move_to(Point::new(
        rgba_bounds.x + PADDING.left,
        rgba_bounds.y + PADDING.top,
    ));
    let rgba_bounds = rgba_colors.bounds();

    // Hex text
    let hex_bounds = hex_text_layout.bounds();
    hex_text_layout = hex_text_layout.move_to(Point::new(
        hex_bounds.x + PADDING.left,
        hex_bounds.y + rgba_bounds.height + PADDING.top + SPACING.0,
    ));
    let hex_bounds = hex_text_layout.bounds();

    // Buttons
    let cancel_limits =
        block2_limits.max_width(((rgba_bounds.width / 2.0) - BUTTON_SPACING.0).max(0.0));

    let mut cancel_button = color_picker.cancel_button.as_widget().layout(
        &mut color_picker.tree.children[0],
        renderer,
        &cancel_limits,
    );

    let submit_limits =
        block2_limits.max_width(((rgba_bounds.width / 2.0) - BUTTON_SPACING.0).max(0.0));

    let mut submit_button = color_picker.submit_button.as_widget().layout(
        &mut color_picker.tree.children[1],
        renderer,
        &submit_limits,
    );

    let cancel_bounds = cancel_button.bounds();
    cancel_button = cancel_button.move_to(Point::new(
        cancel_bounds.x + PADDING.left,
        cancel_bounds.y + rgba_bounds.height + hex_bounds.height + PADDING.top + 2.0 * SPACING.0,
    ));
    let cancel_bounds = cancel_button.bounds();

    let submit_bounds = submit_button.bounds();
    submit_button = submit_button.move_to(Point::new(
        submit_bounds.x + rgba_colors.bounds().width - submit_bounds.width + PADDING.left,
        submit_bounds.y + rgba_bounds.height + hex_bounds.height + PADDING.top + 2.0 * SPACING.0,
    ));

    Node::with_children(
        Size::new(
            rgba_bounds.width + PADDING.horizontal(),
            rgba_bounds.height
                + hex_bounds.height
                + cancel_bounds.height
                + PADDING.vertical()
                + (2.0 * SPACING.0),
        ),
        vec![rgba_colors, hex_text_layout, cancel_button, submit_button],
    )
    .move_to(Point::new(bounds.x, bounds.y))
}

/// Draws the 1. block of the color picker containing the HSV part.
fn block1<Message, Theme>(
    renderer: &mut Renderer,
    color_picker: &ColorPickerOverlay<'_, '_, Message, Theme>,
    layout: Layout<'_>,
    cursor: Cursor,
    style_sheet: &HashMap<StyleState, Style>,
) where
    Message: Clone,
    Theme: Catalog
        + iced::widget::button::Catalog
        + iced::widget::text::Catalog
        + text_input::Catalog
        + Clone,
{
    // ----------- Block 1 ----------------------
    let hsv_color_layout = layout;

    // ----------- HSV Color ----------------------
    //let hsv_color_layout = block1_children.next().unwrap();
    hsv_color(
        renderer,
        color_picker,
        hsv_color_layout,
        cursor,
        style_sheet,
    );

    // ----------- Block 1 end ------------------
}

/// Draws the 2. block of the color picker containing the RGBA part, Hex and buttons.
#[allow(clippy::too_many_arguments)]
fn block2<Message, Theme>(
    renderer: &mut Renderer,
    color_picker: &ColorPickerOverlay<'_, '_, Message, Theme>,
    layout: Layout<'_>,
    cursor: Cursor,
    theme: &Theme,
    style: &renderer::Style,
    viewport: &Rectangle,
    style_sheet: &HashMap<StyleState, Style>,
) where
    Message: Clone,
    Theme: Catalog
        + iced::widget::button::Catalog
        + iced::widget::text::Catalog
        + text_input::Catalog
        + Clone,
{
    // ----------- Block 2 ----------------------
    let mut block2_children = layout.children();

    // ----------- RGBA Color ----------------------
    let rgba_color_layout = block2_children
        .next()
        .expect("Graphics: Layout should have a RGBA color layout");
    rgba_color(
        renderer,
        rgba_color_layout,
        &color_picker.state.color,
        cursor,
        style,
        style_sheet,
        color_picker.state.focus,
    );

    // ----------- Hex text ----------------------
    let hex_text_layout = block2_children
        .next()
        .expect("Graphics: Layout should have a hex text layout");
    let hex_input_theme = InternalTheme {
        theme: theme.clone(),
        color: color_picker.state.color,
    };
    (color_picker.hex_input as &dyn Widget<_, _, _>).draw(
        &color_picker.tree.children[3],
        renderer,
        &hex_input_theme,
        style,
        hex_text_layout,
        cursor,
        viewport,
    );

    // ----------- Buttons -------------------------
    let cancel_button_layout = block2_children
        .next()
        .expect("Graphics: Layout should have a cancel button layout for a ColorPicker");

    color_picker.cancel_button.as_widget().draw(
        &color_picker.tree.children[0],
        renderer,
        theme,
        style,
        cancel_button_layout,
        cursor,
        viewport,
    );

    let submit_button_layout = block2_children
        .next()
        .expect("Graphics: Layout should have a submit button layout for a ColorPicker");

    color_picker.submit_button.as_widget().draw(
        &color_picker.tree.children[1],
        renderer,
        theme,
        style,
        submit_button_layout,
        cursor,
        viewport,
    );

    // Buttons are not focusable right now...
    if color_picker.state.focus == Focus::Cancel {
        let bounds = cancel_button_layout.bounds();
        if (bounds.width > 0.) && (bounds.height > 0.) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        radius: style_sheet[&StyleState::Focused].border_radius.into(),
                        width: style_sheet[&StyleState::Focused].border_width,
                        color: style_sheet[&StyleState::Focused].border_color,
                    },
                    shadow: Shadow::default(),
                },
                Color::TRANSPARENT,
            );
        }
    }

    if color_picker.state.focus == Focus::Submit {
        let bounds = submit_button_layout.bounds();
        if (bounds.width > 0.) && (bounds.height > 0.) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        radius: style_sheet[&StyleState::Focused].border_radius.into(),
                        width: style_sheet[&StyleState::Focused].border_width,
                        color: style_sheet[&StyleState::Focused].border_color,
                    },
                    shadow: Shadow::default(),
                },
                Color::TRANSPARENT,
            );
        }
    }
    // ----------- Block 2 end ------------------
}

/// Draws the HSV color area.
#[allow(clippy::too_many_lines)]
fn hsv_color<Message, Theme>(
    renderer: &mut Renderer,
    color_picker: &ColorPickerOverlay<'_, '_, Message, Theme>,
    layout: Layout<'_>,
    cursor: Cursor,
    style_sheet: &HashMap<StyleState, Style>,
) where
    Message: Clone,
    Theme: Catalog
        + iced::widget::button::Catalog
        + iced::widget::text::Catalog
        + text_input::Catalog
        + Clone,
{
    let mut hsv_color_children = layout.children();
    let hsv_color: Hsv = color_picker.state.color.into();

    let sat_value_layout = hsv_color_children
        .next()
        .expect("Graphics: Layout should have a sat/value layout");
    let mut sat_value_style_state = StyleState::Active;
    if color_picker.state.focus == Focus::SatValue {
        sat_value_style_state = sat_value_style_state.max(StyleState::Focused);
    }
    if cursor.is_over(sat_value_layout.bounds()) {
        sat_value_style_state = sat_value_style_state.max(StyleState::Hovered);
    }

    let geometry = color_picker.state.sat_value_canvas_cache.draw(
        renderer,
        sat_value_layout.bounds().size(),
        |frame| {
            let column_count = frame.width() as u16;
            let row_count = frame.height() as u16;

            for column in 0..column_count {
                for row in 0..row_count {
                    let saturation = f32::from(column) / frame.width();
                    let value = f32::from(row) / frame.height();

                    frame.fill_rectangle(
                        Point::new(f32::from(column), f32::from(row)),
                        Size::new(1.0, 1.0),
                        Color::from(Hsv::from_hsv(
                            color_picker.state.current_hue_degrees,
                            saturation,
                            value,
                        )),
                    );
                }
            }

            let stroke = Stroke {
                style: canvas::Style::Solid(
                    Hsv {
                        hue: 0,
                        saturation: 0.0,
                        value: 1.0 - hsv_color.value,
                    }
                    .into(),
                ),
                width: 3.0,
                line_cap: LineCap::Round,
                ..Stroke::default()
            };

            let saturation = hsv_color.saturation * frame.width();
            let value = hsv_color.value * frame.height();

            frame.stroke(
                &Path::line(
                    Point::new(saturation, 0.0),
                    Point::new(saturation, frame.height()),
                ),
                stroke,
            );

            frame.stroke(
                &Path::line(Point::new(0.0, value), Point::new(frame.width(), value)),
                stroke,
            );

            let stroke = Stroke {
                style: canvas::Style::Solid(
                    style_sheet
                        .get(&sat_value_style_state)
                        .expect("Style Sheet not found.")
                        .bar_border_color,
                ),
                width: 2.0,
                line_cap: LineCap::Round,
                ..Stroke::default()
            };

            frame.stroke(
                &Path::rectangle(
                    Point::new(0.0, 0.0),
                    Size::new(frame.size().width - 0.0, frame.size().height - 0.0),
                ),
                stroke,
            );
        },
    );

    let translation = Vector::new(sat_value_layout.bounds().x, sat_value_layout.bounds().y);
    renderer.with_translation(translation, |renderer| {
        renderer.draw_geometry(geometry);
    });

    let hue_layout = hsv_color_children
        .next()
        .expect("Graphics: Layout should have a hue layout");
    let mut hue_style_state = StyleState::Active;
    if color_picker.state.focus == Focus::Hue {
        hue_style_state = hue_style_state.max(StyleState::Focused);
    }
    if cursor.is_over(hue_layout.bounds()) {
        hue_style_state = hue_style_state.max(StyleState::Hovered);
    }

    let geometry =
        color_picker
            .state
            .hue_canvas_cache
            .draw(renderer, hue_layout.bounds().size(), |frame| {
                let column_count = frame.width() as u16;

                for column in 0..column_count {
                    let hue = (f32::from(column) * 360.0 / frame.width()) as u16;

                    let hsv_color = Hsv::from_hsv(hue, 1.0, 1.0);
                    let stroke = Stroke {
                        style: canvas::Style::Solid(hsv_color.into()),
                        width: 1.0,
                        line_cap: LineCap::Round,
                        ..Stroke::default()
                    };

                    frame.stroke(
                        &Path::line(
                            Point::new(f32::from(column), 0.0),
                            Point::new(f32::from(column), frame.height()),
                        ),
                        stroke,
                    );
                }

                let stroke = Stroke {
                    style: canvas::Style::Solid(Color::BLACK),
                    width: 3.0,
                    line_cap: LineCap::Round,
                    ..Stroke::default()
                };

                let column =
                    f32::from(color_picker.state.current_hue_degrees) * frame.width() / 360.0;

                frame.stroke(
                    &Path::line(Point::new(column, 0.0), Point::new(column, frame.height())),
                    stroke,
                );

                let stroke = Stroke {
                    style: canvas::Style::Solid(
                        style_sheet
                            .get(&hue_style_state)
                            .expect("Style Sheet not found.")
                            .bar_border_color,
                    ),
                    width: 2.0,
                    line_cap: LineCap::Round,
                    ..Stroke::default()
                };

                frame.stroke(
                    &Path::rectangle(
                        Point::new(0.0, 0.0),
                        Size::new(frame.size().width, frame.size().height),
                    ),
                    stroke,
                );
            });

    let translation = Vector::new(hue_layout.bounds().x, hue_layout.bounds().y);
    renderer.with_translation(translation, |renderer| {
        renderer.draw_geometry(geometry);
    });
}

/// Draws the RGBA color area.
#[allow(clippy::too_many_lines)]
fn rgba_color(
    renderer: &mut Renderer,
    layout: Layout<'_>,
    color: &Color,
    cursor: Cursor,
    style: &renderer::Style,
    style_sheet: &HashMap<StyleState, Style>,
    focus: Focus,
) {
    let mut rgba_color_children = layout.children();

    let f = |renderer: &mut Renderer,
             layout: Layout,
             label: &str,
             color: Color,
             value: f32,
             cursor: Cursor,
             target: Focus| {
        let mut children = layout.children();

        let label_layout = children
            .next()
            .expect("Graphics: Layout should have a label layout");
        let bar_layout = children
            .next()
            .expect("Graphics: Layout should have a bar layout");
        let value_layout = children
            .next()
            .expect("Graphics: Layout should have a value layout");

        // Label
        renderer.fill_text(
            Text {
                content: label.to_owned(),
                bounds: Size::new(label_layout.bounds().width, label_layout.bounds().height),
                size: renderer.default_size(),
                // font: REQUIRED_FONT,
                font: iced::Font::default(),
                align_x: Horizontal::Center.into(),
                align_y: Vertical::Center,
                line_height: text::LineHeight::Relative(1.3),
                shaping: text::Shaping::Advanced,
                wrapping: Wrapping::default(),
            },
            Point::new(
                label_layout.bounds().center_x(),
                label_layout.bounds().center_y(),
            ),
            style.text_color,
            label_layout.bounds(),
        );

        let bar_bounds = bar_layout.bounds();

        let bar_style_state = if cursor.is_over(bar_bounds) {
            StyleState::Hovered
        } else {
            StyleState::Active
        };

        // Bar background
        let background_bounds = Rectangle {
            x: bar_bounds.x,
            y: bar_bounds.y,
            width: bar_bounds.width * value,
            height: bar_bounds.height,
        };
        if (background_bounds.width > 0.) && (background_bounds.height > 0.) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: background_bounds,
                    border: Border {
                        radius: style_sheet
                            .get(&bar_style_state)
                            .expect("Style Sheet not found.")
                            .bar_border_radius
                            .into(),
                        width: style_sheet
                            .get(&bar_style_state)
                            .expect("Style Sheet not found.")
                            .bar_border_width,
                        color: Color::TRANSPARENT,
                    },
                    shadow: Shadow::default(),
                },
                color,
            );
        }

        // Bar
        if (bar_bounds.width > 0.) && (bar_bounds.height > 0.) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: bar_bounds,
                    border: Border {
                        radius: style_sheet
                            .get(&bar_style_state)
                            .expect("Style Sheet not found.")
                            .bar_border_radius
                            .into(),
                        width: style_sheet
                            .get(&bar_style_state)
                            .expect("Style Sheet not found.")
                            .bar_border_width,
                        color: style_sheet
                            .get(&bar_style_state)
                            .expect("Style Sheet not found.")
                            .bar_border_color,
                    },
                    shadow: Shadow::default(),
                },
                Color::TRANSPARENT,
            );
        }

        // Value
        renderer.fill_text(
            Text {
                content: format!("{}", (255.0 * value) as u8),
                bounds: Size::new(value_layout.bounds().width, value_layout.bounds().height),
                size: renderer.default_size(),
                font: renderer.default_font(),
                align_x: Horizontal::Center.into(),
                align_y: Vertical::Center,
                line_height: iced::widget::text::LineHeight::Relative(1.3),
                shaping: iced::widget::text::Shaping::Advanced,
                wrapping: Wrapping::default(),
            },
            Point::new(
                value_layout.bounds().center_x(),
                value_layout.bounds().center_y(),
            ),
            style.text_color,
            value_layout.bounds(),
        );

        let bounds = layout.bounds();
        if (focus == target) && (bounds.width > 0.) && (bounds.height > 0.) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        radius: style_sheet
                            .get(&StyleState::Focused)
                            .expect("Style Sheet not found.")
                            .border_radius
                            .into(),
                        width: style_sheet
                            .get(&StyleState::Focused)
                            .expect("Style Sheet not found.")
                            .border_width,
                        color: style_sheet
                            .get(&StyleState::Focused)
                            .expect("Style Sheet not found.")
                            .border_color,
                    },
                    shadow: Shadow::default(),
                },
                Color::TRANSPARENT,
            );
        }
    };

    // Red
    let red_row_layout = rgba_color_children
        .next()
        .expect("Graphics: Layout should have a red row layout");

    f(
        renderer,
        red_row_layout,
        "R:",
        Color::from_rgb(color.r, 0.0, 0.0),
        color.r,
        cursor,
        Focus::Red,
    );

    // Green
    let green_row_layout = rgba_color_children
        .next()
        .expect("Graphics: Layout should have a green row layout");

    f(
        renderer,
        green_row_layout,
        "G:",
        Color::from_rgb(0.0, color.g, 0.0),
        color.g,
        cursor,
        Focus::Green,
    );

    // Blue
    let blue_row_layout = rgba_color_children
        .next()
        .expect("Graphics: Layout should have a blue row layout");

    f(
        renderer,
        blue_row_layout,
        "B:",
        Color::from_rgb(0.0, 0.0, color.b),
        color.b,
        cursor,
        Focus::Blue,
    );

    // Alpha
    let alpha_row_layout = rgba_color_children
        .next()
        .expect("Graphics: Layout should have an alpha row layout");

    f(
        renderer,
        alpha_row_layout,
        "A:",
        Color::from_rgba(0.0, 0.0, 0.0, color.a),
        color.a,
        cursor,
        Focus::Alpha,
    );
}

/// The state of the [`ColorPickerOverlay`].
#[derive(Debug)]
pub struct OverlayState {
    /// The selected color of the [`ColorPickerOverlay`].
    pub(crate) color: Color,
    /// The cache of the sat/value canvas of the [`ColorPickerOverlay`].
    pub(crate) sat_value_canvas_cache: canvas::Cache,
    /// The cache of the hue canvas of the [`ColorPickerOverlay`].
    pub(crate) hue_canvas_cache: canvas::Cache,
    /// The previous hue degrees of the [`ColorPickerOverlay`].
    pub(crate) previous_hue_degrees: u16,
    /// The current hue degrees of the [`ColorPickerOverlay`].
    pub(crate) current_hue_degrees: u16,
    /// The dragged color bar of the [`ColorPickerOverlay`].
    pub(crate) color_bar_dragged: ColorBarDragged,
    /// the focus of the [`ColorPickerOverlay`].
    pub(crate) focus: Focus,
    /// The previously pressed keyboard modifiers.
    pub(crate) keyboard_modifiers: keyboard::Modifiers,
}

impl OverlayState {
    /// Creates a new State with the given color.
    #[must_use]
    pub fn new(color: Color) -> Self {
        let hue = Hsv::from(color).hue;
        Self {
            color,
            previous_hue_degrees: hue,
            current_hue_degrees: hue,
            ..Self::default()
        }
    }

    /// Clears the state cache
    pub fn clear(&mut self) {
        self.sat_value_canvas_cache.clear();
        self.hue_canvas_cache.clear();
    }
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            color: Color::from_rgb(0.5, 0.25, 0.25),
            sat_value_canvas_cache: canvas::Cache::default(),
            hue_canvas_cache: canvas::Cache::default(),
            previous_hue_degrees: 0,
            current_hue_degrees: 0,
            color_bar_dragged: ColorBarDragged::None,
            focus: Focus::default(),
            keyboard_modifiers: keyboard::Modifiers::default(),
        }
    }
}

/// The state of the currently dragged area.
#[derive(Copy, Clone, Debug)]
pub enum ColorBarDragged {
    /// No area is focussed.
    None,

    /// The saturation/value area is focussed.
    SatValue,

    /// The hue area is focussed.
    Hue,

    /// The red area is focussed.
    Red,

    /// The green area is focussed.
    Green,

    /// The blue area is focussed.
    Blue,

    /// The alpha area is focussed.
    Alpha,
}

impl Default for ColorBarDragged {
    fn default() -> Self {
        Self::None
    }
}

/// An enumeration of all focusable element of the [`ColorPickerOverlay`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Focus {
    /// Nothing is in focus.
    None,

    /// The overlay itself is in focus.
    Overlay,

    /// The saturation and value area is in focus.
    SatValue,

    /// The hue bar is in focus.
    Hue,

    /// The red bar is in focus.
    Red,

    /// The green bar is in focus.
    Green,

    /// The blue bar is in focus.
    Blue,

    /// The alpha bar is in focus.
    Alpha,

    /// The cancel button is in focus.
    Cancel,

    /// The submit button is in focus.
    Submit,
}

impl Focus {
    /// Gets the next focusable element.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Overlay => Self::SatValue,
            Self::SatValue => Self::Hue,
            Self::Hue => Self::Red,
            Self::Red => Self::Green,
            Self::Green => Self::Blue,
            Self::Blue => Self::Alpha,
            Self::Alpha => Self::Cancel,
            Self::Cancel => Self::Submit,
            Self::Submit | Self::None => Self::Overlay,
        }
    }

    /// Gets the previous focusable element.
    #[must_use]
    pub const fn previous(self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Overlay => Self::Submit,
            Self::SatValue => Self::Overlay,
            Self::Hue => Self::SatValue,
            Self::Red => Self::Hue,
            Self::Green => Self::Red,
            Self::Blue => Self::Green,
            Self::Alpha => Self::Blue,
            Self::Cancel => Self::Alpha,
            Self::Submit => Self::Cancel,
        }
    }
}

impl Default for Focus {
    fn default() -> Self {
        Self::None
    }
}

/// The appearance of a [`ColorPicker`](crate::widget::ColorPicker).
#[derive(Clone, Copy, Debug)]
pub struct Style {
    /// The background of the [`ColorPicker`](crate::widget::ColorPicker).
    pub background: Background,

    /// The border radius of the [`ColorPicker`](crate::widget::ColorPicker).
    pub border_radius: f32,

    /// The border with of the [`ColorPicker`](crate::widget::ColorPicker).
    pub border_width: f32,

    /// The border color of the [`ColorPicker`](crate::widget::ColorPicker).
    pub border_color: Color,

    /// The border radius of the bars of the [`ColorPicker`](crate::widget::ColorPicker).
    pub bar_border_radius: f32,

    /// The border width of the bars of the [`ColorPicker`](crate::widget::ColorPicker).
    pub bar_border_width: f32,

    /// The border color of the bars of the [`ColorPicker`](crate::widget::ColorPicker).
    pub bar_border_color: Color,
}

/// The Catalog of a [`ColorPicker`](crate::widget::ColorPicker).
pub trait Catalog {
    ///Style for the trait to use.
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self, Style>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(primary)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// The primary theme of a [`ColorPicker`](crate::widget::color_picker::ColorPicker).
#[must_use]
pub fn primary(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let foreground = theme.palette();

    let base = Style {
        background: palette.background.base.color.into(),
        border_radius: 15.0,
        border_width: 1.0,
        border_color: foreground.text,
        bar_border_radius: 5.0,
        bar_border_width: 1.0,
        bar_border_color: foreground.text,
    };

    match status {
        Status::Focused => Style {
            border_color: palette.background.strong.color,
            bar_border_color: palette.background.strong.color,
            ..base
        },
        _ => base,
    }
}

/// Status Enum of an mouse Event.
///
/// The Status of a widget event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// can be pressed.
    Active,
    /// can be pressed and it is being hovered.
    Hovered,
    /// is being pressed.
    Pressed,
    /// cannot be pressed.
    Disabled,
    /// is focused.
    Focused,
    /// is Selected.
    Selected,
}

/// The style function of widget.
pub type StyleFn<'a, Theme, Style> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

/// The state of the style
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StyleState {
    /// Use the active style
    Active,
    /// Use the selected style
    Selected,
    /// Use the hovered style
    Hovered,
    /// Use the focused style
    Focused,
}

/// A color in the HSV color space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hsv {
    /// HSV hue.
    pub(crate) hue: u16,
    /// HSV Saturation.
    pub(crate) saturation: f32,
    /// HSV value.
    pub(crate) value: f32,
}

impl Hsv {
    /// Creates a [`Hsv`] from its HSV components.
    #[must_use]
    pub const fn from_hsv(hue: u16, saturation: f32, value: f32) -> Self {
        Self {
            hue,
            saturation,
            value,
        }
    }
}

/// Creates a string of hexadecimal characters.
pub trait HexString {
    /// Turns self into a string of hexadecimal characters.
    fn as_hex_string(&self) -> String;
}

impl HexString for Color {
    fn as_hex_string(&self) -> String {
        format!(
            "#{:02X?}{:02X?}{:02X?}{:02X?}",
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        )
    }
}

impl From<Color> for Hsv {
    // https://de.wikipedia.org/wiki/HSV-Farbraum#Umrechnung_RGB_in_HSV/HSL
    fn from(color: Color) -> Self {
        let max = color.r.max(color.g.max(color.b));
        let min = color.r.min(color.g.min(color.b));

        let hue = if (max - min).abs() < f32::EPSILON {
            0.0
        } else if (max - color.r).abs() < f32::EPSILON {
            60.0 * (0.0 + (color.g - color.b) / (max - min))
        } else if (max - color.g).abs() < f32::EPSILON {
            60.0 * (2.0 + (color.b - color.r) / (max - min))
        } else {
            60.0 * (4.0 + (color.r - color.g) / (max - min))
        };

        let hue = if hue < 0.0 { hue + 360.0 } else { hue } as u16 % 360;

        let saturation = if max == 0.0 { 0.0 } else { (max - min) / max };

        let value = max;

        Self {
            hue,
            saturation,
            value,
        }
    }
}

impl From<Hsv> for Color {
    fn from(hsv: Hsv) -> Self {
        // https://de.wikipedia.org/wiki/HSV-Farbraum#Umrechnung_HSV_in_RGB
        let h_i = (f32::from(hsv.hue) / 60.0).floor();
        let f = (f32::from(hsv.hue) / 60.0) - h_i;

        let p = hsv.value * (1.0 - hsv.saturation);
        let q = hsv.value * (1.0 - hsv.saturation * f);
        let t = hsv.value * (1.0 - hsv.saturation * (1.0 - f));

        let h_i = h_i as u8;
        let (red, green, blue) = match h_i {
            1 => (q, hsv.value, p),
            2 => (p, hsv.value, t),
            3 => (p, q, hsv.value),
            4 => (t, p, hsv.value),
            5 => (hsv.value, p, q),
            _ => (hsv.value, t, p),
        };

        Self::from_rgb(red, green, blue)
    }
}

/// Trait containing functions for positioning of nodes.
pub trait Position {
    /// Position this node on the given position. If the node is over the
    /// specified bounds it's bouncing back to be fully visible on screen.
    /// specified bounds it's bouncing back to be fully visible on screen.
    /// The `top` specifies if the position should be considered the top left
    /// corner of the node or the bottom left corner.
    fn position_and_bounce(&mut self, position: Point, bounds: Size, top: bool);
}

impl Position for Node {
    fn position_and_bounce(&mut self, position: Point, bounds: Size, top: bool) {
        self.move_to_mut(Point::new(position.x.max(0.0), position.y.max(0.0)));

        let new_self_bounds = self.bounds();

        self.move_to_mut(Point::new(
            if new_self_bounds.x + new_self_bounds.width > bounds.width {
                (new_self_bounds.x - (new_self_bounds.width - (bounds.width - new_self_bounds.x)))
                    .max(0.0)
            } else {
                new_self_bounds.x
            },
            if top && (new_self_bounds.y + new_self_bounds.height > bounds.height) {
                (new_self_bounds.y - (new_self_bounds.height - (bounds.height - new_self_bounds.y)))
                    .max(0.0)
            } else if !top && (new_self_bounds.y - new_self_bounds.height < 0.0) {
                (new_self_bounds.y + (new_self_bounds.height - new_self_bounds.y)).max(0.0)
            } else if top {
                new_self_bounds.y
            } else {
                new_self_bounds.y - new_self_bounds.height
            },
        ));
    }
}

/// Shortcut helper to create a [`ColorPicker`] Widget.
///
/// [`ColorPicker`]: crate::ColorPicker
pub fn color_picker<'a, Message, Theme, F>(
    show_picker: bool,
    color: Color,
    underlay: impl Into<Element<'a, Message, Theme, iced::Renderer>>,
    on_cancel: Message,
    on_submit: F,
) -> ColorPicker<'a, Message, Theme>
where
    Message: 'a + Clone,
    Theme: 'a
        + Catalog
        + iced::widget::button::Catalog
        + iced::widget::text::Catalog
        + container::Catalog
        + text_input::Catalog
        + Clone,
    F: 'a + Fn(Color) -> Message,
{
    ColorPicker::new(show_picker, color, underlay, on_cancel, on_submit)
}
