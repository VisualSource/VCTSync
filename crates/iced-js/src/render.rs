use iced::Element;
use rquickjs::{Error, Filter, Object, Value};

use crate::{Event, runtime::Payload};

pub enum Tag {
    Col,
    Row,
    View,
    Button,
    Text,
}

impl Tag {
    fn valid_child_count(&self, len: usize) -> bool {
        match self {
            Tag::View | Tag::Button | Tag::Text => len == 1,
            _ => true,
        }
    }
}

impl TryInto<Tag> for String {
    type Error = Error;

    fn try_into(self) -> Result<Tag, Self::Error> {
        match self.as_str() {
            "row" => Ok(Tag::Row),
            "col" => Ok(Tag::Col),
            "view" => Ok(Tag::View),
            "button" => Ok(Tag::Button),
            "text" => Ok(Tag::Text),
            _ => Err(Error::Unknown),
        }
    }
}

pub enum Node {
    Element {
        tag: Tag,
        props: Props,
        children: Vec<Node>,
    },
    Text(Box<str>),
}

pub enum PropValue {
    Str(Box<str>),
    Num(f64),
    Int(i32),
    Bool(bool),
    Callback(u64),
}
pub type Props = Vec<(Box<str>, PropValue)>;

static MAX_DEPTH: u32 = 256;

pub fn to_node(node: Object<'_>, depth: u32) -> Result<Node, Error> {
    if depth >= MAX_DEPTH {
        return Err(Error::FromJs {
            from: "",
            to: "",
            message: Some("Max component depth".into()),
        });
    }
    if node.contains_key("text")? {
        let text = node.get::<_, String>("text")?.into_boxed_str();
        return Ok(Node::Text(text));
    }

    let tag: Tag = node.get::<_, String>("type")?.try_into()?;

    let mut props = Vec::default();

    let d = node.get::<_, Object<'_>>("props")?;
    for prop in d.own_props::<String, Value<'_>>(Filter::new().enum_only()) {
        let (key, value) = prop?;

        let key = key.into_boxed_str();

        let value = match value.type_of() {
            rquickjs::Type::Undefined => todo!(),
            rquickjs::Type::Null => todo!(),
            rquickjs::Type::Bool => {
                let v = value.as_bool().expect("should have been a bool");
                PropValue::Bool(v)
            }
            rquickjs::Type::Int => {
                let i = value.as_int().expect("should have been a int");
                PropValue::Int(i)
            }
            rquickjs::Type::Float => {
                let f = value.as_float().expect("should have been a float");
                PropValue::Num(f)
            }
            rquickjs::Type::String => {
                let s = value.as_string().expect("should have been a string");
                let r = s.to_string()?.into_boxed_str();

                PropValue::Str(r)
            }

            _ => {
                log::warn!("unsupported value type in prop");
                continue;
            }
        };

        props.push((key, value));
    }

    let mut children = Vec::default();
    let items = node.get::<_, Vec<Object<'_>>>("children")?;
    if !tag.valid_child_count(items.len()) {
        return Err(Error::FromJs {
            from: "",
            to: "",
            message: Some("invalid child count for tag".into()),
        });
    }
    for item in items {
        let child = to_node(item, depth + 1)?;
        children.push(child);
    }

    Ok(Node::Element {
        tag,
        props,
        children,
    })
}

pub fn render_tree<'a>(tree: &'a Node) -> Element<'a, Event> {
    match tree {
        Node::Element {
            tag,
            props,
            children,
        } => match tag {
            Tag::Row => {
                if children.is_empty() {
                    iced::widget::Row::new().into()
                } else {
                    let items = children.iter().map(render_tree);

                    iced::widget::Row::with_children(items).into()
                }
            }
            Tag::Col => {
                if children.is_empty() {
                    iced::widget::Column::new().into()
                } else {
                    let items = children.iter().map(render_tree);

                    iced::widget::Column::with_children(items).into()
                }
            }
            Tag::View => {
                debug_assert_eq!(children.len(), 1);

                let content = render_tree(&children[0]);

                iced::widget::container(content).into()
            }
            Tag::Button => {
                debug_assert_eq!(children.len(), 1);
                let content = render_tree(&children[0]);

                let mut btn = iced::widget::button(content);

                for (key, value) in props {
                    match (&**key, value) {
                        ("onPress", PropValue::Callback(id)) => {
                            btn = btn.on_press(Event::Callback(id.clone(), Payload::None));
                        }
                        _ => {}
                    }
                }

                btn.into()
            }
            Tag::Text => {
                debug_assert_eq!(children.len(), 1);

                let text = match &children[0] {
                    Node::Text(el) => el,
                    Node::Element { .. } => {
                        unreachable!("text node should not contain a element node")
                    }
                };

                iced::widget::text(&**text).into()
            }
        },
        Node::Text(txt) => iced::widget::text(&**txt).into(),
    }
}
