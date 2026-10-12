use iced::{
    Element, Theme,
    widget::svg::{self, Status},
};

use crate::state::Message;

pub fn style_svg(theme: &Theme, _status: Status) -> svg::Style {
    let pal = theme.palette();

    svg::Style {
        color: Some(pal.text),
    }
}

pub fn tooltip_label(text: &str) -> Element<'_, Message> {
    iced_xml::ui! {
        <view padding={[4,8]} style={iced::widget::container::rounded_box}>
            <text>{text}</text>
        </view>
    }
}
