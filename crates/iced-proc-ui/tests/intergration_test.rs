use iced_ui::ui;

#[test]
fn test_parse() {
    let result = ui! {
        <row>
            Hello
        </row>
    };

    println!("{:#?}", result);
}
