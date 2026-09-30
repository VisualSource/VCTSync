use super::{
    props::{ButtonProps, CommonProps, SpaceProps, TextProps, ViewProps},
    tag::Tag,
};

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

    let tag = node.get::<_, String>("type")?;
    let el_tag = match tag.as_str() {
        "col" => Tag::Col(node.get::<_, CommonProps>("props")?),
        "row" => Tag::Row(node.get::<_, CommonProps>("props")?),
        "view" => Tag::View(node.get::<_, ViewProps>("props")?),
        "button" => Tag::Button(node.get::<_, ButtonProps>("props")?),
        "text" => Tag::Text(node.get::<_, TextProps>("props")?),
        "scroll" => Tag::Scroll,
        "input" => {
            unimplemented!()
        }
        "textarea" => {
            unimplemented!()
        }
        "float" => {
            unimplemented!()
        }
        "grid" => {
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

        "theme" => {
            unimplemented!()
        }
        "hr" => Tag::Hr,
        "vr" => Tag::Vr,
        "table" => {
            unimplemented!()
        }
        "space" => Tag::Space(node.get::<_, SpaceProps>("props")?),
        "tooltip" => {
            unimplemented!()
        }

        /* feature only  elements */
        "img" => {
            unimplemented!()
        }
        "canvas" => {
            unimplemented!()
        }
        "markdown" => {
            unimplemented!()
        }
        "qr-code" => {
            unimplemented!()
        }
        #[cfg(feature = "svg-element")]
        "svg" => {
            use super::props::SvgProps;
            Tag::Svg(node.get::<_, SvgProps>("props")?)
        }
        _ => {
            return Err(rquickjs::Error::FromJs {
                from: "object",
                to: "Tag",
                message: Some("unknown tag name".to_string()),
            });
        }
    };

    let mut children = Vec::default();
    let items = node.get::<_, Vec<rquickjs::Object<'_>>>("children")?;
    if !el_tag.valid_child_count(items.len()) {
        return Err(rquickjs::Error::FromJs {
            from: "Value<'js>",
            to: "Vec<Object<'js>>",
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
