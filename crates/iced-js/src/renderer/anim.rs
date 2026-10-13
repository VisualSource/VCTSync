//! `<anim>`: tweens its content between two transform keyframes whenever the
//! `active` prop flips.
//!
//! The animation lives in the widget tree, so React only re-renders when
//! `active` changes and every frame in between is driven natively through
//! `RedrawRequested`.

use std::time::Duration;

use iced::{
    Animation, Element, Event, Length, Rectangle, Size, Transformation, Vector,
    advanced::{
        Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer,
        widget::{self, Tree, tree},
    },
    animation::Easing,
    time::Instant,
    window,
};

use super::props::Keyframe;

pub struct Anim<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    active: bool,
    from: Keyframe,
    to: Keyframe,
    duration: Duration,
    easing: Easing,
    hide_when_inactive: bool,
    on_settle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
}

impl<'a, Message, Theme, Renderer> Anim<'a, Message, Theme, Renderer> {
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>, active: bool) -> Self {
        Self {
            content: content.into(),
            active,
            from: Keyframe::default(),
            to: Keyframe::default(),
            duration: Duration::from_millis(200),
            easing: Easing::EaseOut,
            hide_when_inactive: false,
            on_settle: None,
        }
    }

    pub fn from(mut self, from: Keyframe) -> Self {
        self.from = from;
        self
    }

    pub fn to(mut self, to: Keyframe) -> Self {
        self.to = to;
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    pub fn hide_when_inactive(mut self, hide: bool) -> Self {
        self.hide_when_inactive = hide;
        self
    }

    /// Called with the state the animation ended on, once it comes to rest.
    pub fn on_settle(mut self, on_settle: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_settle = Some(Box::new(on_settle));
        self
    }

    /// The transform for the current point of the animation, around the
    /// center of `bounds`.
    fn transformation(&self, state: &State, bounds: Rectangle) -> Transformation {
        let t = state.progress();
        let lerp = |a: f32, b: f32| a + (b - a) * t;

        let x = lerp(
            self.from.x.resolve(bounds.width),
            self.to.x.resolve(bounds.width),
        );
        let y = lerp(
            self.from.y.resolve(bounds.height),
            self.to.y.resolve(bounds.height),
        );
        let scale = lerp(self.from.scale, self.to.scale);

        let center = bounds.center();

        Transformation::translate(center.x + x, center.y + y)
            * Transformation::scale(scale)
            * Transformation::translate(-center.x, -center.y)
    }

    fn is_hidden(&self, state: &State) -> bool {
        self.hide_when_inactive && !state.animation.value() && !state.is_animating()
    }
}

struct State {
    animation: Animation<bool>,
    /// Time of the last frame, `draw` has no clock of its own.
    now: Instant,
    /// Whether the previous frame was still animating, to catch the frame it
    /// settles on.
    was_animating: bool,
}

impl State {
    fn progress(&self) -> f32 {
        self.animation.interpolate(0.0, 1.0, self.now)
    }

    fn is_animating(&self) -> bool {
        self.animation.is_animating(self.now)
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Anim<'_, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            animation: Animation::new(self.active)
                .duration(self.duration)
                .easing(self.easing),
            now: Instant::now(),
            was_animating: false,
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));

        // A re-render with a flipped `active` prop starts the transition. The
        // runtime redraws after every rebuild, so the first frame arrives on
        // its own and `update` keeps it going from there.
        let state = tree.state.downcast_mut::<State>();
        if state.animation.value() != self.active {
            let now = Instant::now();
            state.animation.go_mut(self.active, now);
            state.now = now;
        }
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
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
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
        let state = tree.state.downcast_mut::<State>();

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            state.now = *now;

            let is_animating = state.is_animating();

            if is_animating {
                shell.request_redraw();
            } else if state.was_animating
                && let Some(on_settle) = &self.on_settle
            {
                shell.publish(on_settle(state.animation.value()));
            }

            state.was_animating = is_animating;
        }

        if self.is_hidden(state) {
            return;
        }

        let inverse = self.transformation(state, layout.bounds()).inverse();

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor * inverse,
            renderer,
            clipboard,
            shell,
            &(*viewport * inverse),
        );
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
        let state = tree.state.downcast_ref::<State>();

        if self.is_hidden(state) {
            return;
        }

        let transformation = self.transformation(state, layout.bounds());
        let inverse = transformation.inverse();

        renderer.with_layer(*viewport, |renderer| {
            renderer.with_transformation(transformation, |renderer| {
                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor * inverse,
                    &(*viewport * inverse),
                );
            });
        });
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();

        // `None` also lets a parent `stack` pass the cursor through to the
        // layers below a hidden element.
        if self.is_hidden(state) {
            return mouse::Interaction::None;
        }

        let inverse = self.transformation(state, layout.bounds()).inverse();

        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor * inverse,
            &(*viewport * inverse),
            renderer,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        offset: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_ref::<State>();

        if self.is_hidden(state) {
            return None;
        }

        // Overlays (e.g. a `select` dropdown) follow the translation but not
        // the scale.
        let translation = self.transformation(state, layout.bounds()).translation();

        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            offset + translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Anim<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(anim: Anim<'a, Message, Theme, Renderer>) -> Self {
        Element::new(anim)
    }
}
