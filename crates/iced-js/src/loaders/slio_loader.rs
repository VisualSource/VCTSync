use rquickjs::{
    Error, Module,
    loader::{Loader, Resolver},
};
use rust_silos::Silo;
use std::{io::Read, ptr};

#[derive(Clone, Copy)]
pub struct SiloAssets {
    slio: &'static Silo,
}

impl SiloAssets {
    pub fn new(slio: &'static Silo) -> Self {
        Self { slio }
    }
}

impl std::hash::Hash for SiloAssets {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ptr::hash(self.slio, state);
    }
}

fn normalize_path(base: &str, name: &str) -> Option<String> {
    let mut parts = if name.starts_with("./") || name.starts_with("../") {
        base.rsplit_once('/')
            .map(|(dir, _)| dir.split('/').collect::<Vec<&str>>())
            .unwrap_or_default()
    } else {
        Vec::default()
    };

    for comp in name.split('/') {
        match comp {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            c => parts.push(c),
        }
    }

    Some(parts.join("/"))
}

impl Resolver for SiloAssets {
    fn resolve<'js>(
        &mut self,
        _ctx: &rquickjs::prelude::Ctx<'js>,
        base: &str, // quickjs module name
        name: &str, // es
        attributes: Option<rquickjs::loader::ImportAttributes<'js>>,
    ) -> rquickjs::Result<String> {
        let Some(import_path) = normalize_path(base, name) else {
            return Err(Error::new_resolving_message(
                base,
                name,
                "failed to resolve path",
            ));
        };

        let file_type = if let Some(attrs) = attributes {
            attrs.get_type()?
        } else {
            None
        }
        .unwrap_or_else(|| "javascript".to_string());

        if file_type != "javascript" && file_type != "json" && file_type != "svg" {
            return Err(Error::new_resolving_message(
                base,
                name,
                "unsupported loader type",
            ));
        }

        #[cfg(not(feature = "svg-element"))]
        if file_type == "svg" {
            return Err(Error::new_resolving_message(
                base,
                name,
                "unable load to svg as it's not enabled",
            ));
        }

        let Some(file) = self.slio.get_file(&import_path) else {
            return Err(Error::new_resolving_message(
                base,
                name,
                "failed to find given file",
            ));
        };

        let path = file.path().to_string_lossy().to_string();

        Ok(path)
    }
}

impl Loader for SiloAssets {
    fn load<'js>(
        &mut self,
        ctx: &rquickjs::prelude::Ctx<'js>,
        path: &str,
        attributes: Option<rquickjs::loader::ImportAttributes<'js>>,
    ) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::module::Declared>> {
        let Some(file) = self.slio.get_file(path) else {
            return Err(Error::new_loading_message(
                path,
                "no such file at given location",
            ));
        };

        let mut reader = file
            .reader()
            .map_err(|err| Error::new_loading_message(path, err.to_string()))?;

        let mut source = Vec::new();
        reader
            .read_to_end(&mut source)
            .map_err(|err| Error::new_loading_message(path, err.to_string()))?;

        #[cfg(feature = "svg-element")]
        if crate::loaders::is_svg(path, &attributes) {
            let handle = iced::widget::svg::Handle::from_memory(source);

            return super::svg::declare_svg_module(ctx, path, handle);
        }

        Module::declare(ctx.clone(), path, source)
    }
}
