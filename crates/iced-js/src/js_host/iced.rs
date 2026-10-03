#[rquickjs::module]
pub mod iced_module {

    #[derive(rquickjs::class::Trace, rquickjs::JsLifetime)]
    #[rquickjs::class()]
    pub struct Font {}

    #[rquickjs::methods]
    impl Font {
        #[qjs(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }
    }

    #[derive(rquickjs::class::Trace, rquickjs::JsLifetime)]
    #[rquickjs::class()]
    pub struct Color {}

    #[rquickjs::methods]
    impl Color {
        #[qjs(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }
    }
}
