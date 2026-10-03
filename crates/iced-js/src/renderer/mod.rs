mod node;
mod props;
mod tag;

use iced::Element;
pub(crate) use node::{Node, to_node};
pub(crate) use tag::Tag;

use crate::{Event, runtime::Payload};

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

                node.into()
            }
            Tag::Col {
                padding,
                height,
                width,
                align_x,
                clip,
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
            } => {
                debug_assert_eq!(children.len(), 1);

                let content = render_tree(&children[0]);
                let mut node = iced::widget::container(content);

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
            } => {
                debug_assert_eq!(children.len(), 1);
                let content = render_tree(&children[0]);

                let mut btn = iced::widget::button(content);

                if let Some(id) = on_press {
                    btn = btn.on_press(Event::Callback(*id, Payload::Click))
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
                ..
            } => {
                debug_assert_eq!(children.len(), 1);

                let text = match &children[0] {
                    Node::Text(el) => el,
                    Node::Element { .. } => {
                        unreachable!("text node should not contain a element node")
                    }
                };

                let mut node = iced::widget::text(&**text);

                apply_opt!(node, size, size);
                apply_opt!(node, width, width);
                apply_opt!(node, height, height);
                apply_opt!(node, align_x, align_x);
                apply_opt!(node, align_y, align_y);
                node = node.color_maybe(*color);

                apply_opt!(node, line_height, line_height);

                if center.is_some_and(|x| x) {
                    node = node.center();
                }

                apply_opt!(node, shaping, shaping);

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
                debug_assert_eq!(children.len(), 1);

                let content = render_tree(&children[0]);
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

            Tag::Textarea { id, on_change } => {
                unimplemented!()
            }

            Tag::Switch { value, on_change } => {
                debug_assert_eq!(children.len(), 0);
                let mut node = iced::widget::toggler(*value);

                if let Some(id) = on_change {
                    node = node
                        .on_toggle(|value| Event::Callback(*id, Payload::BoolInputChange(value)))
                }

                node.into()
            }
            Tag::Checkbox {
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

                node.into()
            }

            Tag::TextInput {
                value,
                placeholder,
                on_change,
                on_submit,
                disabled,
            } => {
                debug_assert_eq!(children.len(), 0);
                let mut node = iced::widget::text_input(placeholder, value);

                if let Some(id) = on_change {
                    let callback = if *disabled {
                        None
                    } else {
                        Some(|value: String| Event::Callback(*id, Payload::TextInputChange(value)))
                    };

                    node = node.on_input_maybe(callback);
                }

                if let Some(id) = on_submit {
                    node = node.on_submit(Event::Callback(*id, Payload::Submit));
                }

                node.into()
            }

            #[cfg(feature = "svg-element")]
            Tag::Svg { src, width, height } => {
                debug_assert_eq!(children.len(), 0);

                let mut node = iced::widget::svg(src.clone());
                apply_opt!(node, width, width);
                apply_opt!(node, height, height);

                node.into()
            }
        },
        Node::Text(txt) => iced::widget::text(&**txt).into(),
    }
}
