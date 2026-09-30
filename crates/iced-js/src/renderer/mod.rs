mod node;
mod props;
mod tag;
mod utils;

use iced::{Element, widget::tooltip};
pub(crate) use node::{Node, to_node};
pub(crate) use tag::Tag;

use crate::{Event, runtime::Payload};

macro_rules! apply_prop {
    ($node: ident, $props: ident, $name: ident) => {
        if let Some(prop) = $props.$name {
            $node = $node.$name(prop);
        }
    };
}

pub fn render_tree<'a>(tree: &'a Node) -> Element<'a, Event> {
    match tree {
        Node::Element { tag, children } => match tag {
            Tag::Row(props) => {
                let mut node = if children.is_empty() {
                    iced::widget::Row::new()
                } else {
                    let items = children.iter().map(render_tree);

                    iced::widget::Row::with_children(items)
                };

                apply_prop!(node, props, height);
                apply_prop!(node, props, width);
                apply_prop!(node, props, clip);
                apply_prop!(node, props, align_y);
                apply_prop!(node, props, padding);

                node.into()
            }
            Tag::Col(props) => {
                let mut node = if children.is_empty() {
                    iced::widget::Column::new()
                } else {
                    let items = children.iter().map(render_tree);

                    iced::widget::Column::with_children(items)
                };

                apply_prop!(node, props, height);
                apply_prop!(node, props, width);
                apply_prop!(node, props, clip);
                apply_prop!(node, props, align_x);
                apply_prop!(node, props, padding);

                node.into()
            }
            Tag::View(props) => {
                debug_assert_eq!(children.len(), 1);

                let content = render_tree(&children[0]);

                let mut node = iced::widget::container(content);

                apply_prop!(node, props, align_bottom);
                apply_prop!(node, props, align_left);
                apply_prop!(node, props, align_right);
                apply_prop!(node, props, align_top);
                apply_prop!(node, props, align_x);
                apply_prop!(node, props, align_y);
                apply_prop!(node, props, center);
                apply_prop!(node, props, center_x);
                apply_prop!(node, props, center_y);
                apply_prop!(node, props, clip);
                apply_prop!(node, props, height);
                //apply_prop!(node, props, id);
                apply_prop!(node, props, max_height);
                apply_prop!(node, props, max_width);
                apply_prop!(node, props, padding);
                apply_prop!(node, props, width);

                node.into()
            }
            Tag::Button(props) => {
                debug_assert_eq!(children.len(), 1);
                let content = render_tree(&children[0]);

                let mut btn = iced::widget::button(content);

                if let Some(on_press) = props.on_press {
                    btn = btn.on_press(Event::Callback(on_press, Payload::Click))
                }

                if let Some(width) = props.width {
                    btn = btn.width(width);
                }

                btn.into()
            }
            Tag::Text(props) => {
                debug_assert_eq!(children.len(), 1);

                let text = match &children[0] {
                    Node::Text(el) => el,
                    Node::Element { .. } => {
                        unreachable!("text node should not contain a element node")
                    }
                };

                let mut node = iced::widget::text(&**text);

                apply_prop!(node, props, align_x);
                apply_prop!(node, props, align_y);
                apply_prop!(node, props, size);
                apply_prop!(node, props, width);
                apply_prop!(node, props, height);
                apply_prop!(node, props, color);
                apply_prop!(node, props, line_height);

                node.into()
            }
            Tag::Space(props) => {
                debug_assert_eq!(children.len(), 0);
                let mut node = iced::widget::space();

                apply_prop!(node, props, width);
                apply_prop!(node, props, height);

                node.into()
            }
            Tag::Hr => {
                debug_assert_eq!(children.len(), 0);

                let hr = iced::widget::rule::horizontal(1);

                hr.into()
            }
            Tag::Vr => {
                debug_assert_eq!(children.len(), 0);
                let vr = iced::widget::rule::vertical(1);

                vr.into()
            }
            Tag::Scroll => {
                debug_assert_eq!(children.len(), 1);

                let content = render_tree(&children[0]);
                let widget = iced::widget::scrollable(content);

                widget.into()
            }
            Tag::Tooltip => {
                let pos = tooltip::Position::default();

                let content = render_tree(&children[0]);
                let tooltip = render_tree(&children[1]);

                let node = iced::widget::tooltip(content, tooltip, pos);

                node.into()
            }

            #[cfg(feature = "svg-element")]
            Tag::Svg(props) => {
                debug_assert_eq!(children.len(), 0);

                let mut node = iced::widget::svg(props.src.clone());
                apply_prop!(node, props, width);
                apply_prop!(node, props, height);

                node.into()
            }
        },
        Node::Text(txt) => iced::widget::text(&**txt).into(),
    }
}
