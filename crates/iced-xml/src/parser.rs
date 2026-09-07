use crate::attribute::Attributes;
use crate::macros::{map_attrs, parse_children, required_attr, required_single_child};
use crate::util::strip_braces;
use quote::{ToTokens, TokenStreamExt, quote};
use rstml::node::Node;
use syn::spanned::Spanned;

macro_rules! self_closed {
    ($node: ident) => {
        if !$node.open_tag.is_self_closed() {
            return Err(syn::Error::new($node.span(), "element is self closed"));
        }
    };
}

pub fn parse_xml(stream: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let nodes = rstml::parse2(stream.clone())?;

    if nodes.len() != 1 {
        return Err(syn::Error::new(stream.span(), "a single node is required"));
    }

    let node = &nodes[0];

    let output = handle_node(node)?;

    Ok(output)
}

fn handle_node(node: &Node) -> syn::Result<proc_macro2::TokenStream> {
    match node {
        Node::Comment(_) => Ok(quote! {}),
        Node::Doctype(_) => unimplemented!(),
        Node::Fragment(_) => unimplemented!(),
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
                    self_closed!(node_element);

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
                    let content = {
                        let children = node_element.children();
                        if children.len() != 1 {
                            return Err(syn::Error::new(
                                node_element.span(),
                                "element only expects a single child",
                            ));
                        }

                        let node = &children[0];

                        strip_braces(node.to_token_stream())
                    }?;
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
                "hr" => {
                    if !node_element.open_tag.is_self_closed() {
                        return Err(syn::Error::new(
                            node_element.span(),
                            "element is self closed",
                        ));
                    }

                    let attrs = Attributes::new(node_element.attributes());

                    let attr = attrs
                        .get_value("height")
                        .map(|x| x.value.to_token_stream())
                        .unwrap_or_else(|| quote! {2});
                    let value = strip_braces(attr)?;

                    Ok(quote! {
                        iced::Element::from(iced::widget::rule::horizontal(#value))
                    })
                }
                "vr" => {
                    self_closed!(node_element);

                    let attrs = Attributes::new(node_element.attributes());

                    let attr = attrs
                        .get_value("width")
                        .map(|x| x.value.to_token_stream())
                        .unwrap_or_else(|| quote! {2});
                    let value = strip_braces(attr)?;

                    Ok(quote! {
                        iced::Element::from(iced::widget::rule::vertical(#value))
                    })
                }
                "table" => Ok(quote! {}),

                "space" => {
                    self_closed!(node_element);

                    let attrs = Attributes::new(node_element.attributes());
                    let mut space = quote! { iced::widget::space() };

                    map_attrs!(attrs,space,
                        "height" => |value|{.height(#value)},
                        "width" => |value|{.width(#value)}
                    );

                    Ok(quote! { iced::Element::from(#space) })
                }

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
