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
#[derive(Debug, Clone, Copy, Default)]
pub struct CustomViewStyle {
    pub background: Option<ThemeColor>,
    pub text_color: Option<ThemeColor>,
    /// Multiplies the alpha of every color in the style; `None` leaves them as is.
    pub opacity: Option<f32>,
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
            background: self.background.map(|c| self.fade(c.resolve(theme)).into()),
            text_color: self.text_color.map(|c| self.fade(c.resolve(theme))),
            border: Border {
                color: self.fade(self.border_color.resolve(theme)),
                width: self.border_width,
                radius: self.border_radius,
            },
            shadow: Shadow {
                color: self.fade(self.shadow_color.resolve(theme)),
                blur_radius: self.shadow_blur_radius,
                offset: self.shadow_offset,
            },
            snap: self.snap,
        }
    }

    fn fade(&self, color: Color) -> Color {
        color.scale_alpha(self.opacity.unwrap_or(1.0))
    }

    /// Reads `prefix`ed keys (e.g. `hoverBackground`), falling back to `base` for any that are missing.
    fn parse(obj: &rquickjs::Object<'_>, prefix: &str, base: CustomViewStyle) -> CustomViewStyle {
        let key = |name: &str| {
            if prefix.is_empty() {
                return name.to_string();
            }
            let mut chars = name.chars();
            let first = chars
                .next()
                .map(|c| c.to_ascii_uppercase())
                .unwrap_or_default();
            format!("{prefix}{first}{}", chars.as_str())
        };
        let color = |name: &str| obj.get::<_, Prop<ThemeColor>>(key(name)).ok().map(to_inner);
        let float = |name: &str| obj.get::<_, f32>(key(name)).ok();

        CustomViewStyle {
            background: color("background").or(base.background),
            text_color: color("color").or(base.text_color),
            opacity: float("opacity").or(base.opacity),
            border_color: color("borderColor").unwrap_or(base.border_color),
            border_width: float("borderWidth").unwrap_or(base.border_width),
            border_radius: obj
                .get::<_, Prop<iced::border::Radius>>(key("borderRadius"))
                .map(to_inner)
                .unwrap_or(base.border_radius),
            shadow_color: color("shadowColor").unwrap_or(base.shadow_color),
            shadow_blur_radius: float("shadowBlurRadius").unwrap_or(base.shadow_blur_radius),
            shadow_offset: Vector::new(
                float("shadowOffsetX").unwrap_or(base.shadow_offset.x),
                float("shadowOffsetY").unwrap_or(base.shadow_offset.y),
            ),
            snap: obj.get::<_, bool>(key("snap")).unwrap_or(base.snap),
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

        Ok(ViewStyle::Custom(CustomViewStyle::parse(
            &obj,
            "",
            CustomViewStyle::default(),
        )))
    }
}

/// A button style where `hover*`, `pressed*` and `disabled*` keys override the active style.
#[derive(Debug, Clone, Copy)]
pub struct CustomButtonStyle {
    active: CustomViewStyle,
    hovered: CustomViewStyle,
    pressed: CustomViewStyle,
    disabled: CustomViewStyle,
}

impl CustomButtonStyle {
    pub fn to_style(
        &self,
        theme: &Theme,
        status: iced::widget::button::Status,
    ) -> iced::widget::button::Style {
        use iced::widget::button::{Status, Style};

        let view = match status {
            Status::Active => &self.active,
            Status::Hovered => &self.hovered,
            Status::Pressed => &self.pressed,
            Status::Disabled => &self.disabled,
        };
        let style = view.to_style(theme);

        Style {
            background: style.background,
            text_color: style
                .text_color
                .unwrap_or_else(|| view.fade(theme.palette().text)),
            border: style.border,
            shadow: style.shadow,
            snap: style.snap,
        }
    }
}

impl JsProp for CustomButtonStyle {
    fn from_prop(value: rquickjs::Value<'_>) -> rquickjs::Result<Self> {
        let obj = value.get::<rquickjs::Object<'_>>()?;
        let active = CustomViewStyle::parse(&obj, "", CustomViewStyle::default());

        Ok(CustomButtonStyle {
            active,
            hovered: CustomViewStyle::parse(&obj, "hover", active),
            pressed: CustomViewStyle::parse(&obj, "pressed", active),
            disabled: CustomViewStyle::parse(&obj, "disabled", active),
        })
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
    Custom(CustomButtonStyle),
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

        let custom = CustomButtonStyle::from_prop(value)?;

        Ok(ButtonStyle::Custom(custom))
    }
}
