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
            _ => Err(rquickjs::Error::new_loading(name)),
        }
    }
}
