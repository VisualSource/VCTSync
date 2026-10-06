#[derive(Clone)]
pub enum Message {
    Js(iced_js::Event),
    Reload,
}
