use iced::advanced::layout::{Limits, Node};
use iced::advanced::renderer;
use iced::advanced::widget;
use iced::advanced::{self, Clipboard, Layout, Shell};
use iced::event::Event;
use iced::mouse;
use iced::{Element, Length, Point, Rectangle, Size, Theme, Vector};

#[allow(missing_debug_implementations)]
pub struct AnchoredOverlay<'a, Message> {
    content: Element<'a, Message>,
    overlay: Element<'a, Message>,
    gap: f32,
    position_above: bool,
}

impl<'a, Message> AnchoredOverlay<'a, Message> {
    pub fn new(
        content: impl Into<Element<'a, Message>>,
        overlay: impl Into<Element<'a, Message>>,
    ) -> Self {
        Self {
            content: content.into(),
            overlay: overlay.into(),
            gap: 6.0,
            position_above: false,
        }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn above(mut self) -> Self {
        self.position_above = true;
        self
    }
}

impl<'a, Message> widget::Widget<Message, Theme, iced::Renderer> for AnchoredOverlay<'a, Message>
where
    Message: 'a,
{
    fn children(&self) -> Vec<widget::Tree> {
        vec![
            widget::Tree::new(&self.content),
            widget::Tree::new(&self.overlay),
        ]
    }

    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(&[self.content.as_widget(), self.overlay.as_widget()]);
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
        renderer: &iced::Renderer,
        limits: &Limits,
    ) -> Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
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
        tree: &'b mut widget::Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<advanced::overlay::Element<'b, Message, Theme, iced::Renderer>> {
        if tree.children.len() < 2 {
            return None;
        }
        let mut children = tree.children.iter_mut();
        let content_state = children.next().unwrap();
        let overlay_state = children.next().unwrap();
        let content_overlay = self.content.as_widget_mut().overlay(
            content_state,
            layout,
            renderer,
            viewport,
            translation,
        );
        let anchored_overlay = Some(advanced::overlay::Element::new(Box::new(Overlay {
            position: layout.position() + translation,
            content_bounds: layout.bounds(),
            overlay: &mut self.overlay,
            overlay_state,
            gap: self.gap,
            position_above: self.position_above,
        })));

        match (content_overlay, anchored_overlay) {
            (None, None) => None,
            (Some(content), None) => Some(content),
            (None, Some(overlay)) => Some(overlay),
            (Some(content), Some(overlay)) => {
                Some(advanced::overlay::Group::with_children(vec![content, overlay]).overlay())
            }
        }
    }
}

impl<'a, Message> From<AnchoredOverlay<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(overlay: AnchoredOverlay<'a, Message>) -> Self {
        Element::new(overlay)
    }
}

struct Overlay<'a, 'b, Message> {
    position: Point,
    content_bounds: Rectangle,
    overlay: &'b mut Element<'a, Message>,
    overlay_state: &'b mut widget::Tree,
    gap: f32,
    position_above: bool,
}

impl<'a, 'b, Message> advanced::overlay::Overlay<Message, Theme, iced::Renderer>
    for Overlay<'a, 'b, Message>
where
    Message: 'a,
{
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> Node {
        let viewport = Rectangle::with_size(bounds);
        let overlay_layout = self.overlay.as_widget_mut().layout(
            self.overlay_state,
            renderer,
            &Limits::new(Size::ZERO, Size::INFINITE),
        );
        let overlay_bounds = overlay_layout.bounds();
        let mut target_bounds = if self.position_above {
            Rectangle {
                x: self.position.x,
                y: self.position.y - overlay_bounds.height - self.gap,
                width: overlay_bounds.width,
                height: overlay_bounds.height,
            }
        } else {
            Rectangle {
                x: self.position.x,
                y: self.position.y + self.content_bounds.height + self.gap,
                width: overlay_bounds.width,
                height: overlay_bounds.height,
            }
        };

        if target_bounds.x + target_bounds.width > viewport.width {
            target_bounds.x = viewport.width - target_bounds.width;
        }
        if target_bounds.x < viewport.x {
            target_bounds.x = viewport.x;
        }
        if target_bounds.y < viewport.y {
            target_bounds.y = viewport.y;
        }

        Node::with_children(target_bounds.size(), vec![overlay_layout])
            .translate(Vector::new(target_bounds.x, target_bounds.y))
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let Some(child_layout) = layout.children().next() else {
            return;
        };
        self.overlay.as_widget().draw(
            self.overlay_state,
            renderer,
            theme,
            style,
            child_layout,
            cursor,
            &Rectangle::with_size(Size::INFINITE),
        );
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let Some(child_layout) = layout.children().next() else {
            return;
        };
        self.overlay.as_widget_mut().update(
            self.overlay_state,
            event,
            child_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &Rectangle::with_size(Size::INFINITE),
        );
    }
}
