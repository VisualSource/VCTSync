mod fs_loader;
mod system_modules;

#[cfg(feature = "svg-element")]
pub(crate) mod svg;

#[cfg(feature = "embed")]
mod slio_loader;

pub use fs_loader::FsAssets;
pub(crate) use system_modules::SystemModule;

use rquickjs::loader::ImportAttributes;
#[cfg(feature = "embed")]
pub use slio_loader::SiloAssets;

pub fn is_svg(path: &str, attrs: &Option<ImportAttributes>) -> bool {
    if let Some(attrs) = attrs
        && let Ok(Some(t)) = attrs.get_type()
    {
        return t == "svg";
    }

    path.ends_with(".svg")
}
