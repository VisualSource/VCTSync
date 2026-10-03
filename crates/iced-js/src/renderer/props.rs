use iced::{
    Length, Padding,
    alignment::{Horizontal, Vertical},
};
use rquickjs::FromJs;

/// wrapper type for iced length so we can have a simple api for converting js value to length
pub struct IcedLength(Length);

impl From<IcedLength> for Length {
    fn from(value: IcedLength) -> Self {
        value.0
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

impl From<IcedPadding> for Padding {
    fn from(value: IcedPadding) -> Self {
        value.0
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

impl From<IcedHorizontal> for Horizontal {
    fn from(value: IcedHorizontal) -> Horizontal {
        value.0
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

impl From<IcedVertical> for Vertical {
    fn from(value: IcedVertical) -> Vertical {
        value.0
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

pub struct IcedTooltipPosition(iced::widget::tooltip::Position);

impl From<IcedTooltipPosition> for iced::widget::tooltip::Position {
    fn from(value: IcedTooltipPosition) -> iced::widget::tooltip::Position {
        value.0
    }
}

impl<'js> FromJs<'js> for IcedTooltipPosition {
    fn from_js(
        _ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        use iced::widget::tooltip::Position;
        if !value.is_string() {
            return Err(rquickjs::Error::new_from_js_message(
                value.type_name(),
                "Position",
                "was expecting a string",
            ));
        }

        let data = value.get::<String>()?;

        match data.as_str() {
            "bottom" => Ok(IcedTooltipPosition(Position::Bottom)),
            "followCursor" => Ok(IcedTooltipPosition(Position::FollowCursor)),
            "left" => Ok(IcedTooltipPosition(Position::Left)),
            "right" => Ok(IcedTooltipPosition(Position::Right)),
            "top" => Ok(IcedTooltipPosition(Position::Top)),

            _ => Err(rquickjs::Error::new_from_js_message(
                "string",
                "Position",
                "invalid unknown position",
            )),
        }
    }
}

pub struct IcedId(iced::widget::Id);

impl From<IcedId> for iced::widget::Id {
    fn from(value: IcedId) -> Self {
        value.0
    }
}

impl<'js> FromJs<'js> for IcedId {
    fn from_js(
        _ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        let data = value.get::<String>()?;
        Ok(IcedId(data.into()))
    }
}
