//! Iced XML
//!    
//! Write iced ui element using xml like syntax
//!
//!

use std::collections::HashMap;

use proc_macro2::Delimiter;
use proc_macro2::TokenTree;
use quote::ToTokens;
use quote::TokenStreamExt;
use quote::quote;
use rstml::node::AttributeValueExpr;
use rstml::node::KeyedAttribute;

use rstml::node::NodeAttribute;

use rstml::{node::Node, parse2};
use syn::spanned::Spanned;

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

#[proc_macro]
pub fn ui(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    match parse_xml(input) {
        Ok(out) => out.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

fn parse_xml(stream: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let nodes = parse2(stream)?;

    if nodes.len() != 1 {
        panic!("handle this error better");
    }

    let node = &nodes[0];

    let output = handle_node(node)?;

    Ok(output)
}

fn handle_node(node: &Node) -> syn::Result<proc_macro2::TokenStream> {
    match node {
        Node::Comment(node_comment) => Ok(quote! {}),
        Node::Doctype(node_doctype) => unimplemented!(),
        Node::Fragment(node_fragment) => unimplemented!(),
        Node::Element(node_element) => {
            let name = node_element.name();

            match name.to_string().as_str() {
                "row" => {
                    let children = parse_children!(node_element);
                    let mut row = quote! {
                        iced::widget::Row::with_children([#children])
                    };

                    let attrs = Attributes::new(node_element.attributes());

                    map_attrs!(
                        attrs,
                        row,
                        "padding" => |value|{ .padding(#value) },
                        "spacing" => |value|{ .spacing(#value)},
                        "height" => |value|{.height(#value)},
                        "width" => |value|{.width(#value)},
                        "alignY" => |value|{.align_y(#value)},
                        "warp" => ||{.warp()},
                        "clip" => |value|{.clip(#value)}
                    );

                    return Ok(quote! { iced::Element::from(#row) });
                }
                "col" => {
                    let children = parse_children!(node_element);
                    let attrs = Attributes::new(node_element.attributes());
                    let mut col = quote! {
                        iced::widget::Column::with_children([#children])
                    };

                    map_attrs!(
                        attrs,
                        col,
                        "padding" => |value|{ .padding(#value) },
                        "spacing" => |value|{ .spacing(#value)},
                        "height" => |value|{.height(#value)},
                        "width" => |value|{.width(#value)},
                        "alignX" => |value|{.align_x(#value)},
                        "warp" => ||{.warp()},
                        "clip" => |value|{.clip(#value)}
                    );

                    return Ok(quote! { iced::Element::from(#col) });
                }
                "svg" => {
                    if !node_element.open_tag.is_self_closed() {
                        return Err(syn::Error::new(
                            node_element.span(),
                            "svg is a self closed tag",
                        ));
                    }

                    let attrs = Attributes::new(node_element.attributes());
                    let source = required_attr!(attrs, node_element, "src");
                    let mut svg = quote! {
                        iced::widget::svg(#source)
                    };

                    map_attrs!(attrs,svg,
                        "style" => |value|{.style(#value)},
                        "width" => |value|{.width(#value)},
                        "height" => |value|{.height(#value)}
                    );

                    Ok(quote! {
                        iced::Element::from(#svg)
                    })
                }
                "scroll" => {
                    let content = required_single_child!(node_element);

                    let scroll = quote! {
                        iced::widget::scrollable(#content)
                    };

                    Ok(quote! { iced::Element::from(#scroll) })
                }
                "button" => {
                    let content = required_single_child!(node_element);
                    let attrs = Attributes::new(node_element.attributes());
                    let mut btn = quote! {
                       iced::widget::button(#content)
                    };

                    map_attrs!(attrs,btn,
                        "class" => |value|{.class(#value)},
                        "clip" => |value|{.clip(#value)},
                        "onPress" => |value|{.on_press(#value)},
                        "onPressMaybe" => |value|{.on_press_maybe(#value)},
                        "onPressWith" => |value|{.on_press_with(#value)},
                        "padding" => |value|{.padding(#value)},
                        "style" => |value|{.style(#value)},
                        "width" => |value|{.width(#value)},
                        "height" => |value|{.height(#value)},
                    );

                    return Ok(quote! { iced::Element::from(#btn) });
                }
                "rich-text" => {
                    let rich = quote! {
                        iced::widget::text::Rich::new()
                    };

                    Ok(quote! { iced::Element::from(#rich) })
                }
                "text" => {
                    let content = required_single_child!(node_element);
                    let attrs = Attributes::new(node_element.attributes());
                    let mut text = quote! {
                         iced::widget::text(#content)
                    };

                    map_attrs!(attrs,text,
                        "size"=> |value| {.size(#value)},
                        "lineHeight"=> |value| {.line_height(#value)},
                        "font" => |value| {.font(#value)},
                        "width" => |value|{.width(#value)},
                        "height" => |value|{.height(#value)},
                        "alignX" => |value|{.align_x(#value)},
                        "alignY" => |value|{.align_y(#value)},
                        "wrapping" => |value| {.wrapping(#value)},
                        "onLinkClick" => |value| {.on_link_click(#value)},
                        "style" => |value|{.style(#value)},
                        "color" => |value|{.color(#value)},
                        "colorMaybe" => |value| {.color_maybe(#value)},
                        "font" => |value|{.font(#value)},
                        "fontMaybe" => |value| {.font_maybe(#value)},
                        "class" => |value|{.class(#value)},
                        "center" =>||{.center()},
                        "shaping" => |value|{.shaping(#value)}
                    );

                    Ok(quote! {
                        iced::Element::from(#text)
                    })
                }
                "span" => {
                    let content = required_single_child!(node_element);
                    let attrs = Attributes::new(node_element.attributes());

                    let mut text = quote! {
                        iced::widget::text::Span::from(iced::widget::span(#content))
                    };

                    map_attrs!(attrs,text,
                        "color" => |value|{.color(#value)}
                    );

                    Ok(text)
                }
                "view" => {
                    let content = required_single_child!(node_element);

                    let attrs = Attributes::new(node_element.attributes());

                    let mut container = quote! {
                        iced::widget::container(#content)
                    };

                    map_attrs!(attrs,container,
                        "id" => |value|{.id(#value)},
                        "padding" => |value|{.padding(#value)},
                        "width" =>|value|{.width(#value)},
                        "height" => |value|{.height(#value)},
                        "maxWidth" => |value|{.max_width(#value)},
                        "maxHeight" => |value|{.max_height(#value)},
                        "centerX" => |value|{.center_y(#value)},
                        "centerY" => |value|{.center_x(#value)},
                        "center" => |value|{.center(#value)},
                        "alignLeft" => |value|{.center_left(#value)},
                        "alignRight" => |value|{.center_right(#value)},
                        "alignTop" => |value|{.center_top(#value)},
                        "alignBottom" => |value|{.center_bottom(#value)},
                        "alignX" => |value|{.center_x(#value)},
                        "alignY" => |value|{.center_y(#value)},
                        "clip" => |value|{.clip(#value)},
                        "style" => |value|{.style(#value)},
                        "class" => |value|{.class(#value)}
                    );

                    Ok(quote! { iced::Element::from(#container) })
                } // container
                "input" => {
                    if !node_element.open_tag.is_self_closed() {
                        return Err(syn::Error::new(
                            node_element.span(),
                            "input is a self closed tag",
                        ));
                    }
                    let attrs = Attributes::new(node_element.attributes());

                    let inputt = match attrs.get_value("type") {
                        Some(attr) => {
                            let st = attr.value_literal_string().ok_or_else(|| {
                                syn::Error::new(attr.span(), "was expecting a literal string")
                            })?;
                            st
                        }
                        None => {
                            return Err(syn::Error::new(
                                node_element.span(),
                                "a type attribute is required",
                            ));
                        }
                    };

                    let widget = match inputt.as_str() {
                        "range" => {
                            let orient = attrs
                                .get_value("orient")
                                .and_then(|v| v.value_literal_string());

                            let widget = match orient {
                                Some(dir) => {
                                    if &dir == "vertical" {
                                        quote! {iced::widget::vertical_slider}
                                    } else {
                                        quote! {iced::widget::slider}
                                    }
                                }
                                None => quote! {iced::widget::slider},
                            };
                            let on_change = required_attr!(attrs, node_element, "onChange");
                            let value = required_attr!(attrs, node_element, "value");

                            let range = match attrs.get_value("range") {
                                Some(value) => strip_braces(value.value.to_token_stream())?,
                                None => {
                                    let min = required_attr!(attrs, node_element, "min");
                                    let max = required_attr!(attrs, node_element, "max");

                                    quote! {#min..=#max}
                                }
                            };

                            let mut slider = quote! { #widget(#range,#value,#on_change) };

                            map_attrs!(attrs,slider,
                                "step" => |value|{.step(#value)},
                                "width" => |value|{.width(#value)},
                                "height" => |value|{.height(#value)},
                            );

                            quote! {iced::Element::from(#slider)}
                        }
                        "text" => {
                            let placeholder = required_attr!(attrs, node_element, "placeholder");
                            let value = required_attr!(attrs, node_element, "value");

                            let mut input = quote! {iced::widget::text_input(#placeholder, #value)};

                            map_attrs!(attrs,input,
                                "id" => |value|{.id(#value)},
                                "onChange" => |value|{.on_input(#value)},
                                "onPaste" => |value|{.on_paste(#value)},
                                "onSubmit" => |value|{.on_submit(#value)},
                                "secure" => |value|{.secure(#value)},
                                "style" => |value|{.style(#value)},
                                "width" => |value|{.width(#value)},
                                "lineHeight" => |value|{.line_height(#value)}
                            );

                            input
                        }
                        "radio" => {
                            let label = required_attr!(attrs, node_element, "label");
                            let value = required_attr!(attrs, node_element, "value");
                            let selected = required_attr!(attrs, node_element, "selected");
                            let on_change = required_attr!(attrs, node_element, "onChange");

                            quote! {iced::widget::radio(#label, #value, #selected, #on_change)}
                        }
                        "checkbox" => {
                            let checked = required_attr!(attrs, node_element, "checked");

                            let mut checkbox = quote! {iced::widget::checkbox(#checked)};

                            map_attrs!(attrs,checkbox,
                                "onChange" => |value|{.on_toggle(#value)},
                                "label" => |value|{.label(#value)},
                                "style" => |value|{.style(#value)},
                                "width" => |value|{.width(#value)},
                            );

                            checkbox
                        }
                        "switch" => {
                            let checked = required_attr!(attrs, node_element, "checked");

                            let mut toggler = quote! {iced::widget::toggler(#checked)};

                            map_attrs!(attrs,toggler,
                                "onChange" => |value|{.on_toggle(#value)},
                                "label" => |value|{.label(#value)},
                                "style" => |value|{.style(#value)},
                                "width" => |value|{.width(#value)},
                            );

                            quote! {iced::Element::from(#toggler)}
                        }
                        _ => {
                            return Err(syn::Error::new(node_element.span(), "unknown input type"));
                        }
                    };

                    Ok(quote! { iced::Element::from(#widget) })
                }
                "textarea" => Ok(quote! {}),
                "canvas" => Ok(quote! {}),
                "float" => Ok(quote! {}),
                "grid" => Ok(quote! {}),
                "img" => Ok(quote! {}),
                "col-keyed" => Ok(quote! {}),
                "markdown" => Ok(quote! {}),
                "pane-gird" => Ok(quote! {}),
                "select" => Ok(quote! {}),
                "progress-bar" => Ok(quote! {}),
                "qr-code" => Ok(quote! {}),
                "switch" => Ok(quote! {}),
                "theme" => Ok(quote! {}),
                "tooltip" => Ok(quote! {}),
                "hr" => Ok(quote! {}),
                "vr" => Ok(quote! {}),
                "table" => Ok(quote! {}),

                _ => unimplemented!(),
            }
        }
        Node::Block(node_block) => strip_braces(node_block.to_token_stream()),
        Node::Text(node_text) => Ok(quote! {
             iced::Element::from(iced::widget::text(#node_text.value))
        }),
        Node::RawText(raw_text) => {
            let source = raw_text.to_token_stream_string();

            Ok(quote! {
                 iced::Element::from(iced::widget::text(#source))
            })
        }
        Node::Custom(_) => unimplemented!(),
    }
}

struct Attributes<'a>(HashMap<String, &'a KeyedAttribute>);

impl<'a> Attributes<'a> {
    fn new(attributes: &'a [NodeAttribute]) -> Self {
        let mut attrs = HashMap::new();

        for attr in attributes {
            match attr {
                NodeAttribute::Block(_) => unimplemented!(),
                NodeAttribute::Attribute(keyed_attribute) => {
                    let key = keyed_attribute.key.to_string();
                    attrs.insert(key, keyed_attribute);
                }
            }
        }

        Self(attrs)
    }

    fn append_if_value(
        &self,
        stream: &mut proc_macro2::TokenStream,
        key: &str,
        tokens: fn(proc_macro2::TokenStream) -> proc_macro2::TokenStream,
    ) -> syn::Result<()> {
        if let Some(attr) = self.0.get(key) {
            if let Some(value) = attr.possible_value.to_value() {
                let value_stream = strip_braces(value.value.to_token_stream())?;

                stream.append_all(tokens(value_stream));
            }
        }

        Ok(())
    }

    fn has_attr(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    fn get_value(&self, key: &str) -> Option<&'a AttributeValueExpr> {
        if let Some(r) = self.0.get(key) {
            r.possible_value.to_value()
        } else {
            None
        }
    }

    fn append_if(
        &self,
        stream: &mut proc_macro2::TokenStream,
        key: &str,
        tokens: fn() -> proc_macro2::TokenStream,
    ) {
        if self.0.contains_key(key) {
            stream.append_all(tokens());
        }
    }
}

fn strip_braces(stream: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let mut iter = stream.clone().into_iter(); // ref count vec

    let (Some(TokenTree::Group(group)), None) = (iter.next(), iter.next()) else {
        return Ok(stream);
    };

    if group.delimiter() != Delimiter::Brace {
        return Ok(stream);
    }

    let inner = group.stream();
    if inner.is_empty() {
        return Err(syn::Error::new(group.span(), "expected an expression"));
    }

    let has_stmt = inner
        .clone()
        .into_iter()
        .any(|t| matches!(&t, TokenTree::Punct(p) if p.as_char() == ';'));

    if has_stmt {
        return Err(syn::Error::new(
            group.span(),
            "expected a single expression, found statements",
        ));
    }

    Ok(inner)
}
