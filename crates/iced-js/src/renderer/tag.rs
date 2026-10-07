use crate::renderer::props::IcedId;

use super::props::{IcedHorizontal, IcedLength, IcedPadding, IcedTooltipPosition, IcedVertical};
use iced::{
    Color, Length, Padding, Pixels,
    alignment::{Horizontal, Vertical},
    widget::{
        self,
        text::{LineHeight, Shaping},
    },
};
use rquickjs::FromJs;
use std::fmt::Display;

type CallbackId = u64;

#[derive(Debug)]
pub enum ViewStyle {
    RoundedBox,
    Custom {},
}

#[derive(Debug)]
pub enum Tag {
    Col {
        padding: Option<Padding>,
        height: Option<Length>,
        width: Option<Length>,
        align_x: Option<Horizontal>,
        align_y: Option<Vertical>,
        clip: Option<bool>,
        warp: Option<bool>,
        spacing: Option<Pixels>,
    },
    Row {
        padding: Option<Padding>,
        height: Option<Length>,
        width: Option<Length>,
        align_x: Option<Horizontal>,
        align_y: Option<Vertical>,
        clip: Option<bool>,
        spacing: Option<Pixels>,
        warp: Option<bool>,
    },
    View {
        id: Option<widget::Id>,
        padding: Option<Padding>,
        width: Option<Length>,
        height: Option<Length>,
        max_width: Option<Pixels>,
        max_height: Option<Pixels>,
        center_x: Option<Length>,
        center_y: Option<Length>,
        center: Option<Length>,
        align_left: Option<Length>,
        align_right: Option<Length>,
        align_top: Option<Length>,
        align_bottom: Option<Length>,
        align_x: Option<Horizontal>,
        align_y: Option<Vertical>,
        clip: Option<bool>,
        style: Option<ViewStyle>,
    },
    Button {
        // class
        clip: Option<bool>,
        on_press: Option<CallbackId>,
        //on_press_maybe
        //on_press_with
        padding: Option<Padding>,
        //style
        width: Option<Length>,
        height: Option<Length>,

        disabled: bool,
    },
    Text {
        size: Option<Pixels>,
        line_height: Option<LineHeight>,
        // font object,
        width: Option<Length>,
        height: Option<Length>,
        align_x: Option<Horizontal>,
        align_y: Option<Vertical>,
        wrapping: Option<bool>,
        // style callback
        color: Option<Color>,
        // colorMaybe
        // font
        // fontMaybe
        center: Option<bool>,
        shaping: Option<Shaping>,
    },
    Scroll {
        width: Option<Length>,
        height: Option<Length>,

        horizontal: Option<bool>,

        anchor_bottom: Option<bool>,
        anchor_left: Option<bool>,
        anchor_right: Option<bool>,
        anchor_top: Option<bool>,

        auto_scroll: Option<bool>,

        spacing: Option<Pixels>, //anchor_y
                                 //anchor_x
                                 // direction
                                 //id
                                 //on_scroll
                                 //style
    },
    Space {
        width: Option<Length>,
        height: Option<Length>,
    },
    Hr {
        //style
        height: Pixels,
    },
    Vr {
        //style
        width: Pixels,
    },
    Tooltip {
        // delay,
        padding: Option<Pixels>,
        gap: Option<Pixels>,
        snap_within_viewport: Option<bool>,
        position: iced::widget::tooltip::Position, //style
    },

    Float {
        scale: Option<f32>,
    },

    #[cfg(feature = "svg-element")]
    Svg {
        src: iced::widget::svg::Handle,
        width: Option<Length>,
        height: Option<Length>,
    },

    TextInput {
        placeholder: String,
        value: String,
        on_change: Option<CallbackId>,
        on_submit: Option<CallbackId>,
        disabled: bool,
    },
    Checkbox {
        value: bool,
        on_change: Option<CallbackId>,
        disabled: bool,
    },
    Switch {
        value: bool,
        on_change: Option<CallbackId>,
    },
    Textarea {
        state_id: String,
        on_change: Option<CallbackId>,
    },
}

macro_rules! map_props {
    ($el: literal, $props: ident, ignore [$($ignored: literal),* $(,)?], $( ($name: ident, $propKey: literal, $type: ident ,$to: ident) ), *) => {
        $(
            let mut $name: Option<$type> = None;
        )*

        for prop in $props.props::<String, rquickjs::Value<'js>>() {
            let (key, value) = prop?;
            match key.as_str() {
                $(
                    $propKey => $name = Some(value.get::<$to>()?.into()),
                )*
                // props with special handling outside of the macro
                $(
                    $ignored => {}
                )*
                _ => {
                    log::warn!("skipping unknown prop: '{key}' on '{}'",$el);
                }
            }
        }

    };
    ($el: literal, $props: ident, $( ($name: ident, $propKey: literal, $type: ident ,$to: ident) ), *) => {
        map_props!($el, $props, ignore [], $( ($name, $propKey, $type, $to) ),*)
    };
}

impl<'js> FromJs<'js> for Tag {
    fn from_js(
        _ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        if !value.is_object() {
            return Err(rquickjs::Error::new_from_js_message(
                value.type_name(),
                "Tag",
                "was expecting an object",
            ));
        }

        let obj = unsafe { value.ref_object() };

        let tag_type = obj.get::<_, String>("type")?;
        let props = obj.get::<_, rquickjs::Object<'js>>("props")?;

        match tag_type.as_str() {
            "col" | "row" => {
                map_props!(
                    "row",
                    props,
                    (padding, "padding", Padding, IcedPadding),
                    (width, "width", Length, IcedLength),
                    (height, "height", Length, IcedLength),
                    (align_x, "alignX", Horizontal, IcedHorizontal),
                    (align_y, "alignY", Vertical, IcedVertical),
                    (clip, "clip", bool, bool),
                    (warp, "warp", bool, bool),
                    (spacing, "spacing", Pixels, f32)
                );
                Ok(if tag_type == "row" {
                    Tag::Row {
                        padding,
                        height,
                        width,
                        align_x,
                        align_y,
                        clip,
                        warp,
                        spacing,
                    }
                } else {
                    Tag::Col {
                        padding,
                        height,
                        width,
                        align_x,
                        align_y,
                        clip,
                        warp,
                        spacing,
                    }
                })
            }
            "view" => {
                use iced::widget::Id;

                map_props!(
                    "view",
                    props,
                    ignore["style"],
                    (padding, "padding", Padding, IcedPadding),
                    (width, "width", Length, IcedLength),
                    (height, "height", Length, IcedLength),
                    (max_width, "maxWidth", Pixels, f32),
                    (max_height, "maxHeight", Pixels, f32),
                    (center_x, "centerX", Length, IcedLength),
                    (center_y, "centerY", Length, IcedLength),
                    (center, "center", Length, IcedLength),
                    (align_left, "alignLeft", Length, IcedLength),
                    (align_right, "alignRight", Length, IcedLength),
                    (align_top, "alignTop", Length, IcedLength),
                    (align_bottom, "alignBottom", Length, IcedLength),
                    (align_x, "alignX", Horizontal, IcedHorizontal),
                    (align_y, "alignY", Vertical, IcedVertical),
                    (clip, "clip", bool, bool),
                    (id, "id", Id, IcedId)
                );

                let style = props.get::<_, String>("style").ok().and_then(|v| {
                    if v == "roundedBox" {
                        return Some(ViewStyle::RoundedBox);
                    }

                    None
                });

                Ok(Tag::View {
                    style,
                    id,
                    padding,
                    width,
                    height,
                    max_width,
                    max_height,
                    center_x,
                    center_y,
                    center,
                    align_left,
                    align_right,
                    align_top,
                    align_bottom,
                    align_x,
                    align_y,
                    clip,
                })
            }
            "button" => {
                map_props!(
                    "button",
                    props,
                    (clip, "clip", bool, bool),
                    (on_press, "onPress", CallbackId, CallbackId),
                    (padding, "padding", Padding, IcedPadding),
                    (width, "width", Length, IcedLength),
                    (height, "height", Length, IcedLength),
                    (disabled, "disabled", bool, bool)
                );

                Ok(Tag::Button {
                    clip,
                    on_press,
                    padding,
                    width,
                    height,
                    disabled: disabled.unwrap_or_default(),
                })
            }
            "text" => {
                map_props!(
                    "text",
                    props,
                    (size, "size", Pixels, f32),
                    (width, "width", Length, IcedLength),
                    (height, "height", Length, IcedLength),
                    (align_x, "alignX", Horizontal, IcedHorizontal),
                    (align_y, "alignY", Vertical, IcedVertical),
                    (wrapping, "wrapping", bool, bool),
                    // color
                    // colorMaybe
                    // font
                    // fontMaybe
                    (center, "center", bool, bool) // shapping
                );

                Ok(Tag::Text {
                    size,
                    line_height: None,
                    width,
                    height,
                    align_x,
                    align_y,
                    wrapping,
                    color: None,
                    center,
                    shaping: None,
                })
            }
            "space" => {
                map_props!(
                    "space",
                    props,
                    (width, "width", Length, IcedLength),
                    (height, "height", Length, IcedLength)
                );

                Ok(Tag::Space { width, height })
            }
            "hr" => {
                map_props!("hr", props, (height, "height", Pixels, f32));

                Ok(Tag::Hr {
                    height: height.unwrap_or_else(|| Pixels::from(1)),
                })
            }
            "vr" => {
                map_props!("vr", props, (width, "width", Pixels, f32));
                Ok(Tag::Vr {
                    width: width.unwrap_or_else(|| Pixels::from(1)),
                })
            }
            "scroll" => {
                map_props!(
                    "scroll",
                    props,
                    (height, "height", Length, IcedLength),
                    (width, "width", Length, IcedLength),
                    (horizontal, "horizontal", bool, bool),
                    (anchor_bottom, "anchor_bottom", bool, bool),
                    (anchor_left, "anchor_left", bool, bool),
                    (anchor_right, "anchor_right", bool, bool),
                    (anchor_top, "anchor_top", bool, bool),
                    (auto_scroll, "auto_scroll", bool, bool),
                    (spacing, "spacing", Pixels, f32)
                );

                Ok(Tag::Scroll {
                    width,
                    height,
                    horizontal,
                    anchor_bottom,
                    anchor_left,
                    anchor_right,
                    anchor_top,
                    auto_scroll,
                    spacing,
                })
            }
            "tooltip" => {
                use iced::widget::tooltip::Position;
                map_props!(
                    "tooltip",
                    props,
                    (gap, "gap", Pixels, f32),
                    (padding, "padding", Pixels, f32),
                    (snap_within_viewport, "snapWithinViewport", bool, bool),
                    (position, "position", Position, IcedTooltipPosition)
                );

                Ok(Tag::Tooltip {
                    padding,
                    gap,
                    snap_within_viewport,
                    position: position.unwrap_or_default(),
                })
            }

            "float" => {
                map_props!("float", props, (scale, "scale", f32, f32));

                Ok(Tag::Float { scale })
            }

            "canvas" => {
                unimplemented!()
            }

            "textarea" => {
                let state_id = obj.get::<_, String>("_stateId")?;

                Ok(Tag::Textarea {
                    state_id,
                    on_change: None,
                })
            }

            "input" => {
                let input_type = props.get::<_, String>("type")?;

                match input_type.as_str() {
                    "text" => {
                        let value = props.get::<_, String>("value")?;
                        let placeholder = props.get::<_, String>("placeholder")?;

                        let on_change = props.get::<_, CallbackId>("onChange").ok();
                        let on_submit = props.get::<_, CallbackId>("onSubmit").ok();

                        let disabled = props.get::<_, bool>("disabled").ok().unwrap_or_default();

                        Ok(Tag::TextInput {
                            value,
                            placeholder,
                            on_change,
                            on_submit,
                            disabled,
                        })
                    }
                    "switch" => {
                        let value = props.get::<_, bool>("checked")?;

                        Ok(Tag::Switch {
                            value,
                            on_change: None,
                        })
                    }
                    "checkbox" => {
                        let value = props.get::<_, bool>("checked")?;

                        Ok(Tag::Checkbox {
                            value,
                            disabled: false,
                            on_change: None,
                        })
                    }

                    _ => Err(rquickjs::Error::new_from_js_message(
                        "props.type",
                        "type",
                        "unknown input type",
                    )),
                }
            }

            #[cfg(feature = "code-element")]
            "code" => {
                unimplemented!()
            }

            #[cfg(feature = "img-element")]
            "img" => {
                unimplemented!()
            }

            #[cfg(feature = "markdown-element")]
            "markdown" => {
                unimplemented!()
            }
            #[cfg(feature = "qr-element")]
            "qr-code" => {
                unimplemented!()
            }
            #[cfg(feature = "svg-element")]
            "svg" => {
                let handle =
                    props.get::<_, rquickjs::Class<'js, crate::loaders::svg::SvgHandle>>("src")?;
                let src = handle.borrow().handle.clone();

                map_props!(
                    "svg",
                    props,
                    ignore["src"],
                    (width, "width", Length, IcedLength),
                    (height, "height", Length, IcedLength)
                );

                Ok(Tag::Svg { src, width, height })
            }

            _ => Err(rquickjs::Error::new_from_js_message(
                "props.type",
                "Tag",
                format!("unknown tag '{}'", tag_type),
            )),
        }
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tag::Col { .. } => write!(f, "col"),
            Tag::Row { .. } => write!(f, "row"),
            Tag::View { .. } => write!(f, "view"),
            Tag::Button { .. } => write!(f, "button"),
            Tag::Text { .. } => write!(f, "text"),

            Self::Switch { .. } => write!(f, "input(switch)"),
            Self::Scroll { .. } => write!(f, "scroll"),
            Self::Space { .. } => write!(f, "space"),
            Self::Hr { .. } => write!(f, "hr"),
            Self::Vr { .. } => write!(f, "vr"),
            Self::Tooltip { .. } => write!(f, "tooltip"),
            Self::Checkbox { .. } => write!(f, "input(checkbox)"),

            Self::Float { .. } => write!(f, "float"),
            Self::TextInput { .. } => write!(f, "input(text)"),
            Self::Textarea { .. } => write!(f, "textarea"),

            #[cfg(feature = "svg-element")]
            Tag::Svg { .. } => write!(f, "svg"),
        }
    }
}

impl Tag {
    pub fn valid_child_count(&self, len: usize) -> bool {
        match self {
            Tag::Space { .. }
            | Tag::Hr { .. }
            | Tag::Vr { .. }
            | Tag::TextInput { .. }
            | Tag::Checkbox { .. }
            | Tag::Switch { .. } => len == 0,
            Self::Float { .. }
            | Tag::View { .. }
            | Tag::Button { .. }
            | Tag::Text { .. }
            | Tag::Textarea { .. } => len == 1,
            Tag::Tooltip { .. } => len == 2,
            _ => true,
        }
    }
}
