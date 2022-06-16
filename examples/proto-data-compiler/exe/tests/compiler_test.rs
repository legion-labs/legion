const RESOURCE_A_CONTENT: &str = "Resource A";
const RESOURCE_B_CONTENT: &str = "Resource B";
const RESOURCE_C_CONTENT: &str = "Resource C";
const RESOURCE_D_CONTENT: &str = "Resource D";
const RESOURCE_E_CONTENT: &str = "Resource E";
const RESOURCE_F_CONTENT: &str = "Resource F";
const RESOURCE_G_CONTENT: &str = "Resource G";
const RESOURCE_H_CONTENT: &str = "Resource H";
const RESOURCE_I_CONTENT: &str = "Resource I";
const RESOURCE_J_CONTENT: &str = "Resource J";
const RESOURCE_K_CONTENT: &str = "Resource K";
const RESOURCE_L_CONTENT: &str = "Resource L";
const RESOURCE_M_CONTENT: &str = "Resource M";
const RESOURCE_N_CONTENT: &str = "Resource N";
const RESOURCE_O_CONTENT: &str = "Resource O";
const RESOURCE_P_CONTENT: &str = "Resource P";

use petgraph::{dot::Dot, Graph};
use service::{
    compiler_interface::{ResourceGuid, TEST_COMPILATION_APPEND, TEST_COMPILER},
    dependency_graph, EdgeDependencyType, ResourcePathId,
};

use crate::common::{graph_eq, setup};

mod common;

// Resource A has a runtime dependency on B
#[tokio::test]
async fn a_runtime_b() {
    let (runtime, services, build_params, commit_root) = setup(&[
        (ResourceGuid::ResourceA, RESOURCE_A_CONTENT),
        (ResourceGuid::ResourceB, RESOURCE_B_CONTENT),
    ])
    .await;

    let source_id =
        ResourcePathId::new(ResourceGuid::ResourceA).transform(TEST_COMPILER.to_string());

    let output = runtime
        .resource_manager
        .load(source_id.clone())
        .await
        .unwrap();

    let expected_output = RESOURCE_A_CONTENT.to_string() + TEST_COMPILATION_APPEND;

    assert_eq!(output, expected_output);

    let expected_build_graph = {
        let mut g = Graph::<ResourcePathId, EdgeDependencyType>::new();
        let resource_a_transform = g.add_node(source_id.clone());
        let resource_a_source = g.add_node(ResourcePathId::new(ResourceGuid::ResourceA));
        let resource_b_transform = g.add_node(
            ResourcePathId::new(ResourceGuid::ResourceB).transform(TEST_COMPILER.to_string()),
        );
        let resource_b_source = g.add_node(ResourcePathId::new(ResourceGuid::ResourceB));

        g.extend_with_edges(&[
            (
                resource_a_transform,
                resource_a_source,
                EdgeDependencyType::Build,
            ),
            (
                resource_a_transform,
                resource_b_transform,
                EdgeDependencyType::Runtime,
            ),
            (
                resource_b_transform,
                resource_b_source,
                EdgeDependencyType::Build,
            ),
        ]);
        g
    };

    let graph = dependency_graph(
        source_id,
        commit_root,
        &build_params,
        &services.build_db,
        &services.source_control,
    )
    .await;

    //dbg!(&graph);
    //dbg!(&expected_build_graph);
    println!("Compiled Graph\n{}", Dot::new(&graph));
    // println!("Expected Graph\n{}", Dot::new(&expected_build_graph));
    assert!(graph_eq(&expected_build_graph, &graph));
}

// Resource C has a runtime dependency on D
// Resource D has a runtime dependency on E
#[tokio::test]
async fn c_runtime_d_runtime_e() {
    let (runtime, services, build_params, commit_root) = setup(&[
        (ResourceGuid::ResourceC, RESOURCE_C_CONTENT),
        (ResourceGuid::ResourceD, RESOURCE_D_CONTENT),
        (ResourceGuid::ResourceE, RESOURCE_E_CONTENT),
    ])
    .await;

    let source_id =
        ResourcePathId::new(ResourceGuid::ResourceC).transform(TEST_COMPILER.to_string());

    let output = runtime
        .resource_manager
        .load(source_id.clone())
        .await
        .unwrap();

    let expected_output = RESOURCE_C_CONTENT.to_string() + TEST_COMPILATION_APPEND;

    assert_eq!(output, expected_output);

    let expected_build_graph = {
        let mut g = Graph::<ResourcePathId, EdgeDependencyType>::new();
        let resource_c_transform = g.add_node(source_id.clone());
        let resource_c_source = g.add_node(ResourcePathId::new(ResourceGuid::ResourceC));
        let resource_d_transform = g.add_node(
            ResourcePathId::new(ResourceGuid::ResourceD).transform(TEST_COMPILER.to_string()),
        );
        let resource_d_source = g.add_node(ResourcePathId::new(ResourceGuid::ResourceD));
        let resource_e_transform = g.add_node(
            ResourcePathId::new(ResourceGuid::ResourceE).transform(TEST_COMPILER.to_string()),
        );
        let resource_e_source = g.add_node(ResourcePathId::new(ResourceGuid::ResourceE));

        g.extend_with_edges(&[
            (
                resource_c_transform,
                resource_c_source,
                EdgeDependencyType::Build,
            ),
            (
                resource_c_transform,
                resource_d_transform,
                EdgeDependencyType::Runtime,
            ),
            (
                resource_d_transform,
                resource_d_source,
                EdgeDependencyType::Build,
            ),
            (
                resource_d_transform,
                resource_e_transform,
                EdgeDependencyType::Runtime,
            ),
            (
                resource_e_transform,
                resource_e_source,
                EdgeDependencyType::Build,
            ),
        ]);
        g
    };

    let graph = dependency_graph(
        source_id,
        commit_root,
        &build_params,
        &services.build_db,
        &services.source_control,
    )
    .await;

    //dbg!(&graph);
    //dbg!(&expected_build_graph);
    println!("Compiled Graph\n{}", Dot::new(&graph));
    // println!("Expected Graph\n{}", Dot::new(&expected_build_graph));
    assert!(graph_eq(&expected_build_graph, &graph));
}

// Resource F has a build dependency on G
#[tokio::test]
async fn f_build_g() {
    let (runtime, services, build_params, commit_root) = setup(&[
        (ResourceGuid::ResourceF, RESOURCE_F_CONTENT),
        (ResourceGuid::ResourceG, RESOURCE_G_CONTENT),
    ])
    .await;

    let source_id =
        ResourcePathId::new(ResourceGuid::ResourceF).transform(TEST_COMPILER.to_string());

    let output = runtime
        .resource_manager
        .load(source_id.clone())
        .await
        .unwrap();

    let expected_output = RESOURCE_F_CONTENT.to_string()
        + RESOURCE_G_CONTENT
        + TEST_COMPILATION_APPEND
        + TEST_COMPILATION_APPEND;

    assert_eq!(output, expected_output);

    let expected_build_graph = {
        let mut g = Graph::<ResourcePathId, EdgeDependencyType>::new();
        let resource_f_transform = g.add_node(source_id.clone());
        let resource_f_source = g.add_node(ResourcePathId::new(ResourceGuid::ResourceF));
        let resource_g_transform = g.add_node(
            ResourcePathId::new(ResourceGuid::ResourceG).transform(TEST_COMPILER.to_string()),
        );
        let resource_g_source = g.add_node(ResourcePathId::new(ResourceGuid::ResourceG));

        g.extend_with_edges(&[
            (
                resource_f_transform,
                resource_f_source,
                EdgeDependencyType::Build,
            ),
            (
                resource_f_transform,
                resource_g_transform,
                EdgeDependencyType::Build,
            ),
            (
                resource_g_transform,
                resource_g_source,
                EdgeDependencyType::Build,
            ),
        ]);
        g
    };

    let graph = dependency_graph(
        source_id,
        commit_root,
        &build_params,
        &services.build_db,
        &services.source_control,
    )
    .await;

    //dbg!(&graph);
    //dbg!(&expected_build_graph);
    println!("Compiled Graph\n{}", Dot::new(&graph));
    // println!("Expected Graph\n{}", Dot::new(&expected_build_graph));
    assert!(graph_eq(&expected_build_graph, &graph));
}

// Resource H has a build dependency on I
// Resource I has a build dependency on J
#[tokio::test]
async fn h_build_i_build_j() {
    let (runtime, services, build_params, commit_root) = setup(&[
        (ResourceGuid::ResourceH, RESOURCE_H_CONTENT),
        (ResourceGuid::ResourceI, RESOURCE_I_CONTENT),
        (ResourceGuid::ResourceJ, RESOURCE_J_CONTENT),
    ])
    .await;

    let source_id =
        ResourcePathId::new(ResourceGuid::ResourceH).transform(TEST_COMPILER.to_string());

    let output = runtime
        .resource_manager
        .load(source_id.clone())
        .await
        .unwrap();

    let expected_output = RESOURCE_H_CONTENT.to_string()
        + RESOURCE_I_CONTENT
        + RESOURCE_J_CONTENT
        + TEST_COMPILATION_APPEND
        + TEST_COMPILATION_APPEND
        + TEST_COMPILATION_APPEND;

    assert_eq!(output, expected_output);

    let expected_build_graph = {
        let mut g = Graph::<ResourcePathId, EdgeDependencyType>::new();
        let resource_h_transform = g.add_node(source_id.clone());
        let resource_h_source = g.add_node(ResourcePathId::new(ResourceGuid::ResourceH));
        let resource_i_transform = g.add_node(
            ResourcePathId::new(ResourceGuid::ResourceI).transform(TEST_COMPILER.to_string()),
        );
        let resource_i_source = g.add_node(ResourcePathId::new(ResourceGuid::ResourceI));
        let resource_j_transform = g.add_node(
            ResourcePathId::new(ResourceGuid::ResourceJ).transform(TEST_COMPILER.to_string()),
        );
        let resource_j_source = g.add_node(ResourcePathId::new(ResourceGuid::ResourceJ));

        g.extend_with_edges(&[
            (
                resource_h_transform,
                resource_h_source,
                EdgeDependencyType::Build,
            ),
            (
                resource_h_transform,
                resource_i_transform,
                EdgeDependencyType::Build,
            ),
            (
                resource_i_transform,
                resource_i_source,
                EdgeDependencyType::Build,
            ),
            (
                resource_i_transform,
                resource_j_transform,
                EdgeDependencyType::Build,
            ),
            (
                resource_j_transform,
                resource_j_source,
                EdgeDependencyType::Build,
            ),
        ]);
        g
    };

    let graph = dependency_graph(
        source_id,
        commit_root,
        &build_params,
        &services.build_db,
        &services.source_control,
    )
    .await;

    //dbg!(&graph);
    //dbg!(&expected_build_graph);
    println!("Compiled Graph\n{}", Dot::new(&graph));
    // println!("Expected Graph\n{}", Dot::new(&expected_build_graph));
    assert!(graph_eq(&expected_build_graph, &graph));
}

// Resource K has a runtime dependency on L
// Resource L has a build dependency on M
#[tokio::test]
async fn k_runtime_l_build_m() {
    let (runtime, services, build_params, commit_root) = setup(&[
        (ResourceGuid::ResourceK, RESOURCE_K_CONTENT),
        (ResourceGuid::ResourceL, RESOURCE_L_CONTENT),
        (ResourceGuid::ResourceM, RESOURCE_M_CONTENT),
    ])
    .await;

    let source_id =
        ResourcePathId::new(ResourceGuid::ResourceK).transform(TEST_COMPILER.to_string());

    let output = runtime
        .resource_manager
        .load(source_id.clone())
        .await
        .unwrap();

    let expected_output = RESOURCE_K_CONTENT.to_string() + TEST_COMPILATION_APPEND;

    assert_eq!(output, expected_output);

    let expected_build_graph = {
        let mut g = Graph::<ResourcePathId, EdgeDependencyType>::new();
        let resource_k_transform = g.add_node(source_id.clone());
        let resource_k_source = g.add_node(ResourcePathId::new(ResourceGuid::ResourceK));
        let resource_l_transform = g.add_node(
            ResourcePathId::new(ResourceGuid::ResourceL).transform(TEST_COMPILER.to_string()),
        );
        let resource_l_source = g.add_node(ResourcePathId::new(ResourceGuid::ResourceL));
        let resource_m_transform = g.add_node(
            ResourcePathId::new(ResourceGuid::ResourceM).transform(TEST_COMPILER.to_string()),
        );
        let resource_m_source = g.add_node(ResourcePathId::new(ResourceGuid::ResourceM));

        g.extend_with_edges(&[
            (
                resource_k_transform,
                resource_k_source,
                EdgeDependencyType::Build,
            ),
            (
                resource_k_transform,
                resource_l_transform,
                EdgeDependencyType::Runtime,
            ),
            (
                resource_l_transform,
                resource_l_source,
                EdgeDependencyType::Build,
            ),
            (
                resource_l_transform,
                resource_m_transform,
                EdgeDependencyType::Build,
            ),
            (
                resource_m_transform,
                resource_m_source,
                EdgeDependencyType::Build,
            ),
        ]);
        g
    };

    let graph = dependency_graph(
        source_id,
        commit_root,
        &build_params,
        &services.build_db,
        &services.source_control,
    )
    .await;

    //dbg!(&graph);
    //dbg!(&expected_build_graph);
    println!("Compiled Graph\n{}", Dot::new(&graph));
    // println!("Expected Graph\n{}", Dot::new(&expected_build_graph));
    assert!(graph_eq(&expected_build_graph, &graph));
}

// Resource N has a build dependency on O
// Resource O has a runtime dependency on P
#[tokio::test]
async fn n_build_o_runtime_p() {
    let (runtime, services, build_params, commit_root) = setup(&[
        (ResourceGuid::ResourceN, RESOURCE_N_CONTENT),
        (ResourceGuid::ResourceO, RESOURCE_O_CONTENT),
        (ResourceGuid::ResourceP, RESOURCE_P_CONTENT),
    ])
    .await;

    let source_id =
        ResourcePathId::new(ResourceGuid::ResourceN).transform(TEST_COMPILER.to_string());

    let output = runtime
        .resource_manager
        .load(source_id.clone())
        .await
        .unwrap();

    let expected_output = RESOURCE_N_CONTENT.to_string()
        + RESOURCE_O_CONTENT
        + TEST_COMPILATION_APPEND
        + TEST_COMPILATION_APPEND;

    assert_eq!(output, expected_output);

    let expected_build_graph = {
        let mut g = Graph::<ResourcePathId, EdgeDependencyType>::new();
        let resource_n_transform = g.add_node(source_id.clone());
        let resource_n_source = g.add_node(ResourcePathId::new(ResourceGuid::ResourceN));
        let resource_o_transform = g.add_node(
            ResourcePathId::new(ResourceGuid::ResourceO).transform(TEST_COMPILER.to_string()),
        );
        let resource_o_source = g.add_node(ResourcePathId::new(ResourceGuid::ResourceO));
        let resource_p_transform = g.add_node(
            ResourcePathId::new(ResourceGuid::ResourceP).transform(TEST_COMPILER.to_string()),
        );
        let resource_p_source = g.add_node(ResourcePathId::new(ResourceGuid::ResourceP));

        g.extend_with_edges(&[
            (
                resource_n_transform,
                resource_n_source,
                EdgeDependencyType::Build,
            ),
            (
                resource_n_transform,
                resource_o_transform,
                EdgeDependencyType::Build,
            ),
            (
                resource_o_transform,
                resource_o_source,
                EdgeDependencyType::Build,
            ),
            (
                resource_o_transform,
                resource_p_transform,
                EdgeDependencyType::Runtime,
            ),
            (
                resource_p_transform,
                resource_p_source,
                EdgeDependencyType::Build,
            ),
        ]);
        g
    };

    let graph = dependency_graph(
        source_id,
        commit_root,
        &build_params,
        &services.build_db,
        &services.source_control,
    )
    .await;

    //dbg!(&graph);
    //dbg!(&expected_build_graph);
    println!("Compiled Graph\n{}", Dot::new(&graph));
    // println!("Expected Graph\n{}", Dot::new(&expected_build_graph));
    assert!(graph_eq(&expected_build_graph, &graph));
}
