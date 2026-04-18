use iced::advanced::layout::{Limits, Node};
use iced::advanced::renderer;
use iced::advanced::widget::tree;
use iced::advanced::{Clipboard, Layout, Shell};
use iced::event::Event;
use iced::mouse;
use iced::{Element, Length, Rectangle, Size, Theme, Vector};

pub struct SeekArea<'a, Message> {
    content: Element<'a, Message>,
    on_press: Option<Box<dyn Fn(u16) -> Message + 'a>>,
    interaction: Option<mouse::Interaction>,
}

impl<'a, Message> SeekArea<'a, Message> {
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            on_press: None,
            interaction: None,
        }
    }

    pub fn on_press(mut self, handler: impl Fn(u16) -> Message + 'a) -> Self {
        self.on_press = Some(Box::new(handler));
        self
    }

    pub fn interaction(mut self, interaction: mouse::Interaction) -> Self {
        self.interaction = Some(interaction);
        self
    }
}

impl<Message> iced::advanced::Widget<Message, Theme, iced::Renderer> for SeekArea<'_, Message>
where
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<tree::Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut tree::Tree) {
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
        tree: &mut tree::Tree,
        renderer: &iced::Renderer,
        limits: &Limits,
    ) -> Node {
        self.content.as_widget_mut().layout(tree, renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut tree::Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut tree::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );

        if shell.is_event_captured() || self.on_press.is_none() {
            return;
        }

        let bounds = layout.bounds();
        if !cursor.is_over(bounds) {
            return;
        }

        let is_trigger = matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        );

        if is_trigger {
            if let Some(position) = cursor.position_in(bounds) {
                let ratio = if bounds.width <= 0.0 {
                    0
                } else {
                    let clamped = (position.x / bounds.width).clamp(0.0, 1.0);
                    (clamped * 1000.0).round() as u16
                };
                if let Some(handler) = self.on_press.as_ref() {
                    shell.publish(handler(ratio));
                    shell.capture_event();
                }
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &tree::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let interaction = self.content.as_widget().mouse_interaction(
            tree, layout, cursor, viewport, renderer,
        );

        match (self.interaction, interaction) {
            (Some(interaction), mouse::Interaction::None)
                if cursor.is_over(layout.bounds()) =>
            {
                interaction
            }
            _ => interaction,
        }
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            tree, renderer, theme, style, layout, cursor, viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut tree::Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, iced::Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
    }
}

impl<'a, Message> From<SeekArea<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(area: SeekArea<'a, Message>) -> Self {
        Element::new(area)
    }
}

pub fn seek_area<'a, Message>(
    content: impl Into<Element<'a, Message>>,
) -> SeekArea<'a, Message> {
    SeekArea::new(content)
}
