use crate::{compiler::Compiler, BuildParams, ContentAddr, Locale, Platform, Target};

pub fn package(
    db: &dyn Compiler,
    languages: Vec<Locale>,
    content_to_package: Vec<String>,
) -> ContentAddr {
    for language in languages {
        let build_params = BuildParams::new(Platform::PS5, Target::Client, language);
        for content in &content_to_package {
            db.run(content.clone(), build_params.clone());
        }
    }

    // Would return the content address for the whole package
    ContentAddr(0)
}

pub fn package_see_ps5(db: &dyn Compiler) -> ContentAddr {
    println!("Package SEE PS5");
    let languages = vec![Locale::English, Locale::French];
    // Include only content to be packaged in this build
    let content_to_package = vec!["MyWorld.entity".to_string()];

    db.package(languages, content_to_package);

    ContentAddr(0)
}

pub fn package_sea_ps5(db: &dyn Compiler) -> ContentAddr {
    println!("Package SEA PS5");
    let languages = vec![Locale::English, Locale::Japenese];
    // Include only content to be packaged in this build
    let content_to_package = vec!["MyWorld.entity".to_string()];

    db.package(languages, content_to_package);

    ContentAddr(0)
}
