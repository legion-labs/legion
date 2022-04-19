use std::{cell::RefCell, str::FromStr};

use crate::{
    offline::{Material, Mesh, Model, SamplerData},
    runtime::RawTexture,
    Color, Filter, TextureType, WrappingMode,
};
use gltf::{
    image::Format,
    material::NormalTexture,
    mesh::util::{ReadIndices, ReadTexCoords},
    texture, Document,
};
use lgn_data_model::ReflectionError;
use lgn_math::{Vec2, Vec3, Vec4};

use lgn_data_runtime::prelude::*;
use lgn_tracing::warn;

pub struct GltfFile {
    document: Document,
    buffers: Vec<gltf::buffer::Data>,
    images: Vec<gltf::image::Data>,
}

impl GltfFile {
    /// Create a `GltfFile` from a byte stream
    /// # Errors
    /// return `ReflectionError` on failure
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ReflectionError> {
        let (document, buffers, images) =
            gltf::import_slice(bytes).map_err(|err| ReflectionError::Generic(err.to_string()))?;
        Ok(Self {
            document,
            buffers,
            images,
        })
    }

    pub fn gather_models(&self, resource_id: ResourceTypeAndId) -> Vec<(Model, String)> {
        let mut models = Vec::new();
        for mesh in self.document.meshes() {
            let mut meshes = Vec::new();
            for primitive in mesh.primitives() {
                let mut positions: Vec<Vec3> = Vec::new();
                let mut normals: Vec<Vec3> = Vec::new();
                let mut tangents: Vec<Vec4> = Vec::new();
                let mut tex_coords: Vec<Vec2> = Vec::new();
                let mut indices: Vec<u16> = Vec::new();

                // let buffer_data = gltf::Document::import_buffer_data(&document, buffer.index(), blob)?;

                let reader = primitive.reader(|buffer| Some(&self.buffers[buffer.index()]));
                if let Some(iter) = reader.read_positions() {
                    for position in iter {
                        // GLTF uses RH Y-up coordinate system, Legion Engine uses LH Y-up. Flipping Z for positions
                        // and normals gives us the desired result. Note that it also rotates the model 90 degrees around
                        // the up axis. It compensates 90 degrees rotation that happens when the model is exported from Blender
                        // to GLTF. As a result, imported model is oriented the same relative to the axis of Legion Engine as it
                        // was oriented relative to the axis of Blender.
                        positions.push(Vec3::new(position[0], position[1], -position[2]));
                    }
                }
                if let Some(iter) = reader.read_normals() {
                    for normal in iter {
                        normals.push(Vec3::new(normal[0], normal[1], -normal[2]));
                    }
                }
                if let Some(iter) = reader.read_tangents() {
                    for tangent in iter {
                        // Same rule as above applies to the tangents. W coordinate of the tangent contains the handedness
                        // of the tangent space. -1 handedness corresponds to a LH tangent basis in a RH coordinate system.
                        // Since our coordinate system is LH, we have to flip it on import.
                        tangents.push(Vec4::new(tangent[0], tangent[1], -tangent[2], -tangent[3]));
                    }
                }
                if let Some(tex_coords_option) = reader.read_tex_coords(0) {
                    match tex_coords_option {
                        ReadTexCoords::F32(iter) => {
                            for tex_coord in iter {
                                tex_coords.push(Vec2::new(tex_coord[0], tex_coord[1]));
                            }
                        }
                        _ => unreachable!("Integer UVs are not supported"),
                    }
                }
                if let Some(indices_option) = reader.read_indices() {
                    match indices_option {
                        ReadIndices::U8(iter) => {
                            for idx in iter {
                                indices.push(u16::from(idx));
                            }
                        }
                        ReadIndices::U16(iter) => {
                            for idx in iter {
                                indices.push(idx);
                            }
                        }
                        ReadIndices::U32(iter) => {
                            for idx in iter {
                                // TODO - will panic if does not fit in 16bits
                                indices.push(idx as u16);
                            }
                        }
                    }
                    for i in 0..indices.len() / 3 {
                        indices.swap(i * 3 + 1, i * 3 + 2);
                    }
                }

                if tangents.is_empty() && !normals.is_empty() {
                    tangents = lgn_math::calculate_tangents(&positions, &tex_coords, &indices)
                        .iter()
                        .map(|v| v.extend(-1.0))
                        .collect();
                }

                let material = primitive.material().name().map_or_else(
                    || {
                        primitive.material().index().map(|idx| {
                            ResourcePathId::from(resource_id)
                                .push_named(
                                    crate::offline::Material::TYPE,
                                    self.document.materials().nth(idx).unwrap().name().unwrap(),
                                )
                                .push(crate::runtime::Material::TYPE)
                        })
                    },
                    |material_name| {
                        Some(
                            ResourcePathId::from(resource_id)
                                .push_named(crate::offline::Material::TYPE, material_name)
                                .push(crate::runtime::Material::TYPE),
                        )
                    },
                );
                meshes.push(Mesh {
                    positions,
                    normals,
                    tangents,
                    tex_coords,
                    indices,
                    colors: Vec::new(),
                    material,
                });
            }
            models.push((
                Model {
                    meshes,
                    ..Model::default()
                },
                String::from(mesh.name().unwrap()),
            ));
        }
        models
    }

    pub fn gather_materials(&self, resource_id: ResourceTypeAndId) -> Vec<(Material, String)> {
        let mut materials = Vec::new();
        for material in self.document.materials() {
            let material_name = material.name().unwrap();
            let base_albedo = material.pbr_metallic_roughness().base_color_factor();
            let base_albedo = Color::from((
                (base_albedo[0] * 255.0) as u8,
                (base_albedo[1] * 255.0) as u8,
                (base_albedo[2] * 255.0) as u8,
                (base_albedo[3] * 255.0) as u8,
            ));
            let material_sampler = RefCell::new(None);
            let albedo = material
                .pbr_metallic_roughness()
                .base_color_texture()
                .map(|info| {
                    *material_sampler.borrow_mut() = Some(info.texture().sampler());
                    ResourcePathId::from(resource_id)
                        .push_named(
                            crate::runtime::RawTexture::TYPE,
                            texture_name(&info.texture()).unwrap().as_str(),
                        )
                        .push_named(crate::runtime::BinTexture::TYPE, "Albedo")
                });

            let normal = material.normal_texture().map(|info| {
                let normal_sampler = info.texture().sampler();
                if let Some(sampler) = &*material_sampler.borrow() {
                    if !samplers_equal(sampler, &normal_sampler) {
                        warn!("Material {} uses more than one sampler", material_name);
                    }
                } else {
                    *material_sampler.borrow_mut() = Some(normal_sampler);
                }

                ResourcePathId::from(resource_id)
                    .push_named(
                        crate::runtime::RawTexture::TYPE,
                        normal_texture_name(&info).unwrap().as_str(),
                    )
                    .push_named(crate::runtime::BinTexture::TYPE, "Normal")
            });
            let base_roughness = material.pbr_metallic_roughness().roughness_factor();
            let base_metalness = material.pbr_metallic_roughness().metallic_factor();
            let roughness = material
                .pbr_metallic_roughness()
                .metallic_roughness_texture()
                .map(|info| {
                    let roughness_sampler = info.texture().sampler();
                    if let Some(sampler) = &*material_sampler.borrow() {
                        if !samplers_equal(sampler, &roughness_sampler) {
                            warn!("Material {} uses more than one sampler", material_name);
                        }
                    } else {
                        *material_sampler.borrow_mut() = Some(roughness_sampler);
                    }
                    ResourcePathId::from(resource_id)
                        .push_named(
                            crate::runtime::RawTexture::TYPE,
                            format!("{}_Roughness", texture_name(&info.texture()).unwrap())
                                .as_str(),
                        )
                        .push_named(crate::runtime::BinTexture::TYPE, "Roughness")
                });
            let metalness = material
                .pbr_metallic_roughness()
                .metallic_roughness_texture()
                .map(|info| {
                    let metalness_sampler = info.texture().sampler();
                    if let Some(sampler) = &*material_sampler.borrow() {
                        if !samplers_equal(sampler, &metalness_sampler) {
                            warn!("Material {} uses more than one sampler", material_name);
                        }
                    } else {
                        *material_sampler.borrow_mut() = Some(metalness_sampler);
                    }
                    ResourcePathId::from(resource_id)
                        .push_named(
                            crate::runtime::RawTexture::TYPE,
                            format!("{}_Metalness", texture_name(&info.texture()).unwrap())
                                .as_str(),
                        )
                        .push_named(crate::runtime::BinTexture::TYPE, "Metalness")
                });
            materials.push((
                Material {
                    albedo,
                    normal,
                    roughness,
                    metalness,
                    base_albedo,
                    base_metalness,
                    base_roughness,
                    sampler: material_sampler.borrow().as_ref().map(build_sampler),
                    ..Material::default()
                },
                String::from(material_name),
            ));
        }
        materials
    }

    pub fn gather_textures(&self) -> Vec<(RawTexture, String)> {
        let mut metallic_roughness_textures = Vec::new();
        for material in self.document.materials() {
            if let Some(info) = material
                .pbr_metallic_roughness()
                .metallic_roughness_texture()
            {
                metallic_roughness_textures.push(texture_name(&info.texture()).unwrap());
            }
        }
        let mut textures = Vec::new();
        for texture in self.document.textures() {
            let name = texture_name(&texture).unwrap();
            let image = &self.images[texture.source().index()];
            if metallic_roughness_textures.contains(&name) {
                let mut roughness = Vec::new();
                let mut metalness = Vec::new();
                for i in 0..(image.width * image.height) as usize {
                    roughness.push(image.pixels[i * 3 + 1]);
                    metalness.push(image.pixels[i * 3 + 2]);
                }
                textures.push((
                    RawTexture {
                        kind: TextureType::_2D,
                        width: image.width,
                        height: image.height,
                        rgba: serde_bytes::ByteBuf::from(roughness),
                    },
                    format!("{}_Roughness", name),
                ));
                textures.push((
                    RawTexture {
                        kind: TextureType::_2D,
                        width: image.width,
                        height: image.height,
                        rgba: serde_bytes::ByteBuf::from(metalness),
                    },
                    format!("{}_Metalness", name),
                ));
            } else {
                textures.push((
                    RawTexture {
                        kind: TextureType::_2D,
                        width: image.width,
                        height: image.height,
                        rgba: match image.format {
                            //Format::R8 => image.pixels.clone().iter().flat_map(|v| vec![*v, 0, 0, 0]).collect(),
                            Format::R8G8B8A8 => serde_bytes::ByteBuf::from(image.pixels.clone()),
                            Format::R8G8B8 => {
                                let mut rgba = Vec::new();
                                for i in 0..(image.width * image.height) as usize {
                                    rgba.push(image.pixels[i * 3]);
                                    rgba.push(image.pixels[i * 3 + 1]);
                                    rgba.push(image.pixels[i * 3 + 2]);
                                    rgba.push(255);
                                }
                                serde_bytes::ByteBuf::from(rgba)
                            }
                            _ => unreachable!(),
                        },
                    },
                    name,
                ));
            }
        }
        textures
    }
}

fn samplers_equal(sampler1: &texture::Sampler<'_>, sampler2: &texture::Sampler<'_>) -> bool {
    sampler1.mag_filter() == sampler2.mag_filter()
        && sampler1.min_filter() == sampler2.min_filter()
        && sampler1.wrap_s() == sampler2.wrap_s()
        && sampler1.wrap_t() == sampler2.wrap_t()
}

fn build_sampler(sampler: &texture::Sampler<'_>) -> SamplerData {
    SamplerData {
        mag_filter: if let Some(mag_filter) = sampler.mag_filter() {
            match mag_filter {
                texture::MagFilter::Nearest => Filter::Nearest,
                texture::MagFilter::Linear => Filter::Linear,
            }
        } else {
            Filter::Linear
        },
        min_filter: if let Some(min_filter) = sampler.min_filter() {
            match min_filter {
                texture::MinFilter::Nearest
                | texture::MinFilter::NearestMipmapNearest
                | texture::MinFilter::NearestMipmapLinear => Filter::Nearest,
                texture::MinFilter::Linear
                | texture::MinFilter::LinearMipmapNearest
                | texture::MinFilter::LinearMipmapLinear => Filter::Linear,
            }
        } else {
            Filter::Linear
        },
        mip_filter: if let Some(min_filter) = sampler.min_filter() {
            #[allow(clippy::match_same_arms)]
            match min_filter {
                texture::MinFilter::NearestMipmapNearest
                | texture::MinFilter::LinearMipmapNearest => Filter::Nearest,
                texture::MinFilter::LinearMipmapLinear
                | texture::MinFilter::NearestMipmapLinear => Filter::Linear,
                _ => Filter::Linear,
            }
        } else {
            Filter::Linear
        },
        wrap_u: match sampler.wrap_s() {
            texture::WrappingMode::ClampToEdge => WrappingMode::ClampToEdge,
            texture::WrappingMode::MirroredRepeat => WrappingMode::MirroredRepeat,
            texture::WrappingMode::Repeat => WrappingMode::Repeat,
        },
        wrap_v: match sampler.wrap_t() {
            texture::WrappingMode::ClampToEdge => WrappingMode::ClampToEdge,
            texture::WrappingMode::MirroredRepeat => WrappingMode::MirroredRepeat,
            texture::WrappingMode::Repeat => WrappingMode::Repeat,
        },
    }
}

fn texture_name(texture: &texture::Texture<'_>) -> Result<String, <String as FromStr>::Err> {
    texture
        .name()
        .map_or(Ok(texture.index().to_string()), |texture_name| {
            String::from_str(texture_name)
        })
}

fn normal_texture_name(info: &NormalTexture<'_>) -> Result<String, <String as FromStr>::Err> {
    info.texture()
        .name()
        .map_or(Ok(info.texture().index().to_string()), |texture_name| {
            String::from_str(texture_name)
        })
}
