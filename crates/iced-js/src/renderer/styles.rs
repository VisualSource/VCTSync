use iced::{Background, Border, Color, Shadow, Vector};
use rquickjs::FromJs;

use crate::renderer::props::{JsProp, Prop};

#[derive(Debug)]
pub enum ViewStyle {
    RoundedBox,
    Custom(iced::widget::container::Style),
}

impl JsProp for ViewStyle {
    fn from_prop(value: rquickjs::Value<'_>) -> rquickjs::Result<Self> {
        if let Ok(v) = value.get::<String>() {
            return match v.as_str() {
                "roundedBox" => Ok(ViewStyle::RoundedBox),
                _ => Err(rquickjs::Error::new_from_js_message(
                    "string",
                    "ViewStyle",
                    "unknown name",
                )),
            };
        }

        let obj = value.get::<rquickjs::Object<'_>>()?;

        let background = obj
            .get::<_, Prop<Background>>("background")
            .ok()
            .map(|x| x.0);
        let text_color = obj.get::<_, Prop<Color>>("color").ok().map(|x| x.0);

        let border = {
            let width = obj.get::<_, f32>("borderWidth").ok().unwrap_or_default();
            let color = obj
                .get::<_, IcedColor>("borderColor")
                .ok()
                .map(Color::from)
                .unwrap_or_default();
            let radius = obj
                .get::<_, IcedBorderRadius>("borderRadius")
                .map(iced::border::Radius::from)
                .unwrap_or_default();

            Border {
                color,
                width,
                radius,
            }
        };

        let shadow = {
            let color = obj
                .get::<_, IcedColor>("shadowColor")
                .ok()
                .map(Color::from)
                .unwrap_or_default();
            let blur_radius = obj
                .get::<_, f32>("shadowBlurRadius")
                .ok()
                .unwrap_or_default();

            let offset_y = obj.get::<_, f32>("shadowOffsetY").ok().unwrap_or_default();
            let offset_x = obj.get::<_, f32>("shadowOffsetX").ok().unwrap_or_default();

            Shadow {
                color,
                blur_radius,
                offset: Vector::new(offset_x, offset_y),
            }
        };

        let snap = obj.get::<_, bool>("snap").ok().unwrap_or_default();

        Ok(ViewStyle::Custom(iced::widget::container::Style {
            background,
            text_color,
            border,
            shadow,
            snap,
        }))
    }
}

#[derive(Debug)]
pub enum ButtonStyle {
    Text,
    Background,
    Success,
    Danger,
    Primary,
    Secondary,
    Warning,
    Subtle,
    Custom,
}

impl<'js> FromJs<'js> for ButtonStyle {
    fn from_js(
        _ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        if let Ok(value) = value.get::<String>() {
            return match value.as_str() {
                "text" => Ok(ButtonStyle::Text),
                "background" => Ok(ButtonStyle::Background),
                "success" => Ok(ButtonStyle::Success),
                "danger" => Ok(ButtonStyle::Danger),
                "primary" => Ok(ButtonStyle::Primary),
                "secondary" => Ok(ButtonStyle::Secondary),
                "warning" => Ok(ButtonStyle::Warning),
                "subtle" => Ok(ButtonStyle::Subtle),
                _ => Err(rquickjs::Error::new_from_js_message(
                    "string",
                    "ButtonStyle",
                    "unknown style preset",
                )),
            };
        }

        unimplemented!()
    }
}

#[cfg(feature = "svg-element")]
#[derive(Debug)]
pub enum SvgStyle {
    CurrentColor,
    Custom,
}

impl<'js> FromJs<'js> for SvgStyle {
    fn from_js(
        _ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        if let Ok(v) = value.get::<String>() {
            return match v.as_str() {
                "currentColor" => Ok(SvgStyle::CurrentColor),
                _ => Err(rquickjs::Error::new_from_js_message(
                    "string",
                    "SvgStyle",
                    "unknown preset for svg style",
                )),
            };
        }

        Err(rquickjs::Error::new_from_js_message(
            value.type_name(),
            "SvgStyle",
            "unsupported type",
        ))
    }
}
