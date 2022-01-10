mod file_writer;
pub mod hlsl;
pub mod product;
pub mod rust;

use heck::ToSnakeCase;
use relative_path::{RelativePath, RelativePathBuf};

use self::product::Product;
use crate::{
    model::{Model, ModelObject},
    run::CGenVariant,
    struct_layout::StructLayouts,
};

pub type GeneratorFunc = for<'r, 's> fn(&'r GeneratorContext<'s>) -> Vec<Product>;
pub struct GeneratorContext<'a> {
    model: &'a Model,
    struct_layouts: StructLayouts,
}

impl<'a> GeneratorContext<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self {
            model,
            struct_layouts: hlsl::struct_layouts_builder::run(model).unwrap(),
        }
    }

    fn get_object_folder<T>() -> RelativePathBuf
    where
        T: ModelObject,
    {
        RelativePath::new(&T::typename().to_snake_case()).to_owned()
    }

    fn get_object_rel_path<T>(ty: &T, cgen_variant: CGenVariant) -> RelativePathBuf
    where
        T: ModelObject,
    {
        let filename = Self::get_object_filename(ty, cgen_variant);
        Self::get_object_folder::<T>().join(filename)
    }

    fn get_object_filename<T>(obj: &T, cgen_variant: CGenVariant) -> RelativePathBuf
    where
        T: ModelObject,
    {
        let mut file_name = RelativePath::new(&obj.name().to_snake_case()).to_relative_path_buf();
        file_name.set_extension(cgen_variant.get_file_ext());
        file_name
    }
}
