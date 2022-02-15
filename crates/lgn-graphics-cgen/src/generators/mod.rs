mod file_writer;
pub mod hlsl;
pub mod product;
pub mod rust;
use lgn_utils::DefaultHasher;
use std::hash::{Hash, Hasher};

use heck::ToSnakeCase;

use self::product::Product;
use crate::{
    db::{Model, ModelObject},
    run::CGenVariant,
    struct_layout::StructLayouts,
};

pub type GeneratorFunc = for<'r, 's> fn(&'r GeneratorContext<'s>) -> Vec<Product>;
pub struct GeneratorContext<'a> {
    crate_name: String,
    crate_id: u64,
    model: &'a Model,
    struct_layouts: StructLayouts,
}

impl<'a> GeneratorContext<'a> {
    pub fn new(crate_name: &str, model: &'a Model) -> Self {
        let mut hasher = DefaultHasher::new();
        crate_name.hash(&mut hasher);
        let crate_id = hasher.finish();

        Self {
            crate_name: crate_name.to_owned(),
            crate_id,
            model,
            struct_layouts: hlsl::struct_layouts_builder::run(model).unwrap(),
        }
    }

    fn object_folder<T>() -> String
    where
        T: ModelObject,
    {
        T::typename().to_snake_case()
    }

    fn object_relative_path<T>(obj: &T, cgen_variant: CGenVariant) -> String
    where
        T: ModelObject,
    {
        format!(
            "{}/{}.{}",
            Self::object_folder::<T>(),
            obj.name().to_snake_case(),
            cgen_variant.get_file_ext()
        )
    }

    fn embedded_fs_path<T>(&self, obj: &T, cgen_variant: CGenVariant) -> String
    where
        T: ModelObject,
    {
        // convention here is to use the gpu folder
        format!(
            "crate://{}/gpu/{}/{}.{}",
            &self.crate_name,
            T::typename().to_snake_case(),
            &obj.name().to_snake_case(),
            cgen_variant.get_file_ext()
        )
    }
}
