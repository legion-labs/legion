use std::{
    collections::{hash_map::DefaultHasher, BTreeMap},
    hash::{Hash, Hasher},
};

use dashmap::DashMap;

use crate::{
    compiler_interface::CompilerError,
    content_store::{ContentAddr, ContentStore},
    ResourceGuid,
};

// todo: support many commit_root version
#[derive(Default)]
pub struct SourceControlBuilder {
    content: Vec<(ResourceGuid, String)>,
}

pub type CommitRoot = u64;

impl SourceControlBuilder {
    pub fn calculate_hash(content: &BTreeMap<ResourceGuid, ContentAddr>) -> CommitRoot {
        let mut s = DefaultHasher::new();
        content.hash(&mut s);
        s.finish()
    }

    pub fn add(mut self, guid: ResourceGuid, content: &str) -> Self {
        self.content.push((guid, content.to_owned()));
        self
    }

    pub async fn commit(self, content_store: &mut ContentStore) -> (CommitRoot, SourceControl) {
        let mut resource_content = BTreeMap::<ResourceGuid, ContentAddr>::new();
        for (guid, content) in self.content {
            let addr = content_store.store(content).await;
            resource_content.insert(guid, addr);
        }

        let resource_list: Vec<ResourceGuid> = resource_content.keys().copied().collect();

        let commit_root = Self::calculate_hash(&resource_content);

        let commit_resources = resource_content.iter().fold(
            DashMap::<(ResourceGuid, CommitRoot), ContentAddr>::new(),
            |acc, (guid, addr)| {
                acc.insert((*guid, commit_root), *addr);
                acc
            },
        );

        (
            commit_root,
            SourceControl {
                resource_list: DashMap::from_iter([(commit_root, resource_list)]),
                source_resource: commit_resources,
            },
        )
    }
}

#[derive(Default, Debug)]
pub struct SourceControl {
    resource_list: DashMap<CommitRoot, Vec<ResourceGuid>>,
    source_resource: DashMap<(ResourceGuid, CommitRoot), ContentAddr>,
}

impl SourceControl {
    pub async fn get(
        &self,
        content_store: &ContentStore,
        guid: ResourceGuid,
        commit_root: CommitRoot,
    ) -> Option<String> {
        if let Some(addr) = self.find_address(guid, commit_root) {
            let content = content_store.find(addr).await?;
            Some(content)
        } else {
            None
        }
    }

    pub fn find_address(&self, guid: ResourceGuid, commit_root: CommitRoot) -> Option<ContentAddr> {
        self.source_resource
            .get(&(guid, commit_root))
            .map(|addr| addr.clone())
    }

    pub async fn update(
        &self,
        content_store: &ContentStore,
        guid: ResourceGuid,
        new_content: &str,
        commit_root: CommitRoot,
    ) -> Result<(CommitRoot, ContentAddr), CompilerError> {
        if let Some(current_resource_list) = self.resource_list.get(&commit_root) {
            if !current_resource_list.contains(&guid) {
                return Err(CompilerError::NotFound);
            }

            let current_addr = self
                .source_resource
                .get(&(guid, commit_root))
                .map(|v| v.clone())
                .unwrap();

            let new_addr = content_store.store(new_content.to_owned()).await;

            if new_addr == current_addr {
                Ok((commit_root, current_addr))
            } else {
                let mut resources: BTreeMap<ResourceGuid, ContentAddr> = current_resource_list
                    .iter()
                    .map(|guid| {
                        (
                            *guid,
                            self.source_resource
                                .get(&(*guid, commit_root))
                                .map(|v| v.clone())
                                .unwrap(),
                        )
                    })
                    .collect();
                let prev_value = resources.insert(guid, new_addr);
                assert_eq!(prev_value.unwrap(), current_addr);

                let new_commit_root = SourceControlBuilder::calculate_hash(&resources);

                for (guid, addr) in resources {
                    let prev_addr = self.source_resource.insert((guid, new_commit_root), addr);
                    assert!(prev_addr.is_none() || prev_addr.unwrap() == addr);
                }

                self.resource_list
                    .insert(new_commit_root, current_resource_list.clone());

                Ok((new_commit_root, new_addr))
            }
        } else {
            Err(CompilerError::InvalidArg)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{compiler_interface::ResourceGuid, content_store::ContentStore};

    use super::SourceControlBuilder;

    #[tokio::test]
    async fn general() {
        let guid = ResourceGuid::Car;
        let original_content = "first";
        let changed_content = "second";

        let mut content_store = ContentStore::default();
        let (original_root_id, source_control) = SourceControlBuilder::default()
            .add(guid, original_content)
            .commit(&mut content_store)
            .await;

        let (new_root_id, _) = source_control
            .update(&content_store, guid, original_content, original_root_id)
            .await
            .unwrap();

        assert_eq!(new_root_id, original_root_id);

        let (new_root_id, _) = source_control
            .update(&content_store, guid, changed_content, original_root_id)
            .await
            .unwrap();

        assert_ne!(new_root_id, original_root_id);

        let (final_root_id, _) = source_control
            .update(&content_store, guid, original_content, new_root_id)
            .await
            .unwrap();

        assert_eq!(final_root_id, original_root_id);
    }
}
