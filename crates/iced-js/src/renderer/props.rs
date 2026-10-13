use std::fmt::Display;

use iced::{
    Background, Color, Font, Length, Padding,
    alignment::{Horizontal, Vertical},
    font::Weight,
};
use rquickjs::{FromJs, Object};

pub trait JsProp: Sized {
    fn from_prop(value: rquickjs::Value<'_>) -> rquickjs::Result<Self>;
}

macro_rules! impl_via_from_js {
    ($type: ident) => {
        impl JsProp for $type {
            fn from_prop(value: rquickjs::Value<'_>) -> rquickjs::Result<Self> {
                value.get::<$type>()
            }
        }
    };
}

pub struct Prop<T>(pub T);

impl<'js, T: JsProp> FromJs<'js> for Prop<T> {
    fn from_js(
        _ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        let inner = T::from_prop(value)?;
        Ok(Self(inner))
    }
}

impl_via_from_js!(f32);
impl_via_from_js!(String);
impl_via_from_js!(bool);
impl_via_from_js!(u64);

impl JsProp for Length {
    fn from_prop(v: rquickjs::Value<'_>) -> rquickjs::Result<Self> {
        match v.type_of() {
            rquickjs::Type::Int | rquickjs::Type::Float => {
                let data = v.get::<f32>()?;

                Ok(Length::Fixed(data))
            }
            rquickjs::Type::String => {
                let data = v.get::<String>()?;

                if data.ends_with('%') {
                    let present = data[0..data.len() - 1].parse::<u16>().map_err(|err| {
                        rquickjs::Error::new_from_js_message("string", "u16", err.to_string())
                    })?;
                    return Ok(Length::FillPortion(present));
                }

                match data.as_str() {
                    "shrink" => Ok(Length::Shrink),
                    "fill" => Ok(Length::Fill),
                    _ => Err(rquickjs::Error::new_from_js_message(
                        "string",
                        "Length",
                        "unknown length key word",
                    )),
                }
            }
            _ => Err(rquickjs::Error::new_from_js_message(
                v.type_name(),
                "Length",
                "unsupported type conversion",
            )),
        }
    }
}

impl JsProp for Padding {
    fn from_prop(v: rquickjs::Value<'_>) -> rquickjs::Result<Self> {
        match v.type_of() {
            rquickjs::Type::Float | rquickjs::Type::Int => Ok(Padding::from(v.get::<f32>()?)),
            rquickjs::Type::Array => {
                // we just check that this was an array;
                let list = unsafe { v.ref_array() };

                match list.len() {
                    2 => {
                        let y = list.get::<f32>(0)?;
                        let x = list.get::<f32>(1)?;

                        Ok(Padding::from([y, x]))
                    }
                    3 => {
                        let top = list.get::<f32>(0)?;
                        let hor = list.get::<f32>(1)?;
                        let bottom = list.get::<f32>(2)?;

                        Ok(Padding::ZERO.horizontal(hor).top(top).bottom(bottom))
                    }
                    4 => {
                        let top = list.get::<f32>(0)?;
                        let right = list.get::<f32>(1)?;
                        let bottom = list.get::<f32>(2)?;
                        let left = list.get::<f32>(3)?;

                        Ok(Padding::ZERO
                            .left(left)
                            .right(right)
                            .top(top)
                            .bottom(bottom))
                    }
                    _ => Err(rquickjs::Error::new_from_js_message(
                        "Array",
                        "Padding",
                        "invalid array count",
                    )),
                }
            }
            _ => Err(rquickjs::Error::new_from_js_message(
                v.type_name(),
                "Padding",
                "unsupported type",
            )),
        }
    }
}

impl JsProp for Horizontal {
    fn from_prop(v: rquickjs::Value<'_>) -> rquickjs::Result<Self> {
        let data = v.get::<String>()?;

        match data.as_str() {
            "center" => Ok(Horizontal::Center),
            "left" => Ok(Horizontal::Left),
            "right" => Ok(Horizontal::Right),
            _ => Err(rquickjs::Error::new_from_js_message(
                "string",
                "Horizontal",
                "unknown horizontal const name",
            )),
        }
    }
}

impl JsProp for Vertical {
    fn from_prop(v: rquickjs::Value<'_>) -> rquickjs::Result<Self> {
        let data = v.get::<String>()?;

        match data.as_str() {
            "center" => Ok(Vertical::Center),
            "bottom" => Ok(Vertical::Bottom),
            "top" => Ok(Vertical::Top),
            _ => Err(rquickjs::Error::new_from_js_message(
                "string",
                "Horizontal",
                "unknown horizontal const name",
            )),
        }
    }
}

impl JsProp for iced::widget::tooltip::Position {
    fn from_prop(v: rquickjs::Value<'_>) -> rquickjs::Result<Self> {
        use iced::widget::tooltip::Position;
        let data = v.get::<String>()?;

        match data.as_str() {
            "bottom" => Ok(Position::Bottom),
            "followCursor" => Ok(Position::FollowCursor),
            "left" => Ok(Position::Left),
            "right" => Ok(Position::Right),
            "top" => Ok(Position::Top),

            _ => Err(rquickjs::Error::new_from_js_message(
                "string",
                "Position",
                "invalid unknown position",
            )),
        }
    }
}

impl JsProp for iced::widget::Id {
    fn from_prop(v: rquickjs::Value<'_>) -> rquickjs::Result<Self> {
        let data = v.get::<String>()?;
        Ok(data.into())
    }
}

impl JsProp for Font {
    fn from_prop(value: rquickjs::Value<'_>) -> rquickjs::Result<Self> {
        let font = if let Ok(value) = value.get::<String>() {
            match value.as_str() {
                "default" => Font::DEFAULT,
                "monospace" => Font::MONOSPACE,
                _ => {
                    return Err(rquickjs::Error::new_from_js_message(
                        "string",
                        "Font",
                        "invalid font",
                    ));
                }
            }
        } else if let Ok(value) = value.get::<Object<'_>>() {
            let weight = value
                .get::<_, String>("weight")
                .ok()
                .map(|w| match w.as_str() {
                    "extraLight" => Weight::ExtraLight,
                    "light" => Weight::Light,
                    "thin" => Weight::Thin,
                    "normal" => Weight::Normal,
                    "medium" => Weight::Medium,
                    "semibold" => Weight::Semibold,
                    "bold" => Weight::Bold,
                    "extraBold" => Weight::ExtraBold,
                    "black" => Weight::Black,
                    _ => Weight::default(),
                })
                .unwrap_or_default();

            Font {
                weight,
                ..Default::default()
            }
        } else {
            return Err(rquickjs::Error::new_from_js_message(
                value.type_name(),
                "Font",
                "cannot convert type to Font",
            ));
        };

        Ok(font)
    }
}

impl JsProp for Color {
    fn from_prop(value: rquickjs::Value<'_>) -> rquickjs::Result<Self> {
        let v = value.get::<String>()?;

        let c = v.parse::<Color>().map_err(|err| {
            rquickjs::Error::new_from_js_message("string", "color", err.to_string())
        })?;

        Ok(c)
    }
}

impl JsProp for Background {
    fn from_prop(value: rquickjs::Value<'_>) -> rquickjs::Result<Self> {
        let value = value.get::<Prop<Color>>()?;
        Ok(Background::Color(value.0))
    }
}

impl JsProp for iced::border::Radius {
    fn from_prop(value: rquickjs::Value<'_>) -> rquickjs::Result<Self> {
        use iced::border::Radius;

        if let Ok(value) = value.get::<f32>() {
            return Ok(iced::border::radius(value));
        }
        let value = value.get::<String>()?;

        let mut data = Vec::with_capacity(4);
        for item in value.split_whitespace().take(4) {
            let r = item.parse::<f32>().map_err(|err| {
                rquickjs::Error::new_from_js_message("string", "f32", err.to_string())
            })?;

            data.push(r);
        }

        match data.len() {
            1 => Ok(iced::border::radius(data[0])),
            2 => Ok(Radius {
                top_left: data[0],
                top_right: data[0],
                bottom_right: data[1],
                bottom_left: data[1],
            }),
            3 => Ok(Radius {
                top_left: data[0],
                top_right: data[1],
                bottom_right: data[2],
                bottom_left: data[1],
            }),
            4 => Ok(Radius {
                top_left: data[0],
                top_right: data[1],
                bottom_right: data[2],
                bottom_left: data[3],
            }),
            _ => Err(rquickjs::Error::new_from_js_message(
                "string",
                "Radius",
                "invalid count of readies",
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

impl PartialEq for SelectOption {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl Display for SelectOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

impl<'js> FromJs<'js> for SelectOption {
    fn from_js(
        _ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        if let Some(value) = value.get::<String>().ok() {
            return Ok(Self {
                label: value.clone(),
                value,
            });
        }

        let obj = value.get::<Object<'js>>()?;

        let label = obj.get::<_, String>("label")?;
        let value = obj.get::<_, String>("value")?;

        Ok(Self { value, label })
    }
}
