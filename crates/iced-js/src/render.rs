use std::fmt::Display;

use iced::{Element, widget::tooltip};
use rquickjs::{Error, Object};

use crate::{
    Event,
    nodes::{self, ButtonProps, CommonProps, SpaceProps, TextProps, ViewProps},
    runtime::Payload,
};

#[derive(Debug)]
pub enum Tag {
    Col(CommonProps),
    Row(CommonProps),
    View(ViewProps),
    Button(ButtonProps),
    Text(TextProps),
    Scroll,
    Space(SpaceProps),
    Hr,
    Vr,
    Tooltip,

    #[cfg(feature = "svg-element")]
    Svg(nodes::SvgProps),
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tag::Col(_) => write!(f, "col"),
            Tag::Row(_) => write!(f, "row"),
            Tag::View(_) => write!(f, "view"),
            Tag::Button(_) => write!(f, "button"),
            Tag::Text(_) => write!(f, "text"),

            Self::Scroll => write!(f, "scroll"),
            Self::Space(_) => write!(f, "space"),
            Self::Hr => write!(f, "hr"),
            Self::Vr => write!(f, "vr"),
            Self::Tooltip => write!(f, "tooltip"),

            #[cfg(feature = "svg-element")]
            Tag::Svg(_) => write!(f, "svg"),
        }
    }
}

impl Tag {
    fn valid_child_count(&self, len: usize) -> bool {
        match self {
            Tag::Space(_) | Tag::Hr | Tag::Vr => len == 0,
            Tag::View(_) | Tag::Button(_) | Tag::Text(_) | Tag::Scroll => len == 1,
            Tag::Tooltip => len == 2,
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
        "col" => Tag::Col(node.get::<_, CommonProps>("props")?),
        "row" => Tag::Row(node.get::<_, CommonProps>("props")?),
        "view" => Tag::View(node.get::<_, ViewProps>("props")?),
        "button" => Tag::Button(node.get::<_, ButtonProps>("props")?),
        "text" => Tag::Text(node.get::<_, TextProps>("props")?),
        #[cfg(feature = "svg-element")]
        "svg" => {
            use crate::nodes::SvgProps;
            Tag::Svg(node.get::<_, SvgProps>("props")?)
        }
        "scroll" => {
            unimplemented!()
        }
        "input" => {
            unimplemented!()
        }
        "textarea" => {
            unimplemented!()
        }
        "canvas" => {
            unimplemented!()
        }
        "float" => {
            unimplemented!()
        }
        "grid" => {
            unimplemented!()
        }
        "img" => {
            unimplemented!()
        }
        "markdown" => {
            unimplemented!()
        }
        "plane-grid" => {
            unimplemented!()
        }
        "select" => {
            unimplemented!()
        }
        "progress-bar" => {
            unimplemented!()
        }
        "qr-code" => {
            unimplemented!()
        }
        "theme" => {
            unimplemented!()
        }
        "hr" => {
            unimplemented!()
        }
        "vr" => {
            unimplemented!()
        }
        "table" => {
            unimplemented!()
        }
        "space" => {
            unimplemented!()
        }
        "tooltip" => {
            unimplemented!()
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
            message: Some(format!("invalid count tag '{}'", el_tag)),
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
                let mut vr = iced::widget::rule::vertical(1);

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
