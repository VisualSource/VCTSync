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
