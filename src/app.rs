#[cfg(feature = "cli")]
pub mod display;
#[cfg(feature = "cli")]
pub mod handler;

#[cfg(feature = "cli")]
pub use handler::App;
