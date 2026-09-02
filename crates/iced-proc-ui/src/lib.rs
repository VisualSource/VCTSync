use std::collections::HashMap;

use quote::ToTokens;
use quote::TokenStreamExt;
use quote::quote;
use rstml::node::AttributeValueExpr;
use rstml::node::KeyedAttribute;
use rstml::node::KeyedAttributeValue;
use rstml::node::NodeAttribute;
use rstml::node::NodeName;
use rstml::{node::Node, parse2};
use syn::spanned::Spanned;

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
                    let mut children = proc_macro2::TokenStream::new();
                    let mut results = Vec::<proc_macro2::TokenStream>::new();
                    for child in node_element.children() {
                        let item = handle_node(&child)?;

                        results.push(item);
                    }
                    children.append_separated(results, quote! {,});

                    let mut row = quote! {
                        iced::widget::Row::with_children([#children])
                    };

                    let attrs = Attributes::new(node_element.attributes());

                    attrs.append_tokens_if_value(&mut row, "padding", |value| {
                        quote! {
                            .padding(#value)
                        }
                    });

                    attrs.append_tokens_if_value(&mut row, "spacing", |value| {
                        quote! {
                            .spacing(#value)
                        }
                    });

                    attrs.append_tokens_if_value(&mut row, "height", |value| {
                        quote! {
                            .height(#value)
                        }
                    });

                    attrs.append_tokens_if_value(&mut row, "width", |value| {
                        quote! {
                            .width(#value)
                        }
                    });

                    attrs.append_tokens_if_value(&mut row, "alignY", |value| {
                        quote! {
                            .align_y(#value)
                        }
                    });

                    attrs.append_tokens_if_exists(&mut row, "wrap", || {
                        quote! {
                            .wrap()
                        }
                    });
                    attrs.append_tokens_if_value(&mut row, "clip", |value| {
                        quote! {
                            .clip(#value)
                        }
                    });

                    return Ok(quote! { iced::Element::from(#row) });
                }
                "col" => {
                    let mut children = proc_macro2::TokenStream::new();
                    let mut results = Vec::<proc_macro2::TokenStream>::new();
                    for child in node_element.children() {
                        let item = handle_node(&child)?;

                        results.push(item);
                    }
                    children.append_separated(results, quote! {,});

                    let col = quote! {
                        iced::Element::from(iced::widget::Column::with_children([#children]))
                    };

                    return Ok(col);
                }
                "svg" => {
                    if !node_element.open_tag.is_self_closed() {
                        return Err(syn::Error::new(
                            node_element.span(),
                            "svg is a self closed tag",
                        ));
                    }

                    let attrs = Attributes::new(node_element.attributes());

                    let source = attrs
                        .get_value("src")
                        .ok_or_else(|| {
                            syn::Error::new(
                                node_element.open_tag.span(),
                                "was expecting to find src attribute",
                            )
                        })?
                        .to_token_stream();

                    let mut svg = quote! {
                        iced::widget::svg(#source)
                    };

                    attrs.append_tokens_if_value(&mut svg, "style", |value| {
                        quote! {
                            .style(#value)
                        }
                    });

                    attrs.append_tokens_if_value(&mut svg, "width", |value| {
                        quote! {
                            .width(#value)
                        }
                    });
                    attrs.append_tokens_if_value(&mut svg, "height", |value| {
                        quote! {
                            .height(#value)
                        }
                    });

                    Ok(quote! {
                        iced::Element::from(#svg)
                    })
                }
                "scroll" => {
                    unimplemented!()
                }
                "button" => {
                    let child = &node_element.children[0];

                    let content = handle_node(child)?;

                    let btn = quote! {
                       iced::Element::from(iced::widget::button(#content))
                    };

                    return Ok(btn);
                }
                "text" => {
                    let text = node_element.children().first().to_token_stream();

                    Ok(quote! {
                        iced::Element::from(iced::widget::text(#text))
                    })
                } //
                "view" => Ok(quote! {}), // container
                "input" => Ok(quote! {}),
                "textarea" => Ok(quote! {}),
                _ => unimplemented!(),
            }
        }
        Node::Block(node_block) => unimplemented!(),
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
                NodeAttribute::Block(node_block) => {}
                NodeAttribute::Attribute(keyed_attribute) => {
                    let key = keyed_attribute.key.to_string();
                    attrs.insert(key, keyed_attribute);
                }
            }
        }

        Self(attrs)
    }

    fn append_tokens_if_value(
        &self,
        stream: &mut proc_macro2::TokenStream,
        key: &str,
        tokens: fn(proc_macro2::TokenStream) -> proc_macro2::TokenStream,
    ) {
        if let Some(attr) = self.0.get(key) {
            if let Some(value) = attr.possible_value.to_value() {
                let value_stream = value.value.to_token_stream();
                stream.append_all(tokens(value_stream));
            }
        }
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

    fn append_tokens_if_exists(
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
