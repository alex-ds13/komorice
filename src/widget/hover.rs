use iced_core as core;

use core::layout::{self, Layout};
use core::mouse;
use core::renderer;
use core::widget::operation::{self, Operation};
use core::widget::tree::{self, Tree};
use core::{Event, Length, Rectangle, Shell, Size, Widget, Element, window};

pub struct Hover<'a, Message, Theme, Renderer> {
    base: Element<'a, Message, Theme, Renderer>,
    top: Element<'a, Message, Theme, Renderer>,
    is_top_focused: bool,
    is_top_overlay_active: bool,
    is_hovered: bool,
    clip: bool,
}

impl<'a, Message, Theme, Renderer> Hover<'a, Message, Theme, Renderer> {
    pub fn new(
        base: impl Into<Element<'a, Message, Theme, Renderer>>,
        top: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Hover<'a, Message, Theme, Renderer> {
        Hover {
            base: base.into(),
            top: top.into(),
            is_top_focused: false,
            is_top_overlay_active: false,
            is_hovered: false,
            clip: false,
        }
    }

    pub fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Hover<'_, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        struct Tag;
        tree::Tag::of::<Tag>()
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.base), Tree::new(&self.top)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.base, &self.top]);
    }

    fn size(&self) -> Size<Length> {
        self.base.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.base.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let base = self
            .base
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);

        let top = self.top.as_widget_mut().layout(
            &mut tree.children[1],
            renderer,
            &layout::Limits::new(Size::ZERO, base.size()),
        );

        layout::Node::with_children(base.size(), vec![base, top])
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
        if let Some(clipped_viewport) = layout.bounds().intersection(viewport) {
            let bounds = if self.clip {
                clipped_viewport
            } else {
                *viewport
            };
            let mut children = layout.children().zip(&tree.children);

            let (base_layout, base_tree) = children.next().unwrap();

            self.base.as_widget().draw(
                base_tree,
                renderer,
                theme,
                style,
                base_layout,
                cursor,
                viewport,
            );

            if cursor.is_over(layout.bounds())
                || self.is_top_focused
                || self.is_top_overlay_active
            {
                let (top_layout, top_tree) = children.next().unwrap();

                renderer.with_layer(bounds, |renderer| {
                    self.top.as_widget().draw(
                        top_tree, renderer, theme, style, top_layout, cursor, viewport,
                    );
                });
            }
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let children = [&mut self.base, &mut self.top]
            .into_iter()
            .zip(layout.children().zip(&mut tree.children));

        for (child, (layout, tree)) in children {
            child
                .as_widget_mut()
                .operate(tree, layout, renderer, operation);
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn core::Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let mut children = layout.children().zip(&mut tree.children);
        let (base_layout, base_tree) = children.next().unwrap();
        let (top_layout, top_tree) = children.next().unwrap();

        let is_hovered = cursor.is_over(layout.bounds());

        if matches!(event, Event::Window(window::Event::RedrawRequested(_))) {
            let mut count_focused = operation::focusable::count();

            self.top.as_widget_mut().operate(
                top_tree,
                top_layout,
                renderer,
                &mut operation::black_box(&mut count_focused),
            );

            self.is_top_focused = match count_focused.finish() {
                operation::Outcome::Some(count) => count.focused.is_some(),
                _ => false,
            };

            self.is_hovered = is_hovered;
        } else if is_hovered != self.is_hovered {
            shell.request_redraw();
        }

        let is_visible = is_hovered || self.is_top_focused || self.is_top_overlay_active;

        if matches!(
            event,
            Event::Mouse(mouse::Event::CursorMoved { .. } | mouse::Event::ButtonReleased(_))
        ) || is_visible
        {
            let redraw_request = shell.redraw_request();

            self.top.as_widget_mut().update(
                top_tree, event, top_layout, cursor, renderer, clipboard, shell, viewport,
            );

            // Ignore redraw requests of invisible content
            if !is_visible {
                Shell::replace_redraw_request(shell, redraw_request);
            }
        };

        if shell.is_event_captured() {
            return;
        }

        self.base.as_widget_mut().update(
            base_tree,
            event,
            base_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        [&self.base, &self.top]
            .into_iter()
            .rev()
            .zip(layout.children().rev().zip(tree.children.iter().rev()))
            .map(|(child, (layout, tree))| {
                child
                    .as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .find(|&interaction| interaction != mouse::Interaction::None)
            .unwrap_or_default()
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut core::widget::Tree,
        layout: core::Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: core::Vector,
    ) -> Option<core::overlay::Element<'b, Message, Theme, Renderer>> {
        let mut overlays = [&mut self.base, &mut self.top]
            .into_iter()
            .zip(layout.children().zip(tree.children.iter_mut()))
            .map(|(child, (layout, tree))| {
                child
                    .as_widget_mut()
                    .overlay(tree, layout, renderer, viewport, translation)
            });

        if let Some(base_overlay) = overlays.next()? {
            return Some(base_overlay);
        }

        let top_overlay = overlays.next()?;
        self.is_top_overlay_active = top_overlay.is_some();

        top_overlay
    }
}

impl<'a, Message, Theme, Renderer> From<Hover<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: core::Renderer + 'a,
{
    fn from(value: Hover<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}

pub fn hover<'a, Message, Theme, Renderer>(
    base: impl Into<Element<'a, Message, Theme, Renderer>>,
    top: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Hover<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced_core::Renderer + 'a,
{
    Hover::new(base, top)
}
