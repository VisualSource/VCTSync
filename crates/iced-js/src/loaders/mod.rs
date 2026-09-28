mod fs_loader;
#[cfg(feature = "embed")]
mod slio_loader;

pub use fs_loader::FsAssets;

#[cfg(feature = "embed")]
pub use slio_loader::SiloAssets;
