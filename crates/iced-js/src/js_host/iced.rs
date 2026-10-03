#[rquickjs::module]
pub mod iced_module {

    #[derive(rquickjs::class::Trace, rquickjs::JsLifetime)]
    #[rquickjs::class()]
    pub struct Font {}

    #[derive(rquickjs::class::Trace, rquickjs::JsLifetime)]
    #[rquickjs::class()]
    pub struct Color {}
}
