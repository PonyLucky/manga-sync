use utoipa::OpenApi;
use manga_sync::openapi::ApiDoc;

fn main() {
    let yaml = ApiDoc::openapi().to_yaml().expect("Failed to generate YAML");
    println!("{}", yaml);
}
