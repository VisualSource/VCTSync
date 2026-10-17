use crate::js_host;

#[derive(Default)]
pub struct SystemModule;

impl rquickjs::loader::Resolver for SystemModule {
    fn resolve<'js>(
        &mut self,
        _ctx: &rquickjs::prelude::Ctx<'js>,
        base: &str,
        name: &str,
        _attributes: Option<rquickjs::loader::ImportAttributes<'js>>,
    ) -> rquickjs::Result<String> {
        match name {
            "iced" => Ok("iced".to_string()),
            #[cfg(feature = "js-compression")]
            "iced:compression" => Ok("iced:compression".to_string()),
            _ => Err(rquickjs::Error::new_resolving(base, name)),
        }
    }
}

impl rquickjs::loader::Loader for SystemModule {
    fn load<'js>(
        &mut self,
        ctx: &rquickjs::prelude::Ctx<'js>,
        name: &str,
        _attributes: Option<rquickjs::loader::ImportAttributes<'js>>,
    ) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::module::Declared>> {
        match name {
            "iced" => {
                return rquickjs::Module::declare_def::<js_host::iced::js_iced_module, _>(
                    ctx.clone(),
                    "iced",
                );
            }
            #[cfg(feature = "js-compression")]
            "iced:compression" => {
                return rquickjs::Module::declare_def::<js_host::compression::js_compression, _>(
                    ctx.clone(),
                    "iced:compression",
                );
            }
            _ => Err(rquickjs::Error::new_loading(name)),
        }
    }
}
