use std::{
    collections::hash_map::DefaultHasher,
    fmt::Display,
    hash::{Hash, Hasher},
};

use dashmap::DashMap;

/// Address of content in the content store.
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct ContentAddr(u64);

impl ContentAddr {
    pub fn checksum(content: &str) -> Self {
        let mut s = DefaultHasher::new();
        content.hash(&mut s);
        Self(s.finish())
    }
}

impl Display for ContentAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Default, Debug)]
pub struct ContentStore {
    content: DashMap<ContentAddr, String>,
}

impl ContentStore {
    pub async fn store(&self, content: String) -> ContentAddr {
        let addr = ContentAddr::checksum(&content);
        self.content.insert(addr, content);
        addr
    }

    pub async fn find(&self, addr: ContentAddr) -> Option<String> {
        self.content.get(&addr).map(|a| a.clone())
    }
}
