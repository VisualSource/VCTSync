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
}
