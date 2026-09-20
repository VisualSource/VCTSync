use iced::{
    Color, Length, Padding, Pixels,
    alignment::{Horizontal, Vertical},
    widget::text::{LineHeight, Shaping},
};
use rquickjs::{Error, FromJs, Value};

type CallbackId = u64;

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
    fn from_js(_ctx: &rquickjs::prelude::Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let mut props = CommonProps::default();

        if !value.is_object() {
            return Err(Error::FromJs {
                from: "",
                to: "",
                message: Some("was expecting an object".to_string()),
            });
        }

        let obj = unsafe { value.ref_object() };

        if obj.len() == 0 {
            return Ok(props);
        }

        if let Ok(value) = obj.get::<_, Value<'_>>("width") {
            props.width = as_length(value)?;
        }

        if let Ok(value) = obj.get::<_, Value<'_>>("height") {
            props.height = as_length(value)?;
        }

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
    fn from_js(_ctx: &rquickjs::prelude::Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let mut props = Self::default();
        if !value.is_object() {
            return Err(Error::FromJs {
                from: "",
                to: "",
                message: Some("was expecting an object".to_string()),
            });
        }

        let obj = unsafe { value.ref_object() };

        if obj.len() == 0 {
            return Ok(props);
        }

        if let Ok(value) = obj.get::<_, Value<'_>>("width") {
            props.width = as_length(value)?;
        }

        if let Ok(value) = obj.get::<_, Value<'_>>("height") {
            props.height = as_length(value)?;
        }

        if let Ok(value) = obj.get::<_, String>("alignX") {
            props.align_x = Some(as_align_hor(value)?);
        }

        if let Ok(value) = obj.get::<_, String>("alignY") {
            props.align_y = Some(as_align_vert(value)?);
        }
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
    fn from_js(_ctx: &rquickjs::prelude::Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let mut props = Self::default();

        if !value.is_object() {
            return Err(Error::FromJs {
                from: "",
                to: "",
                message: Some("was expecting an object".to_string()),
            });
        }

        let obj = unsafe { value.ref_object() };

        if obj.len() == 0 {
            return Ok(props);
        }

        if let Ok(value) = obj.get::<_, u64>("onPress") {
            props.on_press = Some(value);
        }

        if let Ok(value) = obj.get::<_, Value<'_>>("width") {
            props.width = as_length(value)?;
        }

        if let Ok(value) = obj.get::<_, Value<'_>>("height") {
            props.height = as_length(value)?;
        }

        if let Ok(value) = obj.get::<_, bool>("clip") {
            props.clip = Some(value);
        }

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
    fn from_js(_ctx: &rquickjs::prelude::Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let mut props = Self::default();

        if !value.is_object() {
            return Err(Error::FromJs {
                from: "",
                to: "",
                message: Some("was expecting an object".to_string()),
            });
        }

        let obj = unsafe { value.ref_object() };

        if obj.len() == 0 {
            return Ok(props);
        }

        if let Ok(value) = obj.get::<_, String>("alignX") {
            props.align_x = Some(as_align_hor(value)?);
        }

        if let Ok(value) = obj.get::<_, String>("alignY") {
            props.align_y = Some(as_align_vert(value)?);
        }

        if let Ok(value) = obj.get::<_, Value<'_>>("width") {
            props.width = as_length(value)?;
        }

        if let Ok(value) = obj.get::<_, Value<'_>>("height") {
            props.height = as_length(value)?;
        }

        if let Ok(value) = obj.get::<_, bool>("center") {
            props.center = value;
        }

        Ok(props)
    }
}

fn as_length(node: Value<'_>) -> Result<Option<Length>, Error> {
    match node.type_of() {
        rquickjs::Type::Int => {
            let i = node.as_int().expect("should be a int");

            Ok(Some(Length::Fixed(i as f32)))
        }
        rquickjs::Type::Float => {
            let i = node.as_float().expect("should have been as float");
            Ok(Some(Length::Fixed(i as f32)))
        }
        rquickjs::Type::String => {
            let s = unsafe { node.ref_string() }; // we just check that this is a string.
            let v = s.to_string()?;

            if v.ends_with('%') {
                let a = v[0..v.len() - 1]
                    .parse::<u16>()
                    .map_err(|err| Error::FromJs {
                        from: "",
                        to: "",
                        message: Some(err.to_string()),
                    })?;
                return Ok(Some(Length::FillPortion(a)));
            }

            match v.as_str() {
                "shrink" => Ok(Some(Length::Shrink)),
                "fill" => Ok(Some(Length::Fill)),
                _ => Err(Error::Unknown),
            }
        }
        _ => Ok(None),
    }
}

fn as_align_hor(value: String) -> Result<Horizontal, Error> {
    match value.as_str() {
        "center" => Ok(Horizontal::Center),
        "left" => Ok(Horizontal::Left),
        "right" => Ok(Horizontal::Right),
        _ => Err(Error::FromJs {
            from: "",
            to: "",
            message: Some("Unknown horizontal align value".to_string()),
        }),
    }
}

fn as_align_vert(value: String) -> Result<Vertical, Error> {
    match value.as_str() {
        "center" => Ok(Vertical::Center),
        "bottom" => Ok(Vertical::Bottom),
        "top" => Ok(Vertical::Top),
        _ => Err(Error::FromJs {
            from: "",
            to: "",
            message: Some("Unknown horizontal align value".to_string()),
        }),
    }
}
