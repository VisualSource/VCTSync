use iced_ui::ui;

#[derive(Debug, Clone)]
enum TestEnum {
    Hello,
}

#[test]
fn test_parse() {
    let result = ui! {
        <row space=4>
            <col>
                Hello
            </col>
            <col>
                <button onPress=TestEnum::Hello>Hello</button>
            </col>
        </row>
    };
}
