use iced_xml::ui;

macro_rules! asset {
    ($name:literal) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/", $name)
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RadioValue {
    A,
}

#[derive(Debug, Clone)]
enum TestEnum {
    Hello,
    OnTextChange(String),
    OnRadioChange(RadioValue),
    OnSliderChange(i32),
}

#[test]
fn test_parse() {
    let text_input_value = String::new();
    let checkbox_value = false;
    let selected_radio_value = Some(RadioValue::A);
    let slider_range_value = 100;

    let _result: iced::Element<'_, TestEnum> = ui! {
        <view>

            <row spacing=4>
                <svg src={asset!("network.svg")} width={50} height={50}/>
                <!-- -->
                <scroll>
                    <col>
                        Hello
                    </col>
                </scroll>
                <col>
                    <hr/>
                    <input type="range" min={0} max={100} value={slider_range_value} onChange={TestEnum::OnSliderChange} orient="vertical"/>
                    <input type="range" min={0} max={100} value={slider_range_value} onChange={TestEnum::OnSliderChange}/>
                    <input type="radio" label="A" value={RadioValue::A} selected={selected_radio_value} onChange={TestEnum::OnRadioChange}/>
                    <input type="checkbox" checked={checkbox_value} />
                    <input type="text" placeholder="Type Something here..." value={&text_input_value} onChange={TestEnum::OnTextChange} />
                    <button onPress={TestEnum::Hello}>Hello</button>
                </col>
            </row>
        </view>
    };
}
