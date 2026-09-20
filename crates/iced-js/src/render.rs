use iced::Element;
use rquickjs::{Error, Object};

use crate::{
    Event,
    nodes::{ButtonProps, CommonProps, TextProps, ViewProps},
    runtime::Payload,
};

#[derive(Debug)]
pub enum Tag {
    Col(CommonProps),
    Row(CommonProps),
    View(ViewProps),
    Button(ButtonProps),
    Text(TextProps),
}

impl Tag {
    fn valid_child_count(&self, len: usize) -> bool {
        match self {
            Tag::View(_) | Tag::Button(_) | Tag::Text(_) => len == 1,
            _ => true,
        }
    }
}

#[derive(Debug)]
pub enum Node {
    Element { tag: Tag, children: Vec<Node> },
    Text(Box<str>),
}

static MAX_DEPTH: u32 = 256;

pub fn to_node(node: Object<'_>, depth: u32) -> Result<Node, Error> {
    if depth >= MAX_DEPTH {
        return Err(Error::FromJs {
            from: "react object tree",
            to: "Node",
            message: Some("Max component depth".into()),
        });
    }
    if node.contains_key("text")? {
        let text = node.get::<_, String>("text")?.into_boxed_str();
        return Ok(Node::Text(text));
    }

    let tag = node.get::<_, String>("type")?;
    let el_tag = match tag.as_str() {
        "col" => {
            let props = node.get::<_, CommonProps>("props")?;
            Tag::Col(props)
        }
        "row" => {
            let props = node.get::<_, CommonProps>("props")?;
            Tag::Row(props)
        }
        "view" => {
            let props = node.get::<_, ViewProps>("props")?;
            Tag::View(props)
        }
        "button" => {
            let props = node.get::<_, ButtonProps>("props")?;
            Tag::Button(props)
        }
        "text" => {
            let props = node.get::<_, TextProps>("props")?;
            Tag::Text(props)
        }
        _ => {
            return Err(Error::FromJs {
                from: "object",
                to: "Tag",
                message: Some("unknown tag name".to_string()),
            });
        }
    };

    let mut children = Vec::default();
    let items = node.get::<_, Vec<Object<'_>>>("children")?;
    if !el_tag.valid_child_count(items.len()) {
        return Err(Error::FromJs {
            from: "children",
            to: "children",
            message: Some("invalid child count for tag".into()),
        });
    }
    for item in items {
        let child = to_node(item, depth + 1)?;
        children.push(child);
    }

    Ok(Node::Element {
        tag: el_tag,
        children,
    })
}

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
        },
        Node::Text(txt) => iced::widget::text(&**txt).into(),
    }
}
