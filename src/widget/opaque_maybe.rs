use iced_core as core;

use core::layout::{self, Layout};
use core::mouse;
use core::renderer;
use core::widget::tree::{self, Tree};
use core::{Element, Event, Length, Rectangle, Shell, Size, Widget, widget::operation};

/// Wraps the given widget and captures any mouse button events inside the bounds of
/// the widget by lifting the mouse cursor as well — effectively making it _opaque_.
///
/// This helper is meant to be used to mark elements in a [`Stack`] to avoid mouse
/// events from passing through layers.
///
/// [`Stack`]: crate::Stack
pub fn opaque_maybe<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    opaque: bool,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: core::Renderer + 'a,
{
    struct Opaque<'a, Message, Theme, Renderer> {
        content: Element<'a, Message, Theme, Renderer>,
        opaque: bool,
    }

    impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
        for Opaque<'_, Message, Theme, Renderer>
    where
        Renderer: core::Renderer,
    {
        fn tag(&self) -> tree::Tag {
            self.content.as_widget().tag()
        }

        fn state(&self) -> tree::State {
            self.content.as_widget().state()
        }

        fn children(&self) -> Vec<Tree> {
            self.content.as_widget().children()
        }

        fn diff(&self, tree: &mut Tree) {
            self.content.as_widget().diff(tree);
        }

        fn size(&self) -> Size<Length> {
            self.content.as_widget().size()
        }

        fn size_hint(&self) -> Size<Length> {
            self.content.as_widget().size_hint()
        }

        fn layout(
            &mut self,
            tree: &mut Tree,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            self.content.as_widget_mut().layout(tree, renderer, limits)
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
            self.content
                .as_widget()
                .draw(tree, renderer, theme, style, layout, cursor, viewport);
        }

        fn operate(
            &mut self,
            state: &mut Tree,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn operation::Operation,
        ) {
            self.content
                .as_widget_mut()
                .operate(state, layout, renderer, operation);
        }

        fn update(
            &mut self,
            state: &mut Tree,
            event: &Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            clipboard: &mut dyn core::Clipboard,
            shell: &mut Shell<'_, Message>,
            viewport: &Rectangle,
        ) {
            let is_mouse_event = matches!(event, core::Event::Mouse(_));

            let mut cursor = cursor;
            if self.opaque {
                if is_mouse_event && cursor.is_over(layout.bounds()) {
                    shell.capture_event();
                }
                cursor = cursor.levitate();
            }

            self.content.as_widget_mut().update(
                state, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
        }

        fn mouse_interaction(
            &self,
            state: &core::widget::Tree,
            layout: core::Layout<'_>,
            cursor: core::mouse::Cursor,
            viewport: &core::Rectangle,
            renderer: &Renderer,
        ) -> core::mouse::Interaction {
            if self.opaque {
                mouse::Interaction::Idle
            } else {
                self.content
                    .as_widget()
                    .mouse_interaction(state, layout, cursor, viewport, renderer)
            }
        }

        fn overlay<'b>(
            &'b mut self,
            state: &'b mut core::widget::Tree,
            layout: core::Layout<'b>,
            renderer: &Renderer,
            viewport: &Rectangle,
            translation: core::Vector,
        ) -> Option<core::overlay::Element<'b, Message, Theme, Renderer>> {
            self.content
                .as_widget_mut()
                .overlay(state, layout, renderer, viewport, translation)
        }
    }

    Element::new(Opaque {
        content: content.into(),
        opaque,
    })
}
