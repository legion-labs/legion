#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use generic_data::offline::NullResource;
    use lgn_content_store::Provider;
    use lgn_data_offline::{Project, ResourcePathName, SourceResource};
    use lgn_data_runtime::prelude::*;

    #[allow(clippy::too_many_lines)]
    async fn create_actor(project: &mut Project) {
        let texture = project
            .add_resource(&NullResource::new_named("albedo.texture"))
            .await
            .unwrap();

        let mut material = NullResource::new_named("body.material");
        material.dependencies.push(ResourcePathId::from(texture));

        let material = project.add_resource(&material).await.unwrap();

        let mut geometry = NullResource::new_named("hero.geometry");
        geometry.dependencies.push(ResourcePathId::from(material));
        let geometry = project.add_resource(&geometry).await.unwrap();

        let skeleton = project
            .add_resource(&NullResource::new_named("hero.skeleton"))
            .await
            .unwrap();

        let mut actor = NullResource::new_named("hero.actor");
        actor.dependencies = vec![
            ResourcePathId::from(geometry),
            ResourcePathId::from(skeleton),
        ];
        let _actor = project.add_resource(&actor).await.unwrap();
    }

    async fn create_sky_material(project: &mut Project) {
        let texture = project
            .add_resource(&NullResource::new_named("sky.texture"))
            .await
            .unwrap();

        let mut material = NullResource::new_named("sky.material");
        material.dependencies.push(ResourcePathId::from(texture));

        let _material = project.add_resource(&material).await.unwrap();
    }

    /* test disabled due to problems with project deletion.
    sqlx doesn't release .db file for some reason.

    #[tokio::test]
    async fn proj_create_delete() {
        let root = tempfile::tempdir().unwrap();

        let project = Project::create_with_remote_mock(root.path())
            .await
            .expect("failed to create project");
        let same_project = Project::create_with_remote_mock(root.path()).await;
        assert!(same_project.is_err());

        project.delete().await;

        let _project = Project::create_with_remote_mock(root.path())
            .await
            .expect("failed to re-create project");
        let same_project = Project::create_with_remote_mock(root.path()).await;
        assert!(same_project.is_err());
    }*/

    #[tokio::test]
    async fn local_changes() {
        let root = tempfile::tempdir().unwrap();
        let provider = Arc::new(Provider::new_in_memory());
        let mut project = Project::create_with_remote_mock(root.path(), provider)
            .await
            .expect("new project");

        create_actor(&mut project).await;

        assert_eq!(project.local_resource_list().await.unwrap().len(), 5);
    }

    #[tokio::test]
    async fn commit() {
        let root = tempfile::tempdir().unwrap();

        let provider = Arc::new(Provider::new_in_memory());
        let mut project = Project::create_with_remote_mock(root.path(), provider)
            .await
            .expect("new project");

        create_actor(&mut project).await;

        let actor_id = project
            .find_resource(&ResourcePathName::new("hero.actor"))
            .await
            .unwrap();

        assert_eq!(project.local_resource_list().await.unwrap().len(), 5);
        assert_eq!(project.remote_resource_list().await.unwrap().len(), 0);

        // modify before commit
        {
            let mut resource = project
                .load_resource::<NullResource>(actor_id.id)
                .await
                .unwrap();
            resource.content = 8;

            project
                .save_resource(actor_id.id, resource.as_ref())
                .await
                .unwrap();
        }

        project.commit("add resources").await.unwrap();

        assert_eq!(project.local_resource_list().await.unwrap().len(), 0);
        assert_eq!(project.remote_resource_list().await.unwrap().len(), 5);

        // modify resource
        {
            let mut content = project
                .load_resource::<NullResource>(actor_id.id)
                .await
                .unwrap();
            assert_eq!(content.content, 8);
            content.content = 9;
            project
                .save_resource(actor_id.id, content.as_ref())
                .await
                .unwrap();
            assert_eq!(project.local_resource_list().await.unwrap().len(), 1);
        }

        project.commit("update actor").await.unwrap();

        assert_eq!(project.local_resource_list().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn change_to_previous() {
        let root = tempfile::tempdir().unwrap();
        let provider = Arc::new(Provider::new_in_memory());
        let mut project = Project::create_with_remote_mock(root.path(), provider)
            .await
            .expect("new project");
        create_actor(&mut project).await;

        let actor_id = project
            .find_resource(&ResourcePathName::new("hero.actor"))
            .await
            .unwrap();

        project.commit("initial actor").await.unwrap();

        // modify resource
        let original_content = {
            let mut content = project
                .load_resource::<NullResource>(actor_id.id)
                .await
                .unwrap();
            let previous_value = content.content;
            content.content = 9;
            project
                .save_resource(actor_id.id, content.as_ref())
                .await
                .unwrap();
            previous_value
        };

        {
            let mut content = project
                .load_resource::<NullResource>(actor_id.id)
                .await
                .unwrap();
            content.content = original_content;
            project
                .save_resource(actor_id.id, content.as_ref())
                .await
                .unwrap();
        }

        project.commit("no changes").await.unwrap();
    }

    #[tokio::test]
    async fn immediate_dependencies() {
        let root = tempfile::tempdir().unwrap();
        let provider = Arc::new(Provider::new_in_memory());
        let mut project = Project::create_with_remote_mock(root.path(), provider)
            .await
            .expect("new project");
        create_actor(&mut project).await;

        let top_level_resource = project
            .find_resource(&ResourcePathName::new("hero.actor"))
            .await
            .unwrap();

        let (_, dependencies) = project.resource_info(top_level_resource.id).unwrap();

        assert_eq!(dependencies.len(), 2);
    }

    async fn rename_assert(
        proj: &mut Project,
        old_name: ResourcePathName,
        new_name: ResourcePathName,
    ) {
        let skeleton_id = proj.find_resource(&old_name).await;
        assert!(skeleton_id.is_ok());
        let skeleton_id = skeleton_id.unwrap();

        let prev_name = proj.rename_resource(skeleton_id, &new_name).await;
        assert!(prev_name.is_ok());
        let prev_name = prev_name.unwrap();
        assert_eq!(&prev_name, &old_name);

        assert!(proj.find_resource(&old_name).await.is_err());
        assert_eq!(proj.find_resource(&new_name).await.unwrap(), skeleton_id);
    }

    #[tokio::test]
    async fn rename() {
        let root = tempfile::tempdir().unwrap();
        let provider = Arc::new(Provider::new_in_memory());
        let mut project = Project::create_with_remote_mock(root.path(), provider)
            .await
            .expect("new project");
        create_actor(&mut project).await;
        assert!(project.commit("rename test").await.is_ok());
        create_sky_material(&mut project).await;

        rename_assert(
            &mut project,
            ResourcePathName::new("hero.skeleton"),
            ResourcePathName::new("boss.skeleton"),
        )
        .await;
        rename_assert(
            &mut project,
            ResourcePathName::new("sky.material"),
            ResourcePathName::new("clouds.material"),
        )
        .await;
    }
}
