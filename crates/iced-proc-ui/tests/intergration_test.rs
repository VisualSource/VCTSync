use iced_xml::ui;

macro_rules! asset {
    ($name:literal) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/", $name)
    };
}

#[derive(Debug, Clone)]
enum TestEnum {
    Hello,
}

#[test]
fn test_parse() {
    let _result: iced::Element<'_, TestEnum> = ui! {
        <row spacing=4>
            <svg src={asset!("network.svg")} width={50} height={50}/>
            <col>
                Hello
            </col>
            <col>
                <button onPress={TestEnum::Hello}>Hello</button>
            </col>
        </row>
    };
}
