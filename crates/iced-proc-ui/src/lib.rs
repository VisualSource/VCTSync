use quote::TokenStreamExt;
use quote::quote;
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

                    let col = quote! {
                        iced::widget::Row::with_children(vec![#children])
                    };

                    return Ok(col);
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
                        iced::Element::from(iced::widget::Column::with_children(vec![#children]))
                    };

                    return Ok(col);
                }
                "svg" => {
                    unimplemented!()
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
