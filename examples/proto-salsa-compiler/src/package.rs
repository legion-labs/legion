use proto_salsa_compiler::{BuildParams, ContentAddr, Locale, Platform, Target};

use crate::{entity::EntityCompiler, inputs::Inputs};

#[salsa::query_group(PackageStorage)]
pub trait PackageCompiler: Inputs + EntityCompiler {
    // European countries
    fn package_see_ps5(&self) -> ContentAddr;
    // Asian countries
    fn package_sea_ps5(&self) -> ContentAddr;

    fn package(&self, languages: Vec<Locale>, content_to_package: Vec<String>) -> ContentAddr;
}

fn package(
    db: &dyn PackageCompiler,
    languages: Vec<Locale>,
    content_to_package: Vec<String>,
) -> ContentAddr {
    let mut all_content = String::new();

    for language in languages {
        let build_params = BuildParams::new(Platform::PS5, Target::Client, language);
        for content in &content_to_package {
            all_content.push_str(
                db.compile_entity(content.to_string(), build_params.clone())
                    .as_str(),
            );
        }
    }

    // Would return the content address for the whole package
    ContentAddr(0)
}

pub fn package_see_ps5(db: &dyn PackageCompiler) -> ContentAddr {
    println!("Package SEE PS5");
    let languages = vec![Locale::English, Locale::French];
    // Include only content to be packaged in this build
    let content_to_package = vec!["MyWorld.entity".to_string()];

    db.package(languages, content_to_package);

    ContentAddr(0)
}

pub fn package_sea_ps5(db: &dyn PackageCompiler) -> ContentAddr {
    println!("Package SEA PS5");
    let languages = vec![Locale::English, Locale::Japenese];
    // Include only content to be packaged in this build
    let content_to_package = vec!["MyWorld.entity".to_string()];

    db.package(languages, content_to_package);

    ContentAddr(0)
}
