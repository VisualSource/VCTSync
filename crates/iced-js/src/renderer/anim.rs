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

/// The most a single frame may advance an animation. A frame that stalls
/// (e.g. the first one to draw freshly shown content, which has to shape text
/// and rasterize svgs) would otherwise skip straight past a short animation.
const MAX_FRAME_STEP: Duration = Duration::from_millis(50);

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
    /// The animation's own clock. It follows the frames' timestamps but
    /// advances by at most [`MAX_FRAME_STEP`] per frame. `draw` has no clock
    /// of its own, so this is also what it renders at.
    now: Instant,
    /// Real timestamp of the last frame, to measure the next step from.
    last_frame: Instant,
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
        #[cfg(test)]
        tests::NEW_STATES.with(|n| n.set(n.get() + 1));
        tree::State::new(State {
            animation: Animation::new(self.active)
                .duration(self.duration)
                .easing(self.easing),
            now: Instant::now(),
            last_frame: Instant::now(),
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
            // Measure the first step from here, not from whenever the last
            // frame was drawn before the element went idle.
            state.last_frame = Instant::now();
            state.animation.go_mut(self.active, state.now);
        }
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        // `stack`, `col` and `row` drop children with a void size hint. While
        // the content is a zero-size placeholder (e.g. a hidden `<Activity>`)
        // that would throw this widget away along with its animation state.
        let hint = self.content.as_widget().size_hint();
        if hint.is_void() {
            Size::new(Length::Shrink, Length::Shrink)
        } else {
            hint
        }
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
            state.now += now
                .saturating_duration_since(state.last_frame)
                .min(MAX_FRAME_STEP);
            state.last_frame = *now;

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

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    thread_local! {
        pub static NEW_STATES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    }
    use crate::renderer::props::Offset;
    use iced::{Point, advanced::clipboard};

    type El = Element<'static, (), iced::Theme, ()>;

    fn anim(active: bool, w: f32) -> El {
        Anim::new(iced::widget::Space::new().width(w).height(50.0), active)
            .from(Keyframe {
                x: Offset::Percent(100.0),
                ..Keyframe::default()
            })
            .duration(Duration::from_millis(220))
            .hide_when_inactive(true)
            .into()
    }

    /// Diffs `next` into `tree` like a rebuild would, then runs 20 frames
    /// 16ms apart and returns the progress after each.
    fn run(tree: &mut Tree, mut next: El) -> Vec<f32> {
        tree.diff(&next);
        let node = next.as_widget_mut().layout(
            tree,
            &(),
            &layout::Limits::new(Size::ZERO, Size::new(800.0, 600.0)),
        );
        let start = Instant::now();
        let mut out = Vec::new();

        for k in 1..=20 {
            let event = Event::Window(window::Event::RedrawRequested(
                start + Duration::from_millis(16 * k),
            ));
            let mut messages = Vec::new();
            next.as_widget_mut().update(
                tree,
                &event,
                Layout::new(&node),
                mouse::Cursor::Unavailable,
                &(),
                &mut clipboard::Null,
                &mut Shell::new(&mut messages),
                &Rectangle::new(Point::ORIGIN, Size::new(800.0, 600.0)),
            );
            out.push(tree.state.downcast_ref::<State>().progress());
        }

        out
    }

    /// The shape the profiles page commits: a stack whose second layer is an
    /// `anim`, with its child going from a hidden placeholder to real content.
    fn page(active: bool, open_content: bool) -> El {
        let content: El = if open_content {
            iced::widget::container(iced::widget::column![iced::widget::Space::new()]).into()
        } else {
            // What a hidden node becomes, see `to_node`.
            iced::widget::Space::new().width(0.0).height(0.0).into()
        };
        iced::widget::container(iced::widget::stack![
            iced::widget::column![iced::widget::Space::new()],
            Anim::new(content, active),
        ])
        .into()
    }

    fn anim_state(root: &Tree) -> &State {
        // `container` shares its tree with its content, so this is the stack.
        root.children[1].state.downcast_ref::<State>()
    }

    #[test]
    fn swapping_in_content_keeps_state() {
        let mut tree = Tree::new(&page(false, false));
        let before = anim_state(&tree) as *const State;

        tree.diff(&page(true, true));

        assert_eq!(
            before,
            anim_state(&tree) as *const State,
            "state was recreated"
        );
        assert!(anim_state(&tree).animation.value());
    }

    #[test]
    fn opens_and_closes_over_time() {
        let mut tree = Tree::new(&anim(false, 0.0));
        run(&mut tree, anim(false, 0.0));

        let open = run(&mut tree, anim(true, 100.0));
        let close = run(&mut tree, anim(false, 100.0));
        println!("open:  {open:.2?}\nclose: {close:.2?}");

        assert!(open[0] < 0.5, "opening jumped: {open:?}");
        assert!(close[0] > 0.5, "closing jumped: {close:?}");
        assert_eq!(*open.last().unwrap(), 1.0);
        assert_eq!(*close.last().unwrap(), 0.0);
    }
}
