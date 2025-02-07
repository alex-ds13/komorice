use crate::screen::monitors::DisplayInfo;

use std::collections::HashMap;

use iced::{
    advanced::{
        graphics::core::{event, touch},
        layout::{self, Layout, Limits, Node},
        mouse,
        renderer::Quad,
        widget::tree::Tree,
        Clipboard, Shell, Widget,
    },
    border::Radius,
    Border, Element, Event,
    Length::Shrink,
    Padding, Point, Shadow,
};
use iced::{alignment, Length, Rectangle, Size};

pub struct Monitors<'a, Message> {
    monitors: &'a HashMap<usize, DisplayInfo>,
    selected: Option<usize>,
    on_selected: Option<Box<dyn Fn(usize) -> Message + 'a>>,
}

impl<'a, Message> Monitors<'a, Message> {
    ///// The default size of a [`Checkbox`].
    // const DEFAULT_SIZE: f32 = 16.0;

    ///// The default spacing of a [`Checkbox`].
    // const DEFAULT_SPACING: f32 = 8.0;

    /// The default padding of a monitor rectangle
    const DEFAULT_PADDING: f32 = 5.0;

    pub fn new(monitors: &'a HashMap<usize, DisplayInfo>) -> Self {
        Monitors {
            monitors,
            selected: None,
            on_selected: None,
        }
    }

    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    pub fn on_selected<F>(mut self, on_press: F) -> Self
    where
        F: 'a + Fn(usize) -> Message,
    {
        self.on_selected = Some(Box::new(on_press));
        self
    }

    fn get_rects(&self) -> (Vec<Rectangle<f32>>, Point) {
        let mut top_left = Point::ORIGIN;
        let rects = self
            .monitors
            .iter()
            .map(|(_, DisplayInfo { size, .. })| {
                let x = size.left as f32 / 10.0;
                let y = size.top as f32 / 10.0;
                let width = size.right as f32 / 10.0;
                let height = size.bottom as f32 / 10.0;
                top_left.x = top_left.x.min(x);
                top_left.y = top_left.y.min(y);
                Rectangle {
                    x,
                    y,
                    width,
                    height,
                }
            })
            .collect();
        (rects, top_left)
    }
}

// #[derive(Default)]
// struct State {
//     origin_point: Point,
// }

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Monitors<'_, Message>
where
    Renderer: iced::advanced::text::Renderer,
    // Renderer: iced::advanced::Renderer,
{
    // fn tag(&self) -> tree::Tag {
    //     tree::Tag::of::<State>()
    // }
    //
    // fn state(&self) -> tree::State {
    //     tree::State::new(State::default())
    // }

    fn size(&self) -> Size<Length> {
        Size {
            width: Shrink,
            height: Shrink,
        }
    }

    fn layout(&self, _tree: &mut Tree, _renderer: &Renderer, _limits: &Limits) -> Node {
        // let mut origin_point = Point::ORIGIN;
        let (rects, top_left) = self.get_rects();
        let zero_rect = Rectangle::with_size(Size::ZERO);
        // let rect = rects.iter().fold(zero_rect, |rect, r| {
        //     if r.x < origin_point.x {
        //         origin_point.x += r.width;
        //     }
        //     if r.y < origin_point.y {
        //         origin_point.y += r.height;
        //     }
        //     rect.union(r)
        // });
        // let mut origin_point = Point::ORIGIN;
        let mut rect = zero_rect;
        let children: Vec<Node> = rects
            .iter()
            .map(|r| {
                let n = layout::padded(
                    _limits,
                    Shrink,
                    Shrink,
                    Padding::new(Self::DEFAULT_PADDING),
                    |limits| {
                        // layout::sized(limits, Shrink, Shrink, |_| r.size()).translate(iced::Vector {
                        //     x: origin_point.x + r.x,
                        //     y: origin_point.y + r.y,
                        // })
                        // Node::new(r.size())
                        // let size = Size {
                        //     width: r.size().width - Self::DEFAULT_PADDING * 2.0,
                        //     height: r.size().height - Self::DEFAULT_PADDING * 2.0,
                        // };
                        layout::sized(limits, Shrink, Shrink, |limits| {
                            limits.resolve(Shrink, Shrink, r.size())
                        })
                        // Node::new(r.size()).translate(iced::Vector {
                        //     x: origin_point.x + r.x,
                        //     y: origin_point.y + r.y,
                        // })
                    },
                ); //,
                   // println!("{:#?}", &n);
                   // println!("CHECKING ORIGIN: r.y -> {}, origin.y -> {}, bounds.y -> {}", r.y, origin_point.y, n.bounds().height);
                   // if r.x + 2.0 * Self::DEFAULT_PADDING < 0.0 {
                   //     origin_point.x += n.bounds().width;
                   // }
                   // if r.y + 2.0 * Self::DEFAULT_PADDING < 0.0 {
                   //     // println!("GROWING ORIGIN: r.y -> {}, origin.y -> {}, bounds.y -> {}", r.y, origin_point.y, n.bounds().height);
                   //     origin_point.y += n.bounds().height;
                   // }

                let x_offset = if r.x == top_left.x {
                    0.0
                } else {
                    2.0 * Self::DEFAULT_PADDING
                };
                let y_offset = if r.y == top_left.y {
                    0.0
                } else {
                    2.0 * Self::DEFAULT_PADDING
                };
                let n = n.translate(iced::Vector {
                    x: r.x - top_left.x + x_offset,
                    y: r.y - top_left.y + y_offset,
                });
                rect = rect.union(&n.bounds());
                n
            })
            .collect();

        // println!("ORIGIN_POINT: {origin_point}");
        // let children = children
        //     .into_iter()
        //     .zip(rects)
        //     .map(|(node, r)| {
        //         let x_offset =
        //             if r.x + 2.0 * Self::DEFAULT_PADDING + node.bounds().width < origin_point.x {
        //                 // println!("Negative Offsetting: r.x -> {}, origin.x -> {}", r.x, origin_point.x);
        //                 -2.0 * Self::DEFAULT_PADDING
        //             } else if r.x > origin_point.x {
        //                 // println!("Positive Offsetting: r.x -> {}, origin.x -> {}", r.x, origin_point.x);
        //                 2.0 * Self::DEFAULT_PADDING
        //             } else {
        //                 0.0
        //             };
        //         let y_offset =
        //             if r.y + 2.0 * Self::DEFAULT_PADDING + node.bounds().height < origin_point.y {
        //                 // println!("Negative Offsetting: r.y -> {}, origin.y -> {}", r.y, origin_point.y);
        //                 -2.0 * Self::DEFAULT_PADDING
        //             } else if r.y > origin_point.y {
        //                 // println!("Positive Offsetting: r.y -> {}, origin.y -> {}", r.y, origin_point.y);
        //                 2.0 * Self::DEFAULT_PADDING
        //             } else {
        //                 0.0
        //             };
        //         // let n = node.translate(iced::Vector {
        //         //     x: origin_point.x + x_offset + r.x,
        //         //     y: origin_point.y + y_offset + r.y,
        //         // });
        //         rect = rect.union(&node.bounds());
        //         node
        //     })
        //     .collect();
        // println!("RECT: {rect:#?}");
        // println!("Children: {children:#?}");
        Node::with_children(rect.size(), children)
        // Node::new(rect.size())
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        // println!("DRAW -> layout: {:#?}", layout);
        let background: iced::Background = iced::color!(0x333333).into();
        let hover_background: iced::Background = iced::color!(0x888888).into();
        let selected_background: iced::Background = iced::color!(0x555555).into();
        let border_color: iced::Color = iced::color!(0x000000);
        let hover_border_color: iced::Color = iced::color!(0x333333);
        let selected_border_color: iced::Color = iced::color!(0x45ccff);
        for ((idx, DisplayInfo { device_id, .. }), child_layout) in
            self.monitors.iter().zip(layout.children())
        {
            let bounds = child_layout.children().next().unwrap().bounds();
            let is_hover = _cursor.position_over(bounds).is_some();
            let background = if matches!(self.selected, Some(i) if i == *idx) {
                selected_background
            } else if is_hover {
                hover_background
            } else {
                background
            };
            let border_color = if matches!(self.selected, Some(i) if i == *idx) {
                selected_border_color
            } else if is_hover {
                hover_border_color
            } else {
                border_color
            };
            // println!("DRAW -> child_layout: {:#?}", _child_layout);
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle {
                        x: bounds.x,
                        y: bounds.y,
                        ..bounds
                    },
                    border: Border {
                        color: border_color,
                        width: 1.0,
                        radius: Radius::default(),
                    },
                    shadow: Shadow::default(),
                },
                background,
            );
            let device_id_size = iced::Pixels(10.0);
            let device_id_position = Point {
                x: bounds.center_x(),
                y: bounds.y + device_id_size.0,
            };
            renderer.fill_text(
                iced::advanced::text::Text {
                    content: device_id.clone(),
                    bounds: bounds.size(),
                    horizontal_alignment: alignment::Horizontal::Center,
                    vertical_alignment: alignment::Vertical::Top,
                    shaping: iced::advanced::text::Shaping::default(),
                    font: Renderer::default_font(renderer),
                    size: device_id_size,
                    line_height: iced::advanced::text::LineHeight::default(),
                    wrapping: iced::advanced::text::Wrapping::default(),
                },
                device_id_position,
                iced::color!(0xffffff),
                *_viewport,
            );
            renderer.fill_text(
                iced::advanced::text::Text {
                    content: idx.to_string(),
                    bounds: bounds.size(),
                    horizontal_alignment: alignment::Horizontal::Center,
                    vertical_alignment: alignment::Vertical::Center,
                    shaping: iced::advanced::text::Shaping::default(),
                    font: Renderer::default_font(renderer),
                    size: iced::Pixels(40.0),
                    line_height: iced::advanced::text::LineHeight::default(),
                    wrapping: iced::advanced::text::Wrapping::default(),
                },
                bounds.center(),
                iced::color!(0xffffff),
                *_viewport,
            );
        }
    }

    fn on_event(
        &mut self,
        _tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                for ((idx, _monitor), child_layout) in self.monitors.iter().zip(layout.children()) {
                    let bounds = child_layout.bounds();
                    let mouse_over = cursor.is_over(bounds);

                    if mouse_over {
                        if let Some(on_pressed) = &self.on_selected {
                            shell.publish((on_pressed)(*idx));
                            return event::Status::Captured;
                        }
                    }
                }
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) && self.on_selected.is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message: 'a> From<Monitors<'a, Message>> for Element<'a, Message> {
    fn from(value: Monitors<'a, Message>) -> Self {
        Element::new(value)
    }
}
