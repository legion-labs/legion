use anyhow::Result;
use fuser::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request,
};
use lgn_source_control::{Tree, TreeNode};
use lgn_tracing::{error, info};
use libc::ENOENT;
use std::{
    collections::HashMap,
    ffi::OsStr,
    sync::Mutex,
    time::{Duration, UNIX_EPOCH},
};

const TTL: Duration = Duration::from_secs(1); // 1 second

pub struct SourceControlFilesystem {
    handle: tokio::runtime::Handle,
    index_backend: Box<dyn lgn_source_control::IndexBackend>,
    branch: String,
    uid: u32,
    gid: u32,
    ino_index: Mutex<InoIndex>,
}

#[derive(Debug)]
struct InoIndex {
    by_ino: HashMap<u64, InoEntry>,
    by_hash: HashMap<String, InoEntry>,
    next_ino: u64,
}

#[derive(Debug, Clone)]
struct InoEntry {
    attr: FileAttr,
    tree_node: TreeNode,
}

impl Default for InoIndex {
    fn default() -> Self {
        Self {
            by_ino: HashMap::new(),
            by_hash: HashMap::new(),
            next_ino: 2,
        }
    }
}

impl InoIndex {
    fn get_next_available_ino(&mut self) -> u64 {
        while self.by_ino.contains_key(&self.next_ino) {
            self.next_ino += 1;
        }

        self.next_ino
    }

    fn get_entry_for_tree_node(&mut self, tree_node: &TreeNode, mut attr: FileAttr) -> InoEntry {
        if let Some(entry) = self.by_hash.get(&tree_node.hash) {
            entry.clone()
        } else {
            attr.ino = self.get_next_available_ino();

            let entry = InoEntry {
                tree_node: tree_node.clone(),
                attr,
            };

            self.by_ino.insert(attr.ino, entry.clone());
            self.by_hash.insert(tree_node.hash.clone(), entry.clone());

            entry
        }
    }

    fn get_for_ino(&self, ino: u64) -> Option<InoEntry> {
        self.by_ino.get(&ino).cloned()
    }
}

impl SourceControlFilesystem {
    pub fn new(index_backend: Box<dyn lgn_source_control::IndexBackend>, branch: String) -> Self {
        let handle = tokio::runtime::Handle::current();

        Self {
            handle,
            index_backend,
            branch,
            uid: users::get_current_uid(),
            gid: users::get_current_gid(),
            ino_index: Mutex::new(InoIndex::default()),
        }
    }

    fn get_entry_for_tree_node(&self, tree_node: &TreeNode, attr: FileAttr) -> InoEntry {
        self.ino_index
            .lock()
            .unwrap()
            .get_entry_for_tree_node(tree_node, attr)
    }

    fn get_for_ino(&self, ino: u64) -> Option<InoEntry> {
        self.ino_index.lock().unwrap().get_for_ino(ino)
    }

    fn read_tree(&self) -> Result<Tree> {
        self.handle
            .block_on(async move {
                let branch = self.index_backend.read_branch(&self.branch).await?;
                let commit = self.index_backend.read_commit(&branch.head).await?;
                self.index_backend.read_tree(&commit.root_hash).await
            })
            .map_err(Into::into)
    }

    fn get_blob(&self, hash: &str) -> Result<Vec<u8>> {
        self.handle.block_on(async move {
            let blob_storage_url = self
                .index_backend
                .get_blob_storage_url()
                .await
                .map_err::<anyhow::Error, _>(Into::into)?;

            let blob_storage = blob_storage_url
                .into_blob_storage()
                .await
                .map_err::<anyhow::Error, _>(Into::into)?;

            blob_storage
                .read_blob(hash)
                .await
                .map_err::<anyhow::Error, _>(Into::into)
        })
    }
}

impl Filesystem for SourceControlFilesystem {
    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        info!("lookup({:?}, {:?})", parent, name);

        if parent == 1 {
            let tree = match self.read_tree() {
                Ok(tree) => tree,
                Err(err) => {
                    error!("failed to read tree: {}", err);
                    reply.error(ENOENT);
                    return;
                }
            };

            if let Some(node) = tree
                .file_nodes
                .iter()
                .find(|node| node.name == name.to_string_lossy())
            {
                println!("found node for: {}", name.to_string_lossy());
                let entry = self.get_entry_for_tree_node(
                    node,
                    FileAttr {
                        ino: 0,
                        size: 42,
                        blocks: 1,
                        atime: UNIX_EPOCH, // 1970-01-01 00:00:00
                        mtime: UNIX_EPOCH,
                        ctime: UNIX_EPOCH,
                        crtime: UNIX_EPOCH,
                        kind: FileType::RegularFile,
                        perm: 0o644,
                        nlink: 1,
                        uid: self.uid,
                        gid: self.gid,
                        rdev: 0,
                        flags: 0,
                        blksize: 0,
                    },
                );
                println!("entry: {:?}", entry);

                reply.entry(&TTL, &entry.attr, 0);
                return;
            }

            if let Some(node) = tree
                .directory_nodes
                .iter()
                .find(|node| node.name == name.to_string_lossy())
            {
                let entry = self.get_entry_for_tree_node(
                    node,
                    FileAttr {
                        ino: 0,
                        size: 0,
                        blocks: 0,
                        atime: UNIX_EPOCH, // 1970-01-01 00:00:00
                        mtime: UNIX_EPOCH,
                        ctime: UNIX_EPOCH,
                        crtime: UNIX_EPOCH,
                        kind: FileType::Directory,
                        perm: 0o755,
                        nlink: 2,
                        uid: self.uid,
                        gid: self.gid,
                        rdev: 0,
                        flags: 0,
                        blksize: 0,
                    },
                );

                reply.entry(&TTL, &entry.attr, 0);
                return;
            }
        }

        reply.error(ENOENT);
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, reply: ReplyAttr) {
        // This method gets called for every file access.
        //
        // As a special case, the first call is always made for the root
        // directory, with `ino` set to 1.

        if ino == 1 {
            let attr = FileAttr {
                ino,
                size: 0,
                blocks: 0,
                atime: UNIX_EPOCH, // 1970-01-01 00:00:00
                mtime: UNIX_EPOCH,
                ctime: UNIX_EPOCH,
                crtime: UNIX_EPOCH,
                kind: FileType::Directory,
                perm: 0o755,
                nlink: 2,
                uid: self.uid,
                gid: self.gid,
                rdev: 0,
                flags: 0,
                blksize: 0,
            };

            reply.attr(&TTL, &attr);
            return;
        }

        match self.get_for_ino(ino) {
            Some(entry) => {
                reply.attr(&TTL, &entry.attr);
            }
            None => reply.error(ENOENT),
        }
    }

    fn read(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        _size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        match self.get_for_ino(ino) {
            Some(entry) => match self.get_blob(&entry.tree_node.hash) {
                Ok(data) => {
                    reply.data(&data[offset as usize..]);
                }
                Err(err) => {
                    error!("failed to read blob: {}", err);
                    reply.error(ENOENT);
                }
            },
            None => reply.error(ENOENT),
        }
    }

    fn readdir(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        if ino != 1 {
            reply.error(ENOENT);
            return;
        }

        let tree = match self.read_tree() {
            Ok(tree) => tree,
            Err(err) => {
                error!("failed to read tree: {}", err);
                reply.error(ENOENT);
                return;
            }
        };

        let entries = vec![
            (ino, FileType::Directory, "."),
            (ino, FileType::Directory, ".."),
        ]
        .into_iter()
        .chain(tree.file_nodes.iter().map(|node| {
            let entry = self.get_entry_for_tree_node(
                node,
                FileAttr {
                    ino: 0,
                    size: 42,
                    blocks: 1,
                    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
                    mtime: UNIX_EPOCH,
                    ctime: UNIX_EPOCH,
                    crtime: UNIX_EPOCH,
                    kind: FileType::RegularFile,
                    perm: 0o644,
                    nlink: 1,
                    uid: self.uid,
                    gid: self.gid,
                    rdev: 0,
                    flags: 0,
                    blksize: 0,
                },
            );

            (entry.attr.ino, entry.attr.kind, node.name.as_str())
        }))
        .chain(tree.directory_nodes.iter().map(|node| {
            let entry = self.get_entry_for_tree_node(
                node,
                FileAttr {
                    ino: 0,
                    size: 0,
                    blocks: 0,
                    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
                    mtime: UNIX_EPOCH,
                    ctime: UNIX_EPOCH,
                    crtime: UNIX_EPOCH,
                    kind: FileType::Directory,
                    perm: 0o755,
                    nlink: 2,
                    uid: self.uid,
                    gid: self.gid,
                    rdev: 0,
                    flags: 0,
                    blksize: 0,
                },
            );

            (entry.attr.ino, entry.attr.kind, node.name.as_str())
        }));

        for (i, entry) in entries.skip(offset as usize).enumerate() {
            // i + 1 means the index of the next entry
            if reply.add(entry.0, (i + 1) as i64, entry.1, entry.2) {
                break;
            }
        }

        reply.ok();
    }
}
