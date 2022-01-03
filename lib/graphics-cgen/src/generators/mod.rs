mod file_writer;
pub mod hlsl;
pub mod product;
pub mod rust;

use std::collections::HashSet;

use heck::ToSnakeCase;
use relative_path::{RelativePath, RelativePathBuf};

use self::product::Product;
use crate::{
    model::{CGenType, CGenTypeRef, DescriptorSet, Model, ModelObject},
    run::CGenVariant,
};

pub type GeneratorFunc = for<'r, 's> fn(&'r GeneratorContext<'s>) -> Vec<Product>;
pub struct GeneratorContext<'a> {
    model: &'a Model,
}

impl<'a> GeneratorContext<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }

    fn get_file_ext(cgen_variant: CGenVariant) -> &'static str {
        match cgen_variant {
            CGenVariant::Hlsl => "hlsl",
            CGenVariant::Rust => "rs",
            CGenVariant::Blob => "blob",
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
        file_name.set_extension(Self::get_file_ext(cgen_variant));
        file_name
    }

    pub fn get_type_dependencies(ty: &CGenType) -> HashSet<CGenTypeRef> {
        let mut set = HashSet::new();

        match ty {
            CGenType::Native(_) => {}
            CGenType::Struct(inner_ty) => {
                for mb in &inner_ty.members {
                    set.insert(mb.ty_ref);
                }
            }
        }

        set
    }

    pub fn get_descriptorset_dependencies(ty: &DescriptorSet) -> HashSet<CGenTypeRef> {
        let mut set = HashSet::new();

        for descriptor in &ty.descriptors {
            match &descriptor.def {
                crate::model::DescriptorDef::ConstantBuffer(def) => {
                    set.insert(def.ty_ref);
                }
                crate::model::DescriptorDef::StructuredBuffer(def)
                | crate::model::DescriptorDef::RWStructuredBuffer(def) => {
                    set.insert(def.ty_ref);
                }
                crate::model::DescriptorDef::Sampler
                | crate::model::DescriptorDef::ByteAddressBuffer
                | crate::model::DescriptorDef::RWByteAddressBuffer
                | crate::model::DescriptorDef::Texture2D(_)
                | crate::model::DescriptorDef::RWTexture2D(_)
                | crate::model::DescriptorDef::Texture3D(_)
                | crate::model::DescriptorDef::RWTexture3D(_)
                | crate::model::DescriptorDef::Texture2DArray(_)
                | crate::model::DescriptorDef::RWTexture2DArray(_)
                | crate::model::DescriptorDef::TextureCube(_)
                | crate::model::DescriptorDef::TextureCubeArray(_) => (),
            }
        }

        set
    }

    
}
