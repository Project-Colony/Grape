pub mod app;
pub mod components;
pub mod i18n;
pub mod message;
pub mod state;
pub mod style;

pub use app::GrapeApp;

pub fn run(
    catalog: crate::library::Catalog,
    library_root_override: Option<std::path::PathBuf>,
) -> iced::Result {
    GrapeApp::run(catalog, library_root_override)
}
