#[cfg(target_os="android")]
pub mod assets_android;
#[cfg(target_os="android")]
pub use support::assets_android as assets;

#[cfg(not(target_os="android"))]
pub mod assets;
