mod common;

use service::{
    compiler_interface::{
        ResourceGuid, ATLAS_COMPILER, MATERIAL_CONTENT, TEXTURE_A_CONTENT, TEXTURE_B_CONTENT,
        TEXTURE_C_CONTENT,
    },
    resource_manager::LoadCompletion,
    ResourcePathId,
};

pub const TEXTURE_ATLAS_CONTENT: &str = "Entity2: Texture Atlas";

use crate::common::setup;

#[tokio::test]
async fn source_change() {
    let (mut runtime, _, _, _) = setup(&[
        (ResourceGuid::TextureA, TEXTURE_A_CONTENT),
        (ResourceGuid::TextureB, TEXTURE_B_CONTENT),
        (ResourceGuid::TextureC, TEXTURE_C_CONTENT),
        (ResourceGuid::TextureAtlas, TEXTURE_ATLAS_CONTENT),
    ])
    .await;

    let atlas_id =
        ResourcePathId::new(ResourceGuid::TextureAtlas).transform(ATLAS_COMPILER.to_string());

    let _ = runtime
        .resource_manager
        .load(atlas_id.clone())
        .await
        .unwrap();

    let log = runtime.resource_manager.pop_log().await.unwrap();
    assert_eq!(log.id, atlas_id);
    println!("log: {:?}", log);
    assert!(matches!(log.result, LoadCompletion::Compiled));

    let _ = runtime
        .resource_manager
        .load(atlas_id.clone())
        .await
        .unwrap();

    let log = runtime.resource_manager.pop_log().await.unwrap();
    assert_eq!(log.id, atlas_id);
    println!("log: {:?}", log);
    assert!(matches!(log.result, LoadCompletion::Cached(_)));

    runtime
        .resource_manager
        .change(ResourceGuid::TextureAtlas, "new content")
        .await
        .unwrap();

    let _ = runtime
        .resource_manager
        .load(atlas_id.clone())
        .await
        .unwrap();

    let log = runtime.resource_manager.pop_log().await.unwrap();
    assert_eq!(log.id, atlas_id);
    assert!(matches!(log.result, LoadCompletion::Compiled));
}

#[tokio::test]
async fn unrelated_build_dep_change() {
    let (mut runtime, _, _, _) = setup(&[
        (ResourceGuid::TextureA, TEXTURE_A_CONTENT),
        (ResourceGuid::TextureB, TEXTURE_B_CONTENT),
        (ResourceGuid::TextureC, TEXTURE_C_CONTENT),
        (ResourceGuid::Material, MATERIAL_CONTENT),
        (ResourceGuid::TextureAtlas, TEXTURE_ATLAS_CONTENT),
    ])
    .await;

    let atlas_id =
        ResourcePathId::new(ResourceGuid::TextureAtlas).transform(ATLAS_COMPILER.to_string());

    let _ = runtime
        .resource_manager
        .load(atlas_id.clone())
        .await
        .unwrap();

    let log = runtime.resource_manager.pop_log().await.unwrap();
    assert_eq!(log.id, atlas_id);
    assert!(matches!(log.result, LoadCompletion::Compiled));

    let _ = runtime
        .resource_manager
        .load(atlas_id.clone())
        .await
        .unwrap();

    let log = runtime.resource_manager.pop_log().await.unwrap();
    assert_eq!(log.id, atlas_id);
    assert!(matches!(log.result, LoadCompletion::Cached(_)));

    runtime
        .resource_manager
        .change(ResourceGuid::Material, "new material content")
        .await
        .unwrap();

    let _ = runtime
        .resource_manager
        .load(atlas_id.clone())
        .await
        .unwrap();

    let log = runtime.resource_manager.pop_log().await.unwrap();
    println!("{:?}", log);
    assert_eq!(log.id, atlas_id);
    assert!(matches!(log.result, LoadCompletion::Cached(_)));
}

#[tokio::test]
async fn build_dep_change() {
    let (mut runtime, _, _, _) = setup(&[
        (ResourceGuid::TextureA, TEXTURE_A_CONTENT),
        (ResourceGuid::TextureB, TEXTURE_B_CONTENT),
        (ResourceGuid::TextureC, TEXTURE_C_CONTENT),
        (ResourceGuid::TextureAtlas, TEXTURE_ATLAS_CONTENT),
    ])
    .await;

    let atlas_id =
        ResourcePathId::new(ResourceGuid::TextureAtlas).transform(ATLAS_COMPILER.to_string());

    let _ = runtime
        .resource_manager
        .load(atlas_id.clone())
        .await
        .unwrap();

    let log = runtime.resource_manager.pop_log().await.unwrap();
    assert_eq!(log.id, atlas_id);
    assert!(matches!(log.result, LoadCompletion::Compiled));

    let _ = runtime
        .resource_manager
        .load(atlas_id.clone())
        .await
        .unwrap();

    let log = runtime.resource_manager.pop_log().await.unwrap();
    assert_eq!(log.id, atlas_id);
    assert!(matches!(log.result, LoadCompletion::Cached(_)));

    runtime
        .resource_manager
        .change(ResourceGuid::TextureA, "new texture content")
        .await
        .unwrap();

    let _ = runtime
        .resource_manager
        .load(atlas_id.clone())
        .await
        .unwrap();

    let log = runtime.resource_manager.pop_log().await.unwrap();
    assert_eq!(log.id, atlas_id);
    assert!(matches!(log.result, LoadCompletion::Compiled));
}
