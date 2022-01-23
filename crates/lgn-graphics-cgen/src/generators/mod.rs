mod file_writer;
pub mod hlsl;
pub mod product;
pub mod rust;

use heck::ToSnakeCase;
use relative_path::{RelativePath, RelativePathBuf};

use self::product::Product;
use crate::{
    db::{Model, ModelObject},
    run::CGenVariant,
    struct_layout::StructLayouts,
};

pub type GeneratorFunc = for<'r, 's> fn(&'r GeneratorContext<'s>) -> Vec<Product>;
pub struct GeneratorContext<'a> {
    crate_name: String,
    model: &'a Model,
    struct_layouts: StructLayouts,
}

impl<'a> GeneratorContext<'a> {
    pub fn new(crate_name: &str, model: &'a Model) -> Self {
        Self {
            crate_name: crate_name.to_owned(),
            model,
            struct_layouts: hlsl::struct_layouts_builder::run(model).unwrap(),
        }
    }

    fn object_folder<T>() -> RelativePathBuf
    where
        T: ModelObject,
    {
        RelativePath::new(&T::typename().to_snake_case()).to_owned()
    }

    fn object_relative_path<T>(typ: &T, cgen_variant: CGenVariant) -> RelativePathBuf
    where
        T: ModelObject,
    {
        let file_name = Self::object_filename(typ, cgen_variant);
        Self::object_folder::<T>().join(file_name)
    }

    fn object_filename<T>(obj: &T, cgen_variant: CGenVariant) -> RelativePathBuf
    where
        T: ModelObject,
    {
        let mut file_name = RelativePath::new(&obj.name().to_snake_case()).to_relative_path_buf();
        file_name.set_extension(cgen_variant.get_file_ext());
        file_name
    }

    fn embedded_fs_path<T>(&self, obj: &T, cgen_variant: CGenVariant) -> String
    where
        T: ModelObject,
    {
        format!(
            "crate://{}/codegen/{}/{}/{}.{}",
            &self.crate_name,
            cgen_variant.dir(),
            T::typename().to_snake_case(),
            &obj.name().to_snake_case(),
            cgen_variant.get_file_ext()
        )
    }
}
