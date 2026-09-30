use super::utils::{IcedHorizontal, IcedLength, IcedVertical};
use iced::{
    Color, Length, Padding, Pixels,
    alignment::Horizontal,
    alignment::Vertical,
    widget::text::{LineHeight, Shaping},
};
use rquickjs::FromJs;

type CallbackId = u64;

macro_rules! as_object {
    ($value: ident, $target: ident) => {{
        if !$value.is_object() {
            return Err(rquickjs::Error::FromJs {
                from: "object",
                to: "props",
                message: Some("was expecting an object".to_string()),
            });
        }

        let obj = unsafe { $value.ref_object() };

        if obj.len() == 0 {
            return Ok(Default::default());
        }

        obj
    }};
}

macro_rules! set_prop_opt {
    ($object: ident, $props: ident, $prop: ident, $to: path, $name: literal) => {
        if let Ok(value) = $object.get::<_, $to>($name) {
            $props.$prop = Some(value.into());
        }
    };
}

macro_rules! set_prop {
    ($object: ident, $props: ident, $prop: ident, $to: path, $name: literal) => {
        if let Ok(value) = $object.get::<_, $to>($name) {
            $props.$prop = value.into();
        }
    };
}

#[derive(Debug, Default)]
pub struct SpaceProps {
    pub width: Option<Length>,
    pub height: Option<Length>,
}

impl<'js> FromJs<'js> for SpaceProps {
    fn from_js(
        _ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        let obj = as_object!(value, Self);

        let mut props = SpaceProps::default();

        set_prop_opt!(obj, props, width, IcedLength, "width");
        set_prop_opt!(obj, props, height, IcedLength, "height");

        Ok(props)
    }
}

#[derive(Debug, Default)]
pub struct CommonProps {
    pub padding: Option<Padding>,
    pub height: Option<Length>,
    pub width: Option<Length>,
    pub align_x: Option<Horizontal>,
    pub align_y: Option<Vertical>,
    pub clip: Option<bool>,
    pub warp: Option<bool>,
}

impl<'js> FromJs<'js> for CommonProps {
    fn from_js(
        _ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        let obj = as_object!(value, Self);

        let mut props = Self::default();

        set_prop_opt!(obj, props, width, IcedLength, "width");
        set_prop_opt!(obj, props, height, IcedLength, "height");

        Ok(props)
    }
}

#[derive(Debug, Default)]
pub struct ViewProps {
    pub id: Option<iced::widget::Id>,
    pub padding: Option<Padding>,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub max_width: Option<Pixels>,
    pub max_height: Option<Pixels>,
    pub center_x: Option<Length>,
    pub center_y: Option<Length>,
    pub center: Option<Length>,
    pub align_left: Option<Length>,
    pub align_right: Option<Length>,
    pub align_top: Option<Length>,
    pub align_bottom: Option<Length>,
    pub align_x: Option<Horizontal>,
    pub align_y: Option<Vertical>,
    pub clip: Option<bool>,
}

impl<'js> FromJs<'js> for ViewProps {
    fn from_js(
        _ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        let obj = as_object!(value, Self);

        let mut props = Self::default();

        set_prop_opt!(obj, props, width, IcedLength, "width");
        set_prop_opt!(obj, props, height, IcedLength, "height");

        set_prop_opt!(obj, props, align_x, IcedHorizontal, "alignX");
        set_prop_opt!(obj, props, align_y, IcedVertical, "alignY");

        Ok(props)
    }
}

#[derive(Debug, Default)]
pub struct ButtonProps {
    // class
    pub clip: Option<bool>,
    pub on_press: Option<CallbackId>,
    //on_press_maybe
    //on_press_with
    pub padding: Option<Padding>,
    //style
    pub width: Option<Length>,
    pub height: Option<Length>,
}

impl<'js> FromJs<'js> for ButtonProps {
    fn from_js(
        _ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        let obj = as_object!(value, Self);

        let mut props = Self::default();

        set_prop_opt!(obj, props, width, IcedLength, "width");
        set_prop_opt!(obj, props, height, IcedLength, "height");
        set_prop_opt!(obj, props, on_press, CallbackId, "onPress");
        set_prop_opt!(obj, props, clip, bool, "clip");

        Ok(props)
    }
}

#[derive(Debug, Default)]
pub struct TextProps {
    pub size: Option<Pixels>,
    pub line_height: Option<LineHeight>,
    // font object,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub align_x: Option<Horizontal>,
    pub align_y: Option<Vertical>,
    pub wrapping: Option<bool>,
    pub on_link_click: Option<CallbackId>,
    // style callback
    pub color: Option<Color>,
    // colorMaybe
    // font
    // fontMaybe
    pub center: bool,
    pub shaping: Option<Shaping>,
}

impl<'js> FromJs<'js> for TextProps {
    fn from_js(
        _ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        let obj = as_object!(value, Self);

        let mut props = Self::default();

        set_prop_opt!(obj, props, align_x, IcedHorizontal, "alignX");
        set_prop_opt!(obj, props, align_y, IcedVertical, "alignY");
        set_prop_opt!(obj, props, width, IcedLength, "width");
        set_prop_opt!(obj, props, height, IcedLength, "height");
        set_prop!(obj, props, center, bool, "center");

        Ok(props)
    }
}

#[cfg(feature = "svg-element")]
#[derive(Debug)]
pub struct SvgProps {
    pub src: iced::widget::svg::Handle,
    pub width: Option<Length>,
    pub height: Option<Length>,
}

#[cfg(feature = "svg-element")]
impl<'js> FromJs<'js> for SvgProps {
    fn from_js(
        _ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        if !value.is_object() {
            return Err(rquickjs::Error::FromJs {
                from: "object",
                to: "props",
                message: Some("was expecting an object".to_string()),
            });
        }

        let obj = unsafe { value.ref_object() };

        if obj.len() == 0 {
            return Err(rquickjs::Error::new_from_js_message(
                "props",
                "SvgProps",
                "missing props",
            ));
        }
        let src = obj.get::<_, rquickjs::Class<'js, crate::loaders::svg::SvgHandle>>("src")?;

        let handle = src.borrow().handle.clone();

        let mut props = SvgProps {
            src: handle,
            width: None,
            height: None,
        };

        set_prop_opt!(obj, props, width, IcedLength, "width");
        set_prop_opt!(obj, props, height, IcedLength, "height");

        Ok(props)
    }
}
