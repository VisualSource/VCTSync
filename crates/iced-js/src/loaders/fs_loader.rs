use rquickjs::{
    Error, Module,
    loader::{Loader, Resolver},
};
use std::{path::PathBuf, str::FromStr};

#[derive(Debug, Clone, Hash, Default)]
pub struct FsAssets {
    root: PathBuf,
}

impl FsAssets {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

impl Loader for FsAssets {
    fn load<'js>(
        &mut self,
        ctx: &rquickjs::prelude::Ctx<'js>,
        path: &str,
        attributes: Option<rquickjs::loader::ImportAttributes<'js>>,
    ) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::module::Declared>> {
        #[cfg(feature = "svg-element")]
        if crate::loaders::is_svg(path, &attributes) {
            let handle = iced::widget::svg::Handle::from_path(path);

            return super::svg::declare_svg_module(ctx, path, handle);
        }
        #[cfg(not(feature = "svg-element"))]
        let _ = &attributes;

        let source: Vec<_> = std::fs::read(path)?;
        Module::declare(ctx.clone(), path, source)
    }
}

impl Resolver for FsAssets {
    fn resolve<'js>(
        &mut self,
        _ctx: &rquickjs::prelude::Ctx<'js>,
        module_name: &str, // quickjs module name
        import_path: &str, // esm import path
        _attributes: Option<rquickjs::loader::ImportAttributes<'js>>,
    ) -> rquickjs::Result<String> {
        if self.root.is_empty() {
            return Err(Error::new_resolving_message(
                module_name,
                import_path,
                "no asset dir was configured",
            ));
        }

        let path = PathBuf::from_str(import_path).map_err(|_| {
            Error::new_resolving_message(module_name, import_path, "failed to create path buff")
        })?;

        let mut asset_path = self.root.clone();
        for comp in path.components() {
            use std::path::Component;

            match comp {
                Component::RootDir | Component::Prefix(_) => {
                    return Err(Error::new_resolving(module_name, import_path));
                }
                Component::CurDir => {} // . part in a path like ./
                Component::ParentDir => {
                    asset_path.pop();

                    if !asset_path.starts_with(&self.root) {
                        return Err(Error::new_resolving(module_name, import_path));
                    }

                    if asset_path.exists() {
                        if !asset_path.starts_with(&self.root) {
                            return Err(Error::new_resolving(module_name, import_path));
                        }
                    } else if asset_path.is_symlink() {
                        return Err(Error::new_resolving(module_name, import_path));
                    }
                }
                Component::Normal(os_str) => {
                    asset_path.push(os_str);

                    if asset_path.exists() {
                        if !asset_path.starts_with(&self.root) {
                            return Err(Error::new_resolving(module_name, import_path));
                        }
                    } else if asset_path.is_symlink() {
                        return Err(Error::new_resolving(module_name, import_path));
                    }
                }
            }
        }

        Ok(asset_path.to_string_lossy().to_string())
    }
}
