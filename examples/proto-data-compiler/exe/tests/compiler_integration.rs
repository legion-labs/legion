use petgraph::{dot::Dot, Graph};
use service::compiler_interface::{
    ResourceGuid, ATLAS_COMPILER, MATERIAL_COMPILER, MATERIAL_CONTENT, TEXTURE_A_CONTENT,
    TEXTURE_B_CONTENT, TEXTURE_C_CONTENT,
};
use service::{dependency_graph, EdgeDependencyType, ResourcePathId};

use crate::common::{graph_eq, setup};

mod common;

pub const TEXTURE_ATLAS_CONTENT: &str = "Entity2: Texture Atlas";

#[tokio::test]
async fn atlas_compiler() {
    let (runtime, services, build_params, commit_root) = setup(&[
        (ResourceGuid::TextureA, TEXTURE_A_CONTENT),
        (ResourceGuid::TextureB, TEXTURE_B_CONTENT),
        (ResourceGuid::TextureC, TEXTURE_C_CONTENT),
        (ResourceGuid::TextureAtlas, TEXTURE_ATLAS_CONTENT),
    ])
    .await;

    let source_id =
        ResourcePathId::new(ResourceGuid::TextureAtlas).transform(ATLAS_COMPILER.to_string());

    let output = runtime
        .resource_manager
        .load(source_id.clone())
        .await
        .unwrap();

    let expected_output = [TEXTURE_A_CONTENT, TEXTURE_B_CONTENT, TEXTURE_C_CONTENT].concat();

    assert_eq!(output, expected_output);
    // assert!(context.output.content[0].references.is_empty());

    let expected_build_graph = {
        let mut g = Graph::<ResourcePathId, EdgeDependencyType>::new();
        let source = g.add_node(
            ResourcePathId::new(ResourceGuid::TextureAtlas).transform(ATLAS_COMPILER.to_string()),
        );
        let definition = g.add_node(ResourcePathId::new(ResourceGuid::TextureAtlas));
        let texture_a = g.add_node(ResourcePathId::new(ResourceGuid::TextureA));
        let texture_b = g.add_node(ResourcePathId::new(ResourceGuid::TextureB));
        let texture_c = g.add_node(ResourcePathId::new(ResourceGuid::TextureC));

        g.extend_with_edges(&[
            (source, definition, EdgeDependencyType::Build),
            (source, texture_a, EdgeDependencyType::Build),
            (source, texture_b, EdgeDependencyType::Build),
            (source, texture_c, EdgeDependencyType::Build),
        ]);
        g
    };

    let graph = dependency_graph(
        source_id.clone(),
        commit_root,
        &build_params,
        &services.build_db,
        &services.source_control,
    )
    .await;

    //dbg!(&graph);
    //dbg!(&expected_build_graph);
    println!("{}", Dot::new(&graph));
    assert!(graph_eq(&expected_build_graph, &graph));
}

#[tokio::test]
async fn material_compiler() {
    let (runtime, services, build_params, commit_root) = setup(&[
        (ResourceGuid::TextureA, TEXTURE_A_CONTENT),
        (ResourceGuid::TextureB, TEXTURE_B_CONTENT),
        (ResourceGuid::TextureC, TEXTURE_C_CONTENT),
        (ResourceGuid::TextureAtlas, TEXTURE_ATLAS_CONTENT),
        (ResourceGuid::Material, MATERIAL_CONTENT),
    ])
    .await;

    let material_id =
        ResourcePathId::new(ResourceGuid::Material).transform(MATERIAL_COMPILER.to_string());

    runtime
        .resource_manager
        .load(material_id.clone())
        .await
        .unwrap();

    //assert_eq(output, expected_output);

    let expected_build_graph = {
        let mut g = Graph::<ResourcePathId, EdgeDependencyType>::new();

        let compiled_material = g.add_node(
            ResourcePathId::new(ResourceGuid::Material).transform(MATERIAL_COMPILER.to_string()),
        );
        let material_source = g.add_node(ResourcePathId::new(ResourceGuid::Material));
        let compiled_atlas = g.add_node(
            ResourcePathId::new(ResourceGuid::TextureAtlas).transform(ATLAS_COMPILER.to_string()),
        );
        let atlas_source = g.add_node(ResourcePathId::new(ResourceGuid::TextureAtlas));
        let texture_a = g.add_node(ResourcePathId::new(ResourceGuid::TextureA));
        let texture_b = g.add_node(ResourcePathId::new(ResourceGuid::TextureB));
        let texture_c = g.add_node(ResourcePathId::new(ResourceGuid::TextureC));

        g.extend_with_edges(&[
            (
                compiled_material,
                material_source,
                EdgeDependencyType::Build,
            ),
            (
                compiled_material,
                compiled_atlas,
                EdgeDependencyType::Runtime,
            ),
            (compiled_atlas, atlas_source, EdgeDependencyType::Build),
            (compiled_atlas, texture_a, EdgeDependencyType::Build),
            (compiled_atlas, texture_b, EdgeDependencyType::Build),
            (compiled_atlas, texture_c, EdgeDependencyType::Build),
        ]);
        g
    };

    let graph = dependency_graph(
        material_id,
        commit_root,
        &build_params,
        &services.build_db,
        &services.source_control,
    )
    .await;

    println!("Expected graph\n{}", Dot::new(&expected_build_graph));
    println!("graph\n{}", Dot::new(&graph));
    assert!(graph_eq(&expected_build_graph, &graph));
    println!("{}", Dot::new(&graph));
}
