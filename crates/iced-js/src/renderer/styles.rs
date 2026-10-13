use crate::renderer::props::{JsProp, Prop, to_inner};
use iced::{Border, Color, Shadow, Theme, Vector, theme::palette::Pair};

#[derive(Debug, Clone, Copy)]
pub enum PaletteFamily {
    Background,
    Primary,
    Secondary,
    Success,
    Warning,
    Danger,
}

#[derive(Debug, Clone, Copy)]
pub enum PaletteShade {
    Base,
    Weakest,
    Weaker,
    Weak,
    Neutral,
    Strong,
    Stronger,
    Strongest,
}

/// A color that is either fixed or looked up from the current theme's palette
/// (e.g. `"background"`, `"background.weak"`, `"primary.strong.text"`).
#[derive(Debug, Clone, Copy)]
pub enum ThemeColor {
    Fixed(Color),
    Palette {
        family: PaletteFamily,
        shade: PaletteShade,
        text: bool,
    },
}

impl Default for ThemeColor {
    fn default() -> Self {
        ThemeColor::Fixed(Color::default())
    }
}

impl ThemeColor {
    fn parse_token(value: &str) -> Option<Self> {
        let mut parts = value.split('.');

        let family = match parts.next()? {
            "background" => PaletteFamily::Background,
            "primary" => PaletteFamily::Primary,
            "secondary" => PaletteFamily::Secondary,
            "success" => PaletteFamily::Success,
            "warning" => PaletteFamily::Warning,
            "danger" => PaletteFamily::Danger,
            _ => return None,
        };

        let mut shade = PaletteShade::Base;
        let mut text = false;

        for part in parts {
            match part {
                "text" if !text => text = true,
                _ if text => return None,
                "base" => shade = PaletteShade::Base,
                "weak" => shade = PaletteShade::Weak,
                "strong" => shade = PaletteShade::Strong,
                "weakest" | "weaker" | "neutral" | "stronger" | "strongest"
                    if !matches!(family, PaletteFamily::Background) =>
                {
                    return None;
                }
                "weakest" => shade = PaletteShade::Weakest,
                "weaker" => shade = PaletteShade::Weaker,
                "neutral" => shade = PaletteShade::Neutral,
                "stronger" => shade = PaletteShade::Stronger,
                "strongest" => shade = PaletteShade::Strongest,
                _ => return None,
            }
        }

        Some(ThemeColor::Palette {
            family,
            shade,
            text,
        })
    }

    pub fn resolve(&self, theme: &Theme) -> Color {
        let (family, shade, text) = match *self {
            ThemeColor::Fixed(color) => return color,
            ThemeColor::Palette {
                family,
                shade,
                text,
            } => (family, shade, text),
        };

        let palette = theme.extended_palette();

        let pair: Pair = match family {
            PaletteFamily::Background => {
                let bg = &palette.background;
                match shade {
                    PaletteShade::Base => bg.base,
                    PaletteShade::Weakest => bg.weakest,
                    PaletteShade::Weaker => bg.weaker,
                    PaletteShade::Weak => bg.weak,
                    PaletteShade::Neutral => bg.neutral,
                    PaletteShade::Strong => bg.strong,
                    PaletteShade::Stronger => bg.stronger,
                    PaletteShade::Strongest => bg.strongest,
                }
            }
            _ => {
                let (base, weak, strong) = match family {
                    PaletteFamily::Primary => {
                        let p = &palette.primary;
                        (p.base, p.weak, p.strong)
                    }
                    PaletteFamily::Secondary => {
                        let p = &palette.secondary;
                        (p.base, p.weak, p.strong)
                    }
                    PaletteFamily::Success => {
                        let p = &palette.success;
                        (p.base, p.weak, p.strong)
                    }
                    PaletteFamily::Warning => {
                        let p = &palette.warning;
                        (p.base, p.weak, p.strong)
                    }
                    PaletteFamily::Danger => {
                        let p = &palette.danger;
                        (p.base, p.weak, p.strong)
                    }
                    PaletteFamily::Background => unreachable!(),
                };
                match shade {
                    PaletteShade::Weak => weak,
                    PaletteShade::Strong => strong,
                    _ => base,
                }
            }
        };

        if text { pair.text } else { pair.color }
    }
}

impl JsProp for ThemeColor {
    fn from_prop(value: rquickjs::Value<'_>) -> rquickjs::Result<Self> {
        if let Ok(v) = value.get::<String>()
            && let Some(color) = ThemeColor::parse_token(&v)
        {
            return Ok(color);
        }

        Ok(ThemeColor::Fixed(Color::from_prop(value)?))
    }
}

/// A container style whose colors are resolved against the theme at draw time.
#[derive(Debug, Clone, Copy)]
pub struct CustomViewStyle {
    pub background: Option<ThemeColor>,
    pub text_color: Option<ThemeColor>,
    pub border_color: ThemeColor,
    pub border_width: f32,
    pub border_radius: iced::border::Radius,
    pub shadow_color: ThemeColor,
    pub shadow_blur_radius: f32,
    pub shadow_offset: Vector,
    pub snap: bool,
}

impl CustomViewStyle {
    pub fn to_style(&self, theme: &Theme) -> iced::widget::container::Style {
        iced::widget::container::Style {
            background: self.background.map(|c| c.resolve(theme).into()),
            text_color: self.text_color.map(|c| c.resolve(theme)),
            border: Border {
                color: self.border_color.resolve(theme),
                width: self.border_width,
                radius: self.border_radius,
            },
            shadow: Shadow {
                color: self.shadow_color.resolve(theme),
                blur_radius: self.shadow_blur_radius,
                offset: self.shadow_offset,
            },
            snap: self.snap,
        }
    }
}

#[derive(Debug)]
pub enum ViewStyle {
    RoundedBox,
    Custom(CustomViewStyle),
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
            .get::<_, Prop<ThemeColor>>("background")
            .ok()
            .map(to_inner);
        let text_color = obj.get::<_, Prop<ThemeColor>>("color").ok().map(to_inner);

        let border_width = obj.get::<_, f32>("borderWidth").ok().unwrap_or_default();
        let border_color = obj
            .get::<_, Prop<ThemeColor>>("borderColor")
            .ok()
            .map(to_inner)
            .unwrap_or_default();
        let border_radius = obj
            .get::<_, Prop<iced::border::Radius>>("borderRadius")
            .map(to_inner)
            .unwrap_or_default();

        let shadow_color = obj
            .get::<_, Prop<ThemeColor>>("shadowColor")
            .ok()
            .map(to_inner)
            .unwrap_or_default();
        let shadow_blur_radius = obj
            .get::<_, f32>("shadowBlurRadius")
            .ok()
            .unwrap_or_default();

        let offset_y = obj.get::<_, f32>("shadowOffsetY").ok().unwrap_or_default();
        let offset_x = obj.get::<_, f32>("shadowOffsetX").ok().unwrap_or_default();

        let snap = obj.get::<_, bool>("snap").ok().unwrap_or_default();

        Ok(ViewStyle::Custom(CustomViewStyle {
            background,
            text_color,
            border_color,
            border_width,
            border_radius,
            shadow_color,
            shadow_blur_radius,
            shadow_offset: Vector::new(offset_x, offset_y),
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

impl JsProp for ButtonStyle {
    fn from_prop(value: rquickjs::Value<'_>) -> rquickjs::Result<Self> {
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

impl JsProp for SvgStyle {
    fn from_prop(value: rquickjs::Value<'_>) -> rquickjs::Result<Self> {
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
