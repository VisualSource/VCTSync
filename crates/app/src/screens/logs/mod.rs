use iced::{
    Alignment, Color, Element, Length, Shadow, Task, Theme, Vector,
    widget::{Column, container},
};
use iced_aw::{
    DropDown,
    drop_down::{self, Offset},
};
use iced_query::{Query, QueryClient};
use iced_xml::ui;

use crate::{
    asset,
    state::Message,
    traits::IcedScreen,
    utils::{style_svg, tooltip_label},
};

mod fetcher;
pub mod state;

use state::Action;

const MENU_W: f32 = 200.0;
/// 16px text * 1.3 line-height + button::DEFAULT_PADDING (5 top/bottom).
/// Retune if the button's font size or padding changes.
const BTN_H: f32 = 31.0;

fn dd_bg(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(palette.background.stronger.color.into()),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.15), // Subtle black shadow
            offset: Vector::new(2.0, 4.0),                // Shifts shadow X and Y
            blur_radius: 8.0,                             // Softness of the shadow
        },
        snap: false,
        ..Default::default()
    }
}

#[derive(Debug)]
pub struct Screen {
    show_mod_only_lines: bool,

    menu_expand: bool,

    log_lines: Query<Vec<String>>,
}

impl Screen {
    pub fn new(client: &QueryClient) -> Self {
        Self {
            show_mod_only_lines: false,
            menu_expand: false,
            log_lines: Query::new(client, "logs::files", fetcher::fetch_log_file, None),
        }
    }
}

impl IcedScreen<state::Action> for Screen {
    fn view(&self) -> iced::Element<'_, crate::state::Message> {
        let export_menu: DropDown<'_, Message> = DropDown::new(
            ui! {
                <row>
                  <tooltip content={tooltip_label("Export")} position={iced::widget::tooltip::Position::Left }>
                    <button width={iced::Shrink} onPress={Action::Expand.into()}><svg style={style_svg} src={asset!("folder-input.svg")}/></button>
                  </tooltip>
                </row>
            },
            ui! {
                <view style={dd_bg} padding={[2,4]} width={MENU_W}>
                    <col spacing={4}>
                        <button width={iced::Fill} onPress={Action::ExportLogFile.into()}>Export To File</button>
                        <button width={iced::Fill} onPress={Action::ExportToAgent.into()}>Export To Agent</button>
                    </col>
                </view>
            },
            self.menu_expand,
        )
        .width(iced::Shrink)
        .alignment(drop_down::Alignment::BottomEnd)
        .offset(Offset::new(-MENU_W, BTN_H + 4.0))
        .on_dismiss(Action::Dismiss.into());

        let q = &self.log_lines.snapshot;
        let rv = match (&q.data, &q.error) {
            (Some(data), _) => ui! {
                  <scroll height={iced::Fill} anchorBottom>
                    {iced::Element::from(Column::with_children(
                        data.iter().map(|l| log_line(l))).spacing(2),
                    )}
                  </scroll>
            },
            (None, Some(err)) => {
                let reason = err.to_string();
                ui! {
                    <view height={Length::Fill} center={Length::Fill} width={Length::Fill}>
                        <col alignX={iced::Center} alignY={iced::Center}>
                            <text>{reason}</text>
                        </col>
                    </view>
                }
            }

            (None, None) => {
                ui! {

                    <view height={Length::Fill} center={Length::Fill} width={Length::Fill}>
                        <col alignX={Alignment::Center} alignY={Alignment::Center}>
                            Loading
                        </col>
                    </view>
                }
            }
        };

        ui! {
            <col>
                <row padding={4} alignY={Alignment::Center}>
                    <input type="checkbox" checked={self.show_mod_only_lines} label="Mod lines only" onChange={|v|Action::ToggleVCTLinesOnly(v).into()}/>
                    <space width={iced::Fill}/>
                    {Element::from(export_menu)}
                </row>
                <hr/>
                {rv}
            </col>
        }
    }

    fn update(
        &mut self,
        ev: state::Action,
        _client: &iced_query::QueryClient,
    ) -> iced::Task<crate::state::Message> {
        match ev {
            Action::ToggleVCTLinesOnly(v) => {
                self.show_mod_only_lines = v;
            }
            Action::ExportLogFile => {
                self.menu_expand = false;
            }
            Action::ExportToAgent => {
                self.menu_expand = false;
            }
            Action::Expand => self.menu_expand = !self.menu_expand,
            Action::Dismiss => {
                self.menu_expand = false;
            }
        }

        Task::none()
    }

    fn mount(&self, client: &iced_query::QueryClient) -> iced::Task<crate::state::Message> {
        Task::none()
    }

    fn sync(&mut self, client: &iced_query::QueryClient) {}
}

fn log_line<'a>(line: &'a str) -> Element<'a, Message> {
    ui! {
        <text size={12}>{line}</text>
    }
}
