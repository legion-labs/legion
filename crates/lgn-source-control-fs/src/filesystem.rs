use fuser::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request,
};
use lgn_blob_storage::BlobStorage;
use lgn_source_control::{IndexBackend, MapOtherError, Result, Tree};
use lgn_tracing::{debug, error};
use libc::{EISDIR, ENOENT, ENOTDIR};
use std::{
    ffi::OsStr,
    sync::Mutex,
    time::{Duration, UNIX_EPOCH},
};

use super::inode_index::InodeIndex;

const TTL: Duration = Duration::from_secs(1); // 1 second

pub struct SourceControlFilesystem {
    handle: tokio::runtime::Handle,
    index_backend: Box<dyn IndexBackend>,
    blob_storage: Box<dyn BlobStorage>,
    branch_name: String,
    uid: u32,
    gid: u32,
    inode_index: Mutex<InodeIndex>,
}

impl SourceControlFilesystem {
    pub async fn new(index_backend: Box<dyn IndexBackend>, branch_name: String) -> Result<Self> {
        let handle = tokio::runtime::Handle::current();
        let tree = Self::read_tree(index_backend.as_ref(), &branch_name).await?;
        let blob_storage_url = index_backend.get_blob_storage_url().await?;
        let blob_storage = blob_storage_url
            .into_blob_storage()
            .await
            .map_other_err("failed to create blob storage")?;

        Ok(Self {
            handle,
            index_backend,
            blob_storage,
            branch_name,
            uid: users::get_current_uid(),
            gid: users::get_current_gid(),
            inode_index: Mutex::new(InodeIndex::new(tree)),
        })
    }

    /// Synchronize the filesystem to the latest state.
    fn sync(&self) -> Result<()> {
        let tree = self.handle.block_on(async move {
            Self::read_tree(self.index_backend.as_ref(), &self.branch_name).await
        })?;

        self.inode_index.lock().unwrap().update_tree(tree);

        Ok(())
    }

    async fn read_tree(index_backend: &dyn IndexBackend, branch_name: &str) -> Result<Tree> {
        let branch = index_backend.read_branch(branch_name).await?;
        let commit = index_backend.read_commit(&branch.head).await?;
        index_backend.read_tree(&commit.root_tree_id).await
    }

    fn get_blob(&self, hash: &str) -> Result<Vec<u8>> {
        self.handle.block_on(async move {
            self.blob_storage
                .read_blob(hash)
                .await
                .map_other_err(format!("failed to read blob for `{}`", hash))
        })
    }

    fn tree_to_attr(&self, ino: u64, tree: &Tree) -> FileAttr {
        let (kind, nlink, size, perm) = match tree {
            Tree::Directory { .. } => (FileType::Directory, 2, 0, 0o550),
            Tree::File { .. } => (FileType::RegularFile, 1, 42, 0o440),
        };

        FileAttr {
            ino,
            size,
            blocks: 0,
            atime: UNIX_EPOCH, // 1970-01-01 00:00:00
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind,
            perm,
            nlink,
            uid: self.uid,
            gid: self.gid,
            rdev: 0,
            flags: 0,
            blksize: 0,
        }
    }
}

impl Filesystem for SourceControlFilesystem {
    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        debug!("lookup({:?}, {:?})", parent, name);

        if let Some((ino, _, tree)) = self
            .inode_index
            .lock()
            .unwrap()
            .get_tree_node_by_parent_path(parent, name.to_str().unwrap())
        {
            reply.entry(&TTL, &self.tree_to_attr(ino, tree), 0);
        } else {
            reply.error(ENOENT);
        }
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, reply: ReplyAttr) {
        if let Err(err) = self.sync() {
            error!("failed to sync filesystem: {}", err);
        }

        debug!("getattr({:?})", ino);

        // This method gets called for every file access.
        //
        // The first call is always made for the root directory, with `ino` set
        // to 1.
        if let Some((ino, _, tree)) = self.inode_index.lock().unwrap().get_tree_node(ino) {
            reply.attr(&TTL, &self.tree_to_attr(ino, tree));
        } else {
            reply.error(ENOENT);
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
        debug!("read({:?}, {:?})", ino, offset);

        if let Some((_, _, tree)) = self.inode_index.lock().unwrap().get_tree_node(ino) {
            match tree {
                Tree::Directory { .. } => reply.error(EISDIR),
                Tree::File { hash, .. } => match self.get_blob(hash) {
                    Ok(data) => {
                        reply.data(&data[offset as usize..]);
                    }
                    Err(e) => {
                        error!("failed to read blob for inode {}: {}", ino, e);
                        reply.error(ENOENT);
                    }
                },
            }
        } else {
            reply.error(ENOENT);
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
        debug!("readdir({:?}, {:?})", ino, offset);

        let inode_index = self.inode_index.lock().unwrap();

        if let Some((_, parent_path, tree)) = inode_index.get_tree_node(ino) {
            match tree {
                Tree::File { .. } => reply.error(ENOTDIR),
                Tree::Directory { children, .. } => {
                    for (i, entry) in [
                        (ino, FileType::Directory, "."),
                        (ino, FileType::Directory, ".."),
                    ]
                    .into_iter()
                    .chain(children.iter().map(|(name, child)| {
                        (
                            inode_index
                                .get_inode_by_path(&parent_path.append(name))
                                .unwrap(),
                            match child {
                                Tree::Directory { .. } => FileType::Directory,
                                Tree::File { .. } => FileType::RegularFile,
                            },
                            name.as_str(),
                        )
                    }))
                    .skip(offset as usize)
                    .enumerate()
                    {
                        if reply.add(entry.0, (i + 1) as i64, entry.1, entry.2) {
                            break;
                        }
                    }

                    reply.ok();
                }
            }
        } else {
            reply.error(ENOENT);
        }
    }
}
