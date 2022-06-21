use proto_salsa_compiler::ContentAddr;

use crate::inputs::Inputs;

#[salsa::query_group(PackageStorage)]
pub trait Package: Inputs {
    // European countries
    fn package_see_ps5(&self) -> ContentAddr;
    // Asian countries
    fn package_sea_ps5(&self) -> ContentAddr;
}

pub fn package_see_ps5(_db: &dyn Package) -> ContentAddr {
    /*
        let languages = vec![Locale::English, Locale::French, Locale::Spanish];
        // Would be Vec<ResourcePathId>
        let content_to_package = vec!["MyWorld.entity"]; // Include only content to be packaged in this build

        for language in languages {
            for content in &content_to_package {}
        }
    */
    ContentAddr(0)
}

pub fn package_sea_ps5(_db: &dyn Package) -> ContentAddr {
    //let languages = vec![Locale::English, Locale::Japenese];

    ContentAddr(0)
}
