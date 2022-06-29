use std::sync::Arc;

use downcast_rs::{impl_downcast, Downcast};

use crate::{
    atlas::compile_atlas,
    collision::{compile_aabb, query_collisions, AABBCollision},
    expression::{execute_expression, run},
    meta::meta_get_resource_path,
    navmesh::{compile_navmesh, Navmesh},
    package::{package, package_sea_ps5, package_see_ps5},
    runtime_dependency::add_runtime_dependency,
    texture::{compile_jpg, compile_png, compile_texture, CompressionType},
    BuildParams, ContentAddr, Locale,
};

pub trait AnyEq: Downcast {
    // Perform the test.
    fn equals_a(&self, _: &dyn AnyEq) -> bool;
}
impl_downcast!(AnyEq);

impl PartialEq for dyn AnyEq + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.equals_a(other)
    }
}

impl Eq for dyn AnyEq + '_ {
    fn assert_receiver_is_total_eq(&self) {}
}

// Implement A for all 'static types implementing PartialEq.
impl<T: 'static + PartialEq> AnyEq for T {
    fn equals_a(&self, other: &dyn AnyEq) -> bool {
        // Do a type-safe casting. If the types are different,
        // return false, otherwise test the values for equality.
        other.downcast_ref::<T>().map_or(false, |a| self == a)
    }
}

impl std::fmt::Debug for dyn AnyEq + '_ {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[salsa::query_group(CompilerStorage)]
pub trait Compiler<'a> {
    #[salsa::input]
    fn read(&self, name: String) -> String;

    fn compile_atlas(&self, atlas_expressions: String, build_params: Arc<BuildParams>) -> String;

    fn compile_aabb(
        &self,
        min_x: Arc<String>,
        min_y: Arc<String>,
        min_z: Arc<String>,
        max_x: Arc<String>,
        max_y: Arc<String>,
        max_z: Arc<String>,
    ) -> AABBCollision;

    fn run(&self, expressions: String, build_params: Arc<BuildParams>) -> Vec<Arc<Box<dyn AnyEq>>>;

    fn execute_expression(
        &self,
        expression: String,
        build_params: Arc<BuildParams>,
    ) -> Arc<Box<dyn AnyEq>>;

    fn meta_get_resource_path(
        &self,
        meta_content: String,
        build_params: Arc<BuildParams>,
    ) -> String;

    fn query_collisions(
        &self,
        expressions: String,
        build_params: Arc<BuildParams>,
    ) -> Arc<Vec<AABBCollision>>;

    fn compile_navmesh(&self, collisions: Arc<Vec<AABBCollision>>) -> Navmesh;

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
