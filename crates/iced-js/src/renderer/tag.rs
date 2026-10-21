use super::props::{Keyframe, Prop, SelectOption};
use super::styles::{ButtonStyle, ThemeColor, ViewStyle};

use iced::{
    Font, Length, Padding, Pixels,
    alignment::{Horizontal, Vertical},
    animation::Easing,
    widget::{
        self,
        text::{LineHeight, Shaping, Wrapping},
    },
};
use rquickjs::FromJs;
use std::fmt::Display;

type CallbackId = u64;

#[derive(Debug)]
pub enum Tag {
    /// iced columns only align children horizontally, so there is no `align_y`.
    Col {
        padding: Option<Padding>,
        height: Option<Length>,
        width: Option<Length>,
        align_x: Option<Horizontal>,
        clip: Option<bool>,
        warp: Option<bool>,
        spacing: Option<Pixels>,
    },
    /// iced rows only align children vertically, so there is no `align_x`.
    Row {
        padding: Option<Padding>,
        height: Option<Length>,
        width: Option<Length>,
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
        clip: Option<bool>,
        on_press: Option<CallbackId>,
        padding: Option<Padding>,
        width: Option<Length>,
        height: Option<Length>,
        disabled: bool,
        style: Option<ButtonStyle>,
    },
    Text {
        font: Option<Font>,
        size: Option<Pixels>,
        line_height: Option<LineHeight>,
        width: Option<Length>,
        height: Option<Length>,
        align_x: Option<Horizontal>,
        align_y: Option<Vertical>,
        wrapping: Option<Wrapping>,
        color: Option<ThemeColor>,
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
        spacing: Option<Pixels>,
    },
    Space {
        width: Option<Length>,
        height: Option<Length>,
    },
    Hr {
        height: Pixels,
    },
    Vr {
        width: Pixels,
    },
    Tooltip {
        padding: Option<Pixels>,
        gap: Option<Pixels>,
        snap_within_viewport: Option<bool>,
        position: iced::widget::tooltip::Position,
    },
    Float {
        scale: Option<f32>,
    },
    Anim {
        active: bool,
        from: Option<Keyframe>,
        to: Option<Keyframe>,
        duration: Option<u64>,
        easing: Option<Easing>,
        hide_when_inactive: bool,
        on_settle: Option<CallbackId>,
    },
    TextInput {
        placeholder: String,
        value: String,
        on_change: Option<CallbackId>,
        on_submit: Option<CallbackId>,
        disabled: bool,
        size: Option<Pixels>,
        padding: Option<Padding>,
    },
    Checkbox {
        value: bool,
        on_change: Option<CallbackId>,
        disabled: bool,
        label: Option<String>,
    },
    Switch {
        checked: bool,
        on_change: Option<CallbackId>,
        label: Option<String>,
        disabled: bool,
        width: Option<Length>,
        spacing: Option<Pixels>,
        font: Option<Font>,
        text_size: Option<Pixels>,
        size: Option<Pixels>,
    },
    Textarea {
        state_id: String,
        on_change: Option<CallbackId>,
    },
    Select {
        width: Option<Length>,
        placeholder: Option<String>,
        on_select: CallbackId,
        value: Option<String>,
        options: Vec<SelectOption>,
    },
    Stack {
        height: Option<Length>,
        width: Option<Length>,
        clip: Option<bool>,
    },
    Opaque,
    Progress {
        value: f32,
        max: Option<f32>,
        width: Option<Length>,
        height: Option<Length>,
        /// Named palette: primary | secondary | success | warning | danger.
        style: Option<String>,
    },
    #[cfg(feature = "img-element")]
    Img {
        src: iced::widget::image::Handle,
        width: Option<Length>,
        height: Option<Length>,
        opacity: Option<f32>,
        border_radius: Option<f32>,
    },
    #[cfg(feature = "svg-element")]
    Svg {
        src: iced::widget::svg::Handle,
        width: Option<Length>,
        height: Option<Length>,
        color: Option<super::styles::ThemeColor>,
        opacity: Option<f32>,
    },
}

macro_rules! map_props {
    ($el: literal, $props: ident, ignore [$($ignored: literal),* $(,)?], $( ($name: ident, $propKey: literal, $type: ident) ), *) => {
        $(
            let mut $name: Option<$type> = None;
        )*

        for prop in $props.props::<String, rquickjs::Value<'js>>() {
            let (key, value) = prop?;
            match key.as_str() {
                $(
                    $propKey => $name = value.get::<Prop<$type>>().ok().map(|x|x.0),
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
    ($el: literal, $props: ident, $( ($name: ident, $propKey: literal, $type: ident) ), *) => {
        map_props!($el, $props, ignore [], $( ($name, $propKey, $type) ),*)
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
            // An off-axis alignX/alignY falls through to map_props' "unknown prop" warning.
            "row" => {
                map_props!(
                    "row",
                    props,
                    (padding, "padding", Padding),
                    (width, "width", Length),
                    (height, "height", Length),
                    (align_y, "alignY", Vertical),
                    (clip, "clip", bool),
                    (warp, "warp", bool),
                    (spacing, "spacing", Pixels)
                );
                Ok(Tag::Row {
                    padding,
                    height,
                    width,
                    align_y,
                    clip,
                    warp,
                    spacing,
                })
            }
            "col" => {
                map_props!(
                    "col",
                    props,
                    (padding, "padding", Padding),
                    (width, "width", Length),
                    (height, "height", Length),
                    (align_x, "alignX", Horizontal),
                    (clip, "clip", bool),
                    (warp, "warp", bool),
                    (spacing, "spacing", Pixels)
                );
                Ok(Tag::Col {
                    padding,
                    height,
                    width,
                    align_x,
                    clip,
                    warp,
                    spacing,
                })
            }
            "view" => {
                use iced::widget::Id;

                map_props!(
                    "view",
                    props,
                    (padding, "padding", Padding),
                    (width, "width", Length),
                    (height, "height", Length),
                    (max_width, "maxWidth", Pixels),
                    (max_height, "maxHeight", Pixels),
                    (center_x, "centerX", Length),
                    (center_y, "centerY", Length),
                    (center, "center", Length),
                    (align_left, "alignLeft", Length),
                    (align_right, "alignRight", Length),
                    (align_top, "alignTop", Length),
                    (align_bottom, "alignBottom", Length),
                    (align_x, "alignX", Horizontal),
                    (align_y, "alignY", Vertical),
                    (clip, "clip", bool),
                    (id, "id", Id),
                    (style, "style", ViewStyle)
                );

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
                    (clip, "clip", bool),
                    (on_press, "onPress", CallbackId),
                    (padding, "padding", Padding),
                    (width, "width", Length),
                    (height, "height", Length),
                    (disabled, "disabled", bool),
                    (style, "style", ButtonStyle)
                );

                Ok(Tag::Button {
                    style,
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
                    (size, "size", Pixels),
                    (width, "width", Length),
                    (height, "height", Length),
                    (align_x, "alignX", Horizontal),
                    (align_y, "alignY", Vertical),
                    (wrapping, "wrapping", Wrapping),
                    (font, "font", Font),
                    (color, "color", ThemeColor),
                    (center, "center", bool)
                );

                Ok(Tag::Text {
                    size,
                    font,
                    line_height: None,
                    width,
                    height,
                    align_x,
                    align_y,
                    wrapping,
                    color,
                    center,
                    shaping: None,
                })
            }
            "space" => {
                map_props!(
                    "space",
                    props,
                    (width, "width", Length),
                    (height, "height", Length)
                );

                Ok(Tag::Space { width, height })
            }
            "hr" => {
                map_props!("hr", props, (height, "height", Pixels));

                Ok(Tag::Hr {
                    height: height.unwrap_or_else(|| Pixels::from(1)),
                })
            }
            "vr" => {
                map_props!("vr", props, (width, "width", Pixels));
                Ok(Tag::Vr {
                    width: width.unwrap_or_else(|| Pixels::from(1)),
                })
            }
            "scroll" => {
                map_props!(
                    "scroll",
                    props,
                    (height, "height", Length),
                    (width, "width", Length),
                    (horizontal, "horizontal", bool),
                    (anchor_bottom, "anchorBottom", bool),
                    (anchor_left, "anchorLeft", bool),
                    (anchor_right, "anchorRight", bool),
                    (anchor_top, "anchorTop", bool),
                    (auto_scroll, "autoScroll", bool),
                    (spacing, "spacing", Pixels)
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
                    (gap, "gap", Pixels),
                    (padding, "padding", Pixels),
                    (snap_within_viewport, "snapWithinViewport", bool),
                    (position, "position", Position)
                );

                Ok(Tag::Tooltip {
                    padding,
                    gap,
                    snap_within_viewport,
                    position: position.unwrap_or_default(),
                })
            }
            "float" => {
                map_props!("float", props, (scale, "scale", f32));

                Ok(Tag::Float { scale })
            }
            "anim" => {
                map_props!(
                    "anim",
                    props,
                    (active, "active", bool),
                    (from, "from", Keyframe),
                    (to, "to", Keyframe),
                    (duration, "duration", u64),
                    (easing, "easing", Easing),
                    (hide_when_inactive, "hideWhenInactive", bool),
                    (on_settle, "onSettle", CallbackId)
                );

                Ok(Tag::Anim {
                    active: active.unwrap_or_default(),
                    from,
                    to,
                    duration,
                    easing,
                    hide_when_inactive: hide_when_inactive.unwrap_or_default(),
                    on_settle,
                })
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

                        map_props!(
                            "input(text)",
                            props,
                            ignore["value","type"],
                            (placeholder, "placeholder", String),
                            (on_change, "onChange", CallbackId),
                            (on_submit, "onSubmit", CallbackId),
                            (disabled, "disabled", bool),
                            (padding, "padding", Padding),
                            (size, "size", Pixels)
                        );

                        Ok(Tag::TextInput {
                            value,
                            placeholder: placeholder.unwrap_or_default(),
                            on_change,
                            on_submit,
                            size,
                            padding,
                            disabled: disabled.unwrap_or_default(),
                        })
                    }
                    "switch" => {
                        map_props!(
                            "switch",
                            props,
                            ignore["type"],
                            (on_change, "onChange", CallbackId),
                            (label, "label", String),
                            (disabled, "disabled", bool),
                            (width, "width", Length),
                            (spacing, "spacing", Pixels),
                            (size, "size", Pixels),
                            (font, "font", Font),
                            (text_size, "textSize", Pixels),
                            (checked, "checked", bool)
                        );

                        Ok(Tag::Switch {
                            checked: checked.unwrap_or_default(),
                            on_change,
                            label,
                            disabled: disabled.unwrap_or_default(),
                            width,
                            spacing,
                            size,
                            font,
                            text_size,
                        })
                    }
                    "checkbox" => {
                        let value = props.get::<_, bool>("checked")?;
                        let label = props.get::<_, String>("label").ok();
                        let disabled = props.get::<_, bool>("disabled").ok();
                        let callback = props.get::<_, CallbackId>("onChange").ok();

                        Ok(Tag::Checkbox {
                            value,
                            label,
                            disabled: disabled.unwrap_or_default(),
                            on_change: callback,
                        })
                    }

                    _ => Err(rquickjs::Error::new_from_js_message(
                        "props.type",
                        "type",
                        "unknown input type",
                    )),
                }
            }
            "select" => {
                map_props!(
                    "select",
                    props,
                    ignore["src","onSelect","options"],
                    (width, "width", Length),
                    (placeholder, "placeholder", String),
                    (value, "value", String)
                );

                let on_select = props.get::<_, CallbackId>("onSelect")?;
                let options = props.get::<_, Vec<SelectOption>>("options")?;

                Ok(Tag::Select {
                    width,
                    placeholder,
                    on_select,
                    value,
                    options,
                })
            }
            "stack" => {
                map_props!(
                    "stack",
                    props,
                    (width, "width", Length),
                    (height, "height", Length),
                    (clip, "clip", bool)
                );

                Ok(Tag::Stack {
                    width,
                    height,
                    clip,
                })
            }
            "opaque" => Ok(Tag::Opaque),

            #[cfg(feature = "code-element")]
            "code" => {
                unimplemented!()
            }

            #[cfg(feature = "img-element")]
            "progress" => {
                map_props!(
                    "progress",
                    props,
                    (value, "value", f32),
                    (max, "max", f32),
                    (width, "width", Length),
                    (height, "height", Length),
                    (style, "style", String)
                );

                Ok(Tag::Progress {
                    value: value.unwrap_or(0.0),
                    max,
                    width,
                    height,
                    style,
                })
            }

            #[cfg(feature = "img-element")]
            "img" => {
                // Local file path; iced caches the decoded image by path.
                let src: String = props.get("src")?;

                map_props!(
                    "img",
                    props,
                    ignore["src"],
                    (width, "width", Length),
                    (height, "height", Length),
                    (opacity, "opacity", f32),
                    (border_radius, "borderRadius", f32)
                );

                Ok(Tag::Img {
                    src: iced::widget::image::Handle::from_path(src),
                    width,
                    height,
                    opacity,
                    border_radius,
                })
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
                use super::styles::ThemeColor;
                let handle =
                    props.get::<_, rquickjs::Class<'js, crate::loaders::svg::SvgHandle>>("src")?;
                let src = handle.borrow().handle.clone();

                map_props!(
                    "svg",
                    props,
                    ignore["src"],
                    (width, "width", Length),
                    (height, "height", Length),
                    (color, "color", ThemeColor),
                    (opacity, "opacity", f32)
                );

                Ok(Tag::Svg {
                    src,
                    width,
                    height,
                    color,
                    opacity,
                })
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
            Tag::Opaque => write!(f, "opaque"),
            Tag::Stack { .. } => write!(f, "stack"),
            Tag::Select { .. } => write!(f, "select"),
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
            Self::Anim { .. } => write!(f, "anim"),
            Self::TextInput { .. } => write!(f, "input(text)"),
            Self::Textarea { .. } => write!(f, "textarea"),

            Tag::Progress { .. } => write!(f, "progress"),
            #[cfg(feature = "img-element")]
            Tag::Img { .. } => write!(f, "img"),
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
            | Tag::Switch { .. }
            | Tag::Progress { .. } => len == 0,
            Self::Float { .. }
            | Tag::View { .. }
            | Tag::Button { .. }
            | Tag::Text { .. }
            | Tag::Textarea { .. }
            | Tag::Opaque => len == 1,
            // Empty while a hidden `<Activity>` inside it has not rendered its
            // children yet.
            Self::Anim { .. } => len <= 1,
            Tag::Tooltip { .. } => len == 2,
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::props::Offset;
    use rquickjs::{Context, Runtime};

    fn parse(js: &str) -> Tag {
        let rt = Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| ctx.eval::<Tag, _>(js).unwrap())
    }

    #[test]
    fn anim_parses_all_props() {
        let tag = parse(
            r#"({ type: "anim", props: {
                active: true,
                from: { x: "100%", scale: 0.5 },
                to: { y: -8 },
                duration: 220,
                easing: "easeInOut",
                hideWhenInactive: true,
                onSettle: 7,
            } })"#,
        );

        let Tag::Anim {
            active,
            from,
            to,
            duration,
            easing,
            hide_when_inactive,
            ..
        } = tag
        else {
            panic!("expected anim, got {tag}");
        };

        assert!(active);
        assert_eq!(
            from,
            Some(Keyframe {
                x: Offset::Percent(100.0),
                y: Offset::Px(0.0),
                scale: 0.5,
            })
        );
        assert_eq!(
            to,
            Some(Keyframe {
                x: Offset::Px(0.0),
                y: Offset::Px(-8.0),
                scale: 1.0,
            })
        );
        assert_eq!(duration, Some(220));
        assert_eq!(easing, Some(Easing::EaseInOut));
        assert!(hide_when_inactive);
        assert!(matches!(
            tag,
            Tag::Anim {
                on_settle: Some(7),
                ..
            }
        ));
    }

    #[test]
    fn anim_defaults() {
        let tag = parse(r#"({ type: "anim", props: {} })"#);

        let Tag::Anim {
            active,
            from,
            to,
            duration,
            easing,
            hide_when_inactive,
            ..
        } = tag
        else {
            panic!("expected anim, got {tag}");
        };

        assert!(!active);
        assert_eq!(from, None);
        assert_eq!(to, None);
        assert_eq!(duration, None);
        assert_eq!(easing, None);
        assert!(!hide_when_inactive);
    }

    #[test]
    fn offset_resolves_against_size() {
        assert_eq!(Offset::Percent(50.0).resolve(300.0), 150.0);
        assert_eq!(Offset::Px(12.0).resolve(300.0), 12.0);
    }
}
