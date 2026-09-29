use iced::widget::svg::Handle;
use rquickjs::{
    Class, Ctx, JsLifetime, Module,
    class::Trace,
    module::{Declarations, Declared, Exports, ModuleDef},
};
use std::{cell::RefCell, collections::HashMap};

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct SvgHandle {
    #[qjs(skip_trace)]
    pub(crate) handle: Handle,
}

#[rquickjs::methods]
impl<'js> SvgHandle {
    #[qjs(constructor)]
    fn constructor(ctx: Ctx<'js>) -> rquickjs::Result<()> {
        Err(rquickjs::Exception::throw_syntax(
            &ctx,
            "manual instancing is not allowed",
        ))
    }
}

impl SvgHandle {
    fn new(handle: Handle) -> Self {
        Self { handle }
    }
}

#[derive(Default, JsLifetime)]
struct SvgAssets(RefCell<HashMap<String, Handle>>);

struct NativeSvgModule;

impl ModuleDef for NativeSvgModule {
    fn declare(decl: &Declarations) -> rquickjs::Result<()> {
        decl.declare("default")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> rquickjs::Result<()> {
        let path: String = exports.module().name()?;

        let handle = ctx
            .userdata::<SvgAssets>()
            .and_then(|assets| assets.0.borrow_mut().remove(&path))
            .ok_or_else(|| {
                rquickjs::Error::new_loading_message(&path, "no pending svg handle for this module")
            })?;

        exports.export(
            "default",
            Class::instance(ctx.clone(), SvgHandle::new(handle))?,
        )?;

        Ok(())
    }
}

pub fn init<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
    let globals = ctx.globals();

    Class::<SvgHandle>::define(&globals)?;

    Ok(())
}

pub fn declare_svg_module<'js>(
    ctx: &Ctx<'js>,
    path: &str,
    handle: Handle,
) -> rquickjs::Result<Module<'js, Declared>> {
    if ctx.userdata::<SvgAssets>().is_none() {
        // The error case is "userdata is currently borrowed", which cannot
        // happen on this path.
        let _ = ctx.store_userdata(SvgAssets::default());
    }

    ctx.userdata::<SvgAssets>()
        .expect("svg asset store was just installed")
        .0
        .borrow_mut()
        .insert(path.to_owned(), handle);

    Module::declare_def::<NativeSvgModule, _>(ctx.clone(), path)
}
