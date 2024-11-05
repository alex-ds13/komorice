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
    monitors: Vec<&'a komorebi_client::Monitor>,
    selected: Option<usize>,
    on_selected: Option<Box<dyn Fn(usize) -> Message + 'a>>,
}

impl<'a, Message> Monitors<'a, Message> {
    /// The default size of a [`Checkbox`].
    // const DEFAULT_SIZE: f32 = 16.0;

    /// The default spacing of a [`Checkbox`].
    // const DEFAULT_SPACING: f32 = 8.0;

    /// The default padding of a monitor rectangle
    const DEFAULT_PADDING: f32 = 5.0;

    pub fn new(monitors: Vec<&'a komorebi_client::Monitor>) -> Self {
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

    fn get_rects(&self) -> Vec<Rectangle<f32>> {
        self.monitors
            .iter()
            .map(|monitor| {
                let size = monitor.size();
                let x = size.left as f32 / 10.0;
                let y = size.top as f32 / 10.0;
                let width = size.right as f32 / 10.0;
                let height = size.bottom as f32 / 10.0;
                Rectangle {
                    x,
                    y,
                    width,
                    height,
                }
            })
            .collect()
    }
}

// #[derive(Default)]
// struct State {
//     origin_point: Point,
// }

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Monitors<'a, Message>
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
        let rects = self.get_rects();
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
        let mut origin_point = Point::ORIGIN;
        let mut rect = zero_rect;
        let mut children: Vec<Node> = rects
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
                        layout::sized(limits, Shrink, Shrink, |limits| {
                            limits.resolve(Shrink, Shrink, r.size())
                        })
                        // Node::new(r.size()).translate(iced::Vector {
                        //     x: origin_point.x + r.x,
                        //     y: origin_point.y + r.y,
                        // })
                    },
                ); //,
                println!("{:#?}", &n);
                if r.x + 2.0 * Self::DEFAULT_PADDING < origin_point.x {
                    origin_point.x += n.bounds().width;
                }
                if r.y + 2.0 * Self::DEFAULT_PADDING < origin_point.y {
                    println!("GROWING ORIGIN: r.y -> {}, origin.y -> {}", r.y, origin_point.y);
                    origin_point.y += n.bounds().height;
                }
                n
            })
            .collect();

        println!("ORIGIN_POINT: {origin_point}");
        let children = children
            .iter_mut()
            .zip(rects)
            .map(|(node, r)| {
                let x_offset = if r.x + 2.0 * Self::DEFAULT_PADDING + node.bounds().width < origin_point.x {
                    println!("Negative Offsetting: r.x -> {}, origin.x -> {}", r.x, origin_point.x);
                    - 2.0 * Self::DEFAULT_PADDING
                } else if r.x > origin_point.x {
                    println!("Positive Offsetting: r.x -> {}, origin.x -> {}", r.x, origin_point.x);
                    2.0 * Self::DEFAULT_PADDING
                } else {
                    0.0
                };
                let y_offset = if r.y + 2.0 * Self::DEFAULT_PADDING + node.bounds().height < origin_point.y {
                    println!("Negative Offsetting: r.y -> {}, origin.y -> {}", r.y, origin_point.y);
                    - 2.0 * Self::DEFAULT_PADDING
                } else if r.y > origin_point.y {
                    println!("Positive Offsetting: r.y -> {}, origin.y -> {}", r.y, origin_point.y);
                    2.0 * Self::DEFAULT_PADDING
                } else {
                    0.0
                };
                let n = node.clone().translate(iced::Vector {
                    x: origin_point.x + x_offset + r.x,
                    y: origin_point.y + y_offset + r.y,
                });
                rect = rect.union(&n.bounds());
                n
            })
            .collect();
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
        println!("DRAW -> layout: {:#?}", layout);
        let background: iced::Background = iced::color!(0x333333).into();
        let selected_background: iced::Background = iced::color!(0x555555).into();
        for (((idx, monitor), rect), child_layout) in self
            .monitors
            .iter()
            .enumerate()
            .zip(self.get_rects())
            .zip(layout.children())
        {
            let bounds = child_layout.children().next().unwrap().bounds();
            let background = if matches!(self.selected, Some(i) if i == idx) {
                selected_background
            } else {
                background
            };
            // println!("DRAW -> child_layout: {:#?}", _child_layout);
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle {
                        x: bounds.x,
                        y: bounds.y,
                        ..rect
                    },
                    border: Border {
                        color: iced::color!(0x000000),
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
                    content: monitor.device_id().to_string(),
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
                for ((idx, _monitor), child_layout) in
                    self.monitors.iter().enumerate().zip(layout.children())
                {
                    let bounds = child_layout.bounds();
                    let mouse_over = cursor.is_over(bounds);

                    if mouse_over {
                        if let Some(on_pressed) = &self.on_selected {
                            shell.publish((on_pressed)(idx));
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

