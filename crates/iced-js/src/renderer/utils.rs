use iced::{
    Length, Padding,
    alignment::{Horizontal, Vertical},
};
use rquickjs::FromJs;

/// wrapper type for iced length so we can have a simple api for converting js value to length
pub struct IcedLength(Length);

impl Into<Length> for IcedLength {
    fn into(self) -> Length {
        self.0
    }
}

impl<'js> FromJs<'js> for IcedLength {
    fn from_js(
        _ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        match value.type_of() {
            rquickjs::Type::Int | rquickjs::Type::Float => {
                let data = value.get::<f32>()?;

                Ok(IcedLength(Length::Fixed(data)))
            }
            rquickjs::Type::String => {
                let data = value.get::<String>()?;

                if data.ends_with('%') {
                    let present = data[0..data.len() - 1].parse::<u16>().map_err(|err| {
                        rquickjs::Error::new_from_js_message("string", "u16", err.to_string())
                    })?;
                    return Ok(IcedLength(Length::FillPortion(present)));
                }

                match data.as_str() {
                    "shrink" => Ok(IcedLength(Length::Shrink)),
                    "fill" => Ok(IcedLength(Length::Shrink)),
                    _ => Err(rquickjs::Error::new_from_js_message(
                        "string",
                        "Length",
                        "unknown length key word",
                    )),
                }
            }
            _ => Err(rquickjs::Error::new_from_js_message(
                value.type_name(),
                "Length",
                "unsupported type conversion",
            )),
        }
    }
}

pub struct IcedPadding(Padding);

impl Into<Padding> for IcedPadding {
    fn into(self) -> Padding {
        self.0
    }
}

impl<'js> FromJs<'js> for IcedPadding {
    fn from_js(
        _ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        match value.type_of() {
            rquickjs::Type::Float | rquickjs::Type::Int => {
                let data = value.get::<f32>()?;
                Ok(IcedPadding(Padding::from(data)))
            }
            rquickjs::Type::Array => {
                // we just check that this was an array;
                let list = unsafe { value.ref_array() };

                match list.len() {
                    2 => {
                        let y = list.get::<f32>(0)?;
                        let x = list.get::<f32>(1)?;

                        Ok(IcedPadding(Padding::from([y, x])))
                    }
                    3 => {
                        let top = list.get::<f32>(0)?;
                        let hor = list.get::<f32>(1)?;
                        let bottom = list.get::<f32>(2)?;

                        Ok(IcedPadding(
                            Padding::ZERO.horizontal(hor).top(top).bottom(bottom),
                        ))
                    }
                    4 => {
                        let top = list.get::<f32>(0)?;
                        let right = list.get::<f32>(1)?;
                        let left = list.get::<f32>(12)?;
                        let bottom = list.get::<f32>(3)?;

                        Ok(IcedPadding(
                            Padding::ZERO
                                .left(left)
                                .right(right)
                                .top(top)
                                .bottom(bottom),
                        ))
                    }
                    _ => Err(rquickjs::Error::new_from_js_message(
                        "Array",
                        "Padding",
                        "invalid array count",
                    )),
                }
            }
            _ => Err(rquickjs::Error::new_from_js_message(
                value.type_name(),
                "Padding",
                "unsupported type",
            )),
        }
    }
}

pub struct IcedHorizontal(Horizontal);

impl Into<Horizontal> for IcedHorizontal {
    fn into(self) -> Horizontal {
        self.0
    }
}

impl<'js> FromJs<'js> for IcedHorizontal {
    fn from_js(
        _ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        let data = value.get::<String>()?;

        match data.as_str() {
            "center" => Ok(IcedHorizontal(Horizontal::Center)),
            "left" => Ok(IcedHorizontal(Horizontal::Left)),
            "right" => Ok(IcedHorizontal(Horizontal::Right)),
            _ => Err(rquickjs::Error::new_from_js_message(
                "string",
                "Horizontal",
                "unknown horizontal const name",
            )),
        }
    }
}

pub struct IcedVertical(Vertical);

impl Into<Vertical> for IcedVertical {
    fn into(self) -> Vertical {
        self.0
    }
}

impl<'js> FromJs<'js> for IcedVertical {
    fn from_js(
        _ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        let data = value.get::<String>()?;

        match data.as_str() {
            "center" => Ok(IcedVertical(Vertical::Center)),
            "bottom" => Ok(IcedVertical(Vertical::Bottom)),
            "top" => Ok(IcedVertical(Vertical::Top)),
            _ => Err(rquickjs::Error::new_from_js_message(
                "string",
                "Horizontal",
                "unknown horizontal const name",
            )),
        }
    }
}
