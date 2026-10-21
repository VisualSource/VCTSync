mod anim;
mod node;
mod props;
mod styles;
mod tag;

use iced::Element;
pub(crate) use node::{Node, to_node};
pub(crate) use tag::Tag;

use crate::{Event, renderer::styles::ViewStyle, runtime::Payload};

macro_rules! apply_opt {
    ($node: ident, $prop: ident, $fn: ident) => {
        if let Some(prop) = $prop {
            $node = $node.$fn(*prop);
        }
    };
}

pub fn render_tree<'a>(tree: &'a Node) -> Element<'a, Event> {
    match tree {
        Node::Element { tag, children } => match tag {
            Tag::Row {
                height,
                width,
                clip,
                align_y,
                padding,
                spacing,
                ..
            } => {
                let mut node = if children.is_empty() {
                    iced::widget::Row::new()
                } else {
                    let items = children.iter().map(render_tree);
                    iced::widget::Row::with_children(items)
                };

                apply_opt!(node, height, height);
                apply_opt!(node, width, width);
                apply_opt!(node, align_y, align_y);
                apply_opt!(node, padding, padding);
                apply_opt!(node, clip, clip);
                apply_opt!(node, spacing, spacing);

                node.into()
            }
            Tag::Col {
                padding,
                height,
                width,
                align_x,
                clip,
                spacing,
                ..
            } => {
                let mut node = if children.is_empty() {
                    iced::widget::Column::new()
                } else {
                    let items = children.iter().map(render_tree);

                    iced::widget::Column::with_children(items)
                };

                apply_opt!(node, height, height);
                apply_opt!(node, width, width);
                apply_opt!(node, align_x, align_x);
                apply_opt!(node, padding, padding);
                apply_opt!(node, clip, clip);
                apply_opt!(node, spacing, spacing);

                node.into()
            }
            Tag::View {
                padding,
                width,
                height,
                max_width,
                max_height,
                center_x,
                center_y,
                center,
                align_left,
                align_right,
                align_top,
                align_bottom,
                align_x,
                align_y,
                clip,
                id,
                style,
            } => {
                debug_assert_eq!(children.len(), 1);

                let content = render_tree(&children[0]);
                let mut node = iced::widget::container(content);

                if let Some(style) = style {
                    match style {
                        ViewStyle::RoundedBox => {
                            node = node.style(iced::widget::container::rounded_box)
                        }
                        ViewStyle::Custom(style) => {
                            node = node.style(move |theme| style.to_style(theme));
                        }
                    }
                }

                apply_opt!(node, align_bottom, align_bottom);
                apply_opt!(node, align_left, align_left);
                apply_opt!(node, align_right, align_right);
                apply_opt!(node, align_top, align_top);
                apply_opt!(node, align_x, align_x);
                apply_opt!(node, align_y, align_y);
                apply_opt!(node, center, center);
                apply_opt!(node, center_x, center_x);
                apply_opt!(node, center_y, center_y);
                apply_opt!(node, clip, clip);
                apply_opt!(node, height, height);
                apply_opt!(node, width, width);
                apply_opt!(node, max_height, max_height);
                apply_opt!(node, max_width, max_width);
                apply_opt!(node, padding, padding);

                if let Some(id) = id {
                    node = node.id(id.clone())
                }

                node.into()
            }
            Tag::Button {
                clip,
                on_press,
                padding,
                width,
                height,
                disabled,
                style,
            } => {
                debug_assert_eq!(children.len(), 1);
                let content = render_tree(&children[0]);

                let mut btn = iced::widget::button(content);

                if let Some(id) = on_press {
                    let callback = if *disabled {
                        None
                    } else {
                        Some(Event::Callback(*id, Payload::Click))
                    };
                    btn = btn.on_press_maybe(callback);
                }

                if let Some(style) = style {
                    match style {
                        styles::ButtonStyle::Text => {
                            btn = btn.style(iced::widget::button::text);
                        }
                        styles::ButtonStyle::Custom(state) => {
                            btn = btn.style(move |theme, status| state.to_style(theme, status))
                        }
                        styles::ButtonStyle::Background => {
                            btn = btn.style(iced::widget::button::background);
                        }
                        styles::ButtonStyle::Success => {
                            btn = btn.style(iced::widget::button::success);
                        }
                        styles::ButtonStyle::Danger => {
                            btn = btn.style(iced::widget::button::danger);
                        }
                        styles::ButtonStyle::Primary => {
                            btn = btn.style(iced::widget::button::primary);
                        }
                        styles::ButtonStyle::Secondary => {
                            btn = btn.style(iced::widget::button::secondary);
                        }
                        styles::ButtonStyle::Warning => {
                            btn = btn.style(iced::widget::button::warning);
                        }
                        styles::ButtonStyle::Subtle => {
                            btn = btn.style(iced::widget::button::subtle);
                        }
                    }
                }

                apply_opt!(btn, width, width);
                apply_opt!(btn, height, height);
                apply_opt!(btn, padding, padding);
                apply_opt!(btn, clip, clip);

                btn.into()
            }
            Tag::Text {
                size,
                line_height,
                width,
                height,
                align_x,
                align_y,
                color,
                center,
                shaping,
                font,
                wrapping,
            } => {
                // React drops an empty-string child, so `<text>{""}</text>` arrives with none.
                let text = match children.first() {
                    Some(Node::Text(el)) => &**el,
                    None => "",
                    Some(Node::Element { .. }) => {
                        unreachable!("text node should not contain a element node")
                    }
                };

                let mut node = iced::widget::text(text);

                apply_opt!(node, size, size);
                apply_opt!(node, width, width);
                apply_opt!(node, height, height);
                apply_opt!(node, align_x, align_x);
                apply_opt!(node, align_y, align_y);
                apply_opt!(node, shaping, shaping);
                apply_opt!(node, line_height, line_height);
                apply_opt!(node, wrapping, wrapping);
                if let Some(color) = *color {
                    node = node.style(move |theme| iced::widget::text::Style {
                        color: Some(color.resolve(theme)),
                    });
                }

                if center.is_some_and(|x| x) {
                    node = node.center();
                }

                if let Some(font) = font {
                    node = node.font(*font);
                }

                node.into()
            }
            Tag::Space { height, width } => {
                debug_assert_eq!(children.len(), 0);
                let mut node = iced::widget::space();

                apply_opt!(node, width, width);
                apply_opt!(node, height, height);

                node.into()
            }
            Tag::Hr { height } => {
                debug_assert_eq!(children.len(), 0);

                let hr = iced::widget::rule::horizontal(*height);

                hr.into()
            }
            Tag::Vr { width } => {
                debug_assert_eq!(children.len(), 0);
                let vr = iced::widget::rule::vertical(*width);

                vr.into()
            }
            Tag::Scroll {
                width,
                height,
                horizontal,
                anchor_bottom,
                anchor_left,
                anchor_right,
                anchor_top,
                auto_scroll,
                spacing,
            } => {
                let content: iced::Element<'a, Event> = if children.is_empty() {
                    iced::widget::Column::new().into()
                } else if children.len() == 1 {
                    render_tree(&children[0])
                } else {
                    let items = children.iter().map(render_tree);
                    iced::widget::Column::with_children(items).into()
                };

                let mut widget = iced::widget::scrollable(content);

                apply_opt!(widget, width, width);
                apply_opt!(widget, height, height);
                apply_opt!(widget, spacing, spacing);
                apply_opt!(widget, auto_scroll, auto_scroll);

                if horizontal.is_some_and(|x| x) {
                    widget = widget.horizontal();
                }

                if anchor_bottom.is_some_and(|x| x) {
                    widget = widget.anchor_bottom();
                }

                if anchor_left.is_some_and(|x| x) {
                    widget = widget.anchor_left();
                }
                if anchor_right.is_some_and(|x| x) {
                    widget = widget.anchor_right();
                }
                if anchor_top.is_some_and(|x| x) {
                    widget = widget.anchor_top();
                }

                widget.into()
            }
            Tag::Tooltip {
                padding,
                gap,
                snap_within_viewport,
                position,
            } => {
                let content = render_tree(&children[0]);
                let tooltip = render_tree(&children[1]);

                let mut node = iced::widget::tooltip(content, tooltip, *position);

                apply_opt!(node, gap, gap);
                apply_opt!(node, padding, padding);
                apply_opt!(node, snap_within_viewport, snap_within_viewport);
                node.into()
            }

            Tag::Float { scale } => {
                assert_eq!(children.len(), 1);
                let content = render_tree(&children[0]);

                let mut node = iced::widget::float(content);
                apply_opt!(node, scale, scale);

                node.into()
            }

            Tag::Anim {
                active,
                from,
                to,
                duration,
                easing,
                hide_when_inactive,
                on_settle,
            } => {
                debug_assert!(children.len() <= 1);
                let content = match children.first() {
                    Some(child) => render_tree(child),
                    None => iced::widget::Space::new().into(),
                };

                let mut node =
                    anim::Anim::new(content, *active).hide_when_inactive(*hide_when_inactive);
                apply_opt!(node, from, from);
                apply_opt!(node, to, to);
                apply_opt!(node, easing, easing);

                if let Some(ms) = duration {
                    node = node.duration(std::time::Duration::from_millis(*ms));
                }

                if let Some(id) = on_settle {
                    node = node
                        .on_settle(|active| Event::Callback(*id, Payload::BoolInputChange(active)));
                }

                node.into()
            }

            Tag::Textarea {
                state_id: _,
                on_change: _on_change,
            } => {
                unimplemented!()
            }

            Tag::Switch {
                disabled,
                checked,
                on_change,
                spacing,
                label,
                size,
                text_size,
                width,
                font,
            } => {
                debug_assert_eq!(children.len(), 0);
                let mut node = iced::widget::toggler(*checked);

                if let Some(id) = on_change {
                    let callback = if *disabled {
                        None
                    } else {
                        Some(|value: bool| Event::Callback(*id, Payload::BoolInputChange(value)))
                    };

                    node = node.on_toggle_maybe(callback);
                }

                if let Some(label) = label {
                    node = node.label(label);
                }

                apply_opt!(node, spacing, spacing);
                apply_opt!(node, width, width);
                apply_opt!(node, size, size);
                apply_opt!(node, text_size, text_size);
                apply_opt!(node, font, font);

                node.into()
            }
            Tag::Checkbox {
                label,
                value,
                disabled,
                on_change,
            } => {
                debug_assert_eq!(children.len(), 0);
                let mut node = iced::widget::checkbox(*value);

                if let Some(id) = on_change {
                    let callback = if *disabled {
                        None
                    } else {
                        Some(|value: bool| Event::Callback(*id, Payload::BoolInputChange(value)))
                    };

                    node = node.on_toggle_maybe(callback);
                } else if *disabled {
                    node = node.on_toggle_maybe(Option::<fn(bool) -> Event>::None);
                }

                if let Some(label) = label {
                    node = node.label(label);
                }

                node.into()
            }

            Tag::TextInput {
                value,
                placeholder,
                on_change,
                on_submit,
                disabled,
                size,
                padding,
            } => {
                debug_assert_eq!(children.len(), 0);
                let mut node = iced::widget::text_input(placeholder, value);

                apply_opt!(node, size, size);
                apply_opt!(node, padding, padding);

                if let Some(id) = on_change {
                    let callback = if *disabled {
                        None
                    } else {
                        Some(|value: String| Event::Callback(*id, Payload::TextInputChange(value)))
                    };

                    node = node.on_input_maybe(callback);
                }

                if let Some(id) = on_submit {
                    let callback = if *disabled {
                        None
                    } else {
                        Some(Event::Callback(*id, Payload::Submit))
                    };

                    node = node.on_submit_maybe(callback);
                }

                node.into()
            }

            Tag::Select {
                width,
                value,
                placeholder,
                on_select,
                options,
            } => {
                let selected = value
                    .as_ref()
                    .and_then(|v| options.iter().find(|o| &o.value == v));

                let mut node = iced::widget::pick_list(options.as_slice(), selected, |x| {
                    Event::Callback(*on_select, Payload::TextInputChange(x.value))
                });

                if let Some(placeholder) = placeholder {
                    node = node.placeholder(placeholder);
                }
                apply_opt!(node, width, width);

                node.into()
            }

            Tag::Stack {
                height,
                width,
                clip,
            } => {
                let mut node = if children.is_empty() {
                    iced::widget::Stack::new()
                } else {
                    let items = children.iter().map(render_tree);
                    iced::widget::stack(items)
                };

                apply_opt!(node, height, height);
                apply_opt!(node, width, width);
                apply_opt!(node, clip, clip);

                node.into()
            }

            Tag::Opaque => {
                debug_assert_eq!(children.len(), 1);

                let content = render_tree(&children[0]);
                iced::widget::opaque(content)
            }

            Tag::Progress {
                value,
                max,
                width,
                height,
                style,
            } => {
                debug_assert_eq!(children.len(), 0);
                use iced::widget::progress_bar;

                let mut node = progress_bar(0.0..=max.unwrap_or(1.0), *value);
                apply_opt!(node, width, length);
                apply_opt!(node, height, girth);

                let palette: fn(&iced::Theme) -> progress_bar::Style = match style.as_deref() {
                    Some("secondary") => progress_bar::secondary,
                    Some("success") => progress_bar::success,
                    Some("warning") => progress_bar::warning,
                    Some("danger") => progress_bar::danger,
                    _ => progress_bar::primary,
                };
                node.style(palette).into()
            }

            #[cfg(feature = "img-element")]
            Tag::Img {
                src,
                width,
                height,
                opacity,
                border_radius,
            } => {
                debug_assert_eq!(children.len(), 0);

                let mut node = iced::widget::image(src.clone());
                apply_opt!(node, width, width);
                apply_opt!(node, height, height);
                apply_opt!(node, opacity, opacity);
                apply_opt!(node, border_radius, border_radius);

                node.into()
            }

            #[cfg(feature = "svg-element")]
            Tag::Svg {
                src,
                width,
                height,
                color,
                opacity,
            } => {
                debug_assert_eq!(children.len(), 0);

                let mut node = iced::widget::svg(src.clone());
                apply_opt!(node, width, width);
                apply_opt!(node, height, height);
                apply_opt!(node, opacity, opacity);

                if let Some(color) = *color {
                    node = node.style(move |theme, _status| iced::widget::svg::Style {
                        color: Some(color.resolve(theme)),
                    });
                }

                node.into()
            }
        },
        Node::Text(txt) => iced::widget::text(&**txt).into(),
    }
}

#[cfg(test)]
mod replay_tests {
    use super::*;
    use iced::advanced::widget::Tree;
    use rquickjs::{Context, Runtime};

    /// `<text>{""}</text>` reaches the host with no children (React drops empty strings);
    /// it must render as empty text instead of rejecting the whole commit.
    #[test]
    fn empty_text_renders() {
        let rt = Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();

        let node = ctx.with(|ctx| {
            let commit = ctx
                .eval::<rquickjs::Object, _>(r#"({ type: "col", props: {}, children: [{ type: "text", props: {}, children: [] }] })"#)
                .unwrap();
            node::to_node(commit, 0).unwrap()
        });

        let _ = Tree::new(&render_tree(&node));
    }

    /// Commits as the profiles page sends them (from a real session's
    /// outline log), diffed into one long-lived tree the way iced rebuilds.
    #[test]
    fn opening_profiles_panel_keeps_anim_state() {
        let rt = Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();

        let nodes: Vec<Node> = ctx.with(|ctx| {
            let commits = ctx
                .eval::<Vec<rquickjs::Object>, _>(
                    r#"
                const e = (type, props, ...children) => ({ type, props: props ?? {}, children });
                const t = () => e("text", {}, { text: "x" });
                const tip = () => e("tooltip", {}, e("button", {}, e("space")), e("view", {}, t()));
                const side = e("col", {}, tip(), tip(), tip(), e("space"), tip(), e("space"));
                const page = (body) => e("row", {}, side, e("vr"), e("space"), e("view", {}, body));
                const head = e("col", {},
                    e("row", {}, e("row", {}, e("space")), e("space"), e("button", {}, e("row", {}, e("space"), t()))),
                    e("hr"), e("space"), "LIST");
                const withList = (list) => { const h = JSON.parse(JSON.stringify(head)); h.children[3] = list; return h; };
                const loading = e("view", {}, e("col", {}, t()));
                const list = e("col", {},
                    e("row", {}, t(), e("space"), e("button", {}, e("row", {}, e("space"), t()))),
                    e("scroll", {}, e("col", {}, e("view", {}, e("row", {}, t())))));
                const panel = e("view", {}, e("col", {},
                    e("row", {}, t(), e("space"), e("button", {}, { text: "X" })),
                    e("space"), e("row"), e("hr"),
                    e("col", {}, t(), e("space")), e("col", {}, t(), e("row")),
                    e("col", {}, t(), e("space")), e("col", {}, t(), e("space")),
                    e("space"), e("hr"), e("row", {}, t(), e("space"), e("button", {}, e("row", {}, e("space"), t())))));
                const anim = (active, ...children) => e("anim", { active }, ...children);
                const hidden = { ...e("view", {}, e("col")), hidden: true };
                [
                    page(e("col", {}, e("row", {}, e("input", { type: "checkbox", checked: false }), e("space")), e("hr"), e("view", {}, e("col", {}, t())))),
                    page(e("col", {}, e("row", {}, e("input", { type: "checkbox", checked: false }), e("space")), e("hr"), e("scroll", {}, e("col", {}, t())))),
                    page(e("stack", {}, withList(loading), anim(false))),
                    page(e("stack", {}, withList(loading), anim(false, hidden))),
                    page(e("stack", {}, withList(list), anim(false, hidden))),
                    page(e("stack", {}, withList(list), anim(true, panel))),
                ]
            "#,
                )
                .unwrap();
            commits
                .into_iter()
                .map(|c| node::to_node(c, 0).unwrap())
                .collect()
        });

        let mut tree = Tree::new(&render_tree(&nodes[0]));
        for (i, node) in nodes.iter().enumerate().skip(1) {
            let before = anim::tests::NEW_STATES.with(|n| n.get());
            tree.diff(&render_tree(node));
            let created = anim::tests::NEW_STATES.with(|n| n.get()) - before;
            println!(
                "commit {i}: {}  -> anim states created: {created}",
                node.outline(3)
            );
        }

        // Created once when the page first shows it, never again.
        assert_eq!(anim::tests::NEW_STATES.with(|n| n.get()), 1);
    }
}
