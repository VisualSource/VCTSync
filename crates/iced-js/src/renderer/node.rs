use super::tag::Tag;

#[derive(Debug)]
pub enum Node {
    Element { tag: Tag, children: Vec<Node> },
    Text(Box<str>),
}

static MAX_DEPTH: u32 = 256;

pub(crate) fn to_node(node: rquickjs::Object<'_>, depth: u32) -> rquickjs::Result<Node> {
    if depth >= MAX_DEPTH {
        return Err(rquickjs::Error::FromJs {
            from: "object",
            to: "Node",
            message: Some("max component depth".into()),
        });
    }
    // Set by the host config on a subtree that an `<Activity mode="hidden">`
    // hides. Nothing of it is drawn, so the children are not even decoded.
    if node.get::<_, Option<bool>>("hidden")?.unwrap_or_default() {
        return Ok(Node::Element {
            tag: Tag::Space {
                width: Some(iced::Length::Fixed(0.0)),
                height: Some(iced::Length::Fixed(0.0)),
            },
            children: Vec::new(),
        });
    }

    if node.contains_key("text")? {
        let text = node.get::<_, String>("text")?.into_boxed_str();
        return Ok(Node::Text(text));
    }

    let tag = node.as_value().get::<Tag>()?;

    let mut children = Vec::default();
    let items = node.get::<_, Vec<rquickjs::Object<'_>>>("children")?;
    if !tag.valid_child_count(items.len()) {
        return Err(rquickjs::Error::FromJs {
            from: "Value<'js>",
            to: "Vec<Object<'js>>",
            message: Some(format!("invalid count tag '{}'", tag)),
        });
    }
    for item in items {
        let child = to_node(item, depth + 1)?;
        children.push(child);
    }

    Ok(Node::Element { tag, children })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rquickjs::{Context, Runtime};

    fn parse(js: &str) -> Node {
        let rt = Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let obj = ctx.eval::<rquickjs::Object, _>(js).unwrap();
            to_node(obj, 0).unwrap()
        })
    }

    #[test]
    fn hidden_node_becomes_empty_space() {
        // The child is not a valid node, which also shows it is never decoded.
        let node =
            parse(r#"({ type: "view", hidden: true, props: {}, children: [{ type: "bogus" }] })"#);

        let Node::Element { tag, children } = node else {
            panic!("expected an element");
        };

        assert!(children.is_empty());
        assert!(matches!(
            tag,
            Tag::Space {
                width: Some(iced::Length::Fixed(0.0)),
                height: Some(iced::Length::Fixed(0.0)),
            }
        ));
    }

    #[test]
    fn hidden_text_becomes_empty_space() {
        let node = parse(r#"({ text: "hi", hidden: true })"#);

        assert!(matches!(
            node,
            Node::Element {
                tag: Tag::Space { .. },
                ..
            }
        ));
    }
}
