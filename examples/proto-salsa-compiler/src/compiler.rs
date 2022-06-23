use std::sync::Arc;

use crate::{
    atlas::compile_atlas,
    collision::{compile_collision, AABBCollision},
    entity::compile_entity,
    expression::execute_expression,
    material::compile_material,
    meta::meta_get_resource_path,
    navmesh::{compile_navmesh, query_collisions, Navmesh},
    package::{package, package_sea_ps5, package_see_ps5},
    runtime_dependency::add_runtime_dependency,
    texture::{compile_jpg, compile_png, compile_texture, CompressionType},
    BuildParams, CompilerError, ContentAddr, Locale,
};

#[salsa::query_group(CompilerStorage)]
pub trait Compiler {
    #[salsa::input]
    fn read(&self, name: String) -> String;

    fn compile_material(&self) -> String;

    fn compile_atlas(
        &self,
        textures_in_atlas: Vec<String>,
        build_params: Arc<BuildParams>,
    ) -> String;

    fn compile_collision(&self, name: Arc<String>) -> AABBCollision;

    fn compile_entity(&self, name: String, build_params: Arc<BuildParams>) -> String;

    fn execute_expression(
        &self,
        expression: String,
        build_params: Arc<BuildParams>,
    ) -> Result<Vec<String>, CompilerError>;

    fn meta_get_resource_path(
        &self,
        meta_content: String,
        build_params: Arc<BuildParams>,
    ) -> Result<String, CompilerError>;

    fn query_collisions(&self, quadrant: Arc<AABBCollision>) -> Vec<AABBCollision>;
    fn compile_navmesh(&self, quadrant: Arc<AABBCollision>) -> Navmesh;

    // European countries
    fn package_see_ps5(&self) -> ContentAddr;
    // Asian countries
    fn package_sea_ps5(&self) -> ContentAddr;

    fn package(&self, languages: Vec<Locale>, content_to_package: Vec<String>) -> ContentAddr;

    // Textures
    fn compile_texture(&self, name: String, compression: CompressionType) -> String;
    fn compile_jpg(&self, name: String, compression: CompressionType) -> String;
    fn compile_png(&self, name: String, compression: CompressionType) -> String;

    fn add_runtime_dependency(
        &self,
        resource_path_id: String,
        build_params: Arc<BuildParams>,
    ) -> i8;
}
