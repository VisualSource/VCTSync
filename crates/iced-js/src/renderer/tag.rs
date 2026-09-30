use super::props::{self, ButtonProps, CommonProps, SpaceProps, TextProps, ViewProps};
use std::fmt::Display;

#[derive(Debug)]
pub enum Tag {
    Col(CommonProps),
    Row(CommonProps),
    View(ViewProps),
    Button(ButtonProps),
    Text(TextProps),
    Scroll,
    Space(SpaceProps),
    Hr,
    Vr,
    Tooltip,

    #[cfg(feature = "svg-element")]
    Svg(props::SvgProps),
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tag::Col(_) => write!(f, "col"),
            Tag::Row(_) => write!(f, "row"),
            Tag::View(_) => write!(f, "view"),
            Tag::Button(_) => write!(f, "button"),
            Tag::Text(_) => write!(f, "text"),

            Self::Scroll => write!(f, "scroll"),
            Self::Space(_) => write!(f, "space"),
            Self::Hr => write!(f, "hr"),
            Self::Vr => write!(f, "vr"),
            Self::Tooltip => write!(f, "tooltip"),

            #[cfg(feature = "svg-element")]
            Tag::Svg(_) => write!(f, "svg"),
        }
    }
}

impl Tag {
    pub fn valid_child_count(&self, len: usize) -> bool {
        match self {
            Tag::Space(_) | Tag::Hr | Tag::Vr => len == 0,
            Tag::View(_) | Tag::Button(_) | Tag::Text(_) | Tag::Scroll => len == 1,
            Tag::Tooltip => len == 2,
            _ => true,
        }
    }
}
