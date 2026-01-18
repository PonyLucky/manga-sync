use utoipa::{OpenApi, Modify, openapi::security::{SecurityScheme, HttpAuthScheme, HttpBuilder}};
use crate::handlers;
use crate::models;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Manga Manager API",
        description = "A self-hosted REST API to manage manga reading progress.",
        version = "1.0.0"
    ),
    servers(
        (url = "http://localhost:7783", description = "Local development server")
    ),
    paths(
        handlers::manga::list_manga,
        handlers::manga::get_manga,
        handlers::manga::get_manga_sources,
        handlers::manga::get_manga_history,
        handlers::manga::create_manga,
        handlers::manga::update_manga,
        handlers::manga::delete_manga,
        handlers::manga::delete_manga_source,
        handlers::website::list_websites,
        handlers::website::check_website,
        handlers::website::create_website,
        handlers::website::delete_website,
        handlers::source::list_sources,
        handlers::setting::list_settings,
        handlers::setting::update_setting,
    ),
    components(
        schemas(
            models::Manga,
            models::Website,
            models::Source,
            models::Chapter,
            models::Setting,
            handlers::manga::Pagination,
            handlers::manga::MangaListItem,
            handlers::manga::MangaDetail,
            handlers::manga::MangaSource,
            handlers::manga::HistoryItem,
            handlers::manga::CreateManga,
            handlers::manga::UpdateManga,
            handlers::website::Existence,
        )
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        )
    }
}
