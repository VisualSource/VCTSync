macro_rules! map_attrs {
    ($attrs:ident, $stream:ident $(,)?) => {};
    ($attrs:ident, $stream:ident, $key:literal => |$v:ident| { $($body:tt)* } $($rest:tt)*) => {
        $attrs.append_if_value(&mut $stream, $key, |$v| quote! { $($body)* } )?;
        map_attrs!($attrs, $stream $($rest)*);
    };
    ($attrs:ident, $stream:ident, $key:literal => || { $($body:tt)* } $($rest:tt)* ) => {
        $attrs.append_if(&mut $stream, $key, || quote! { $($body)* });
        map_attrs!($attrs, $stream $($rest)*);
    };
}

macro_rules! parse_children {
    ($node:ident) => {{
        let mut children = proc_macro2::TokenStream::new();
        let mut results = Vec::<proc_macro2::TokenStream>::new();
        for child in $node.children() {
            let item = handle_node(&child)?;

            results.push(item);
        }
        children.append_separated(results, quote! {,});

        children
    }};
}

macro_rules! required_single_child {
    ($node_element:ident) => {{
        let children = $node_element.children();
        if children.len() != 1 {
            return Err(syn::Error::new(
                $node_element.span(),
                "element only expects a single child",
            ));
        }

        let node = &children[0];
        handle_node(node)?
    }};
}

macro_rules! required_attr {
    ($attrs:ident, $node:ident, $key:literal) => {
        strip_braces(
            $attrs
                .get_value($key)
                .ok_or_else(|| {
                    syn::Error::new($node.open_tag.span(), "missing required attribute")
                })?
                .value
                .to_token_stream(),
        )?
    };
}

macro_rules! optional_attr {
    ($attrs:ident, $key:literal) => {
        $attrs.get_value($key).then(|x| strip_braces(x.value))
    };
}

pub(crate) use map_attrs;
pub(crate) use parse_children;

pub(crate) use optional_attr;
pub(crate) use required_attr;
pub(crate) use required_single_child;
