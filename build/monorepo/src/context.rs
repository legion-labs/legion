use std::path::PathBuf;

use camino::Utf8Path;
use git2::Repository;
use guppy::graph::PackageGraph;

use crate::config::MonorepoConfig;
use crate::git::GitCli;
use crate::utils::project_root;
use crate::Error;
use crate::Result;

pub struct Context {
    config: MonorepoConfig,
    package_graph: PackageGraph,
    git_cli: GitCli,
}

impl Context {
    pub fn new() -> Result<Self> {
        let mut cmd = guppy::MetadataCommand::new();
        let package_graph = guppy::graph::PackageGraph::from_command(&mut cmd).map_err(|err| {
            Error::new(format!("failed to parse package graph {}", err)).with_source(err)
        })?;
        let config = MonorepoConfig::new(package_graph.workspace().root()).map_err(|err| {
            Error::new(format!("failed to parse workspace manifest {}", err)).with_source(err)
        })?;
        let git_cli = GitCli::new(project_root())?;
        Ok(Self {
            config,
            package_graph,
            git_cli,
        })
    }
    pub fn config(&self) -> &MonorepoConfig {
        &self.config
    }

    pub fn package_graph(&self) -> &PackageGraph {
        &self.package_graph
    }

    pub fn git_cli(&self) -> &GitCli {
        &self.git_cli
    }

    fn workspace_root(&self) -> &Utf8Path {
        self.package_graph.workspace().root()
    }

    fn git_repository(&self) -> Result<Repository> {
        Repository::open(self.workspace_root())
            .map_err(|err| Error::new("failed to open Git repository").with_source(err))
    }

    pub fn get_changed_files(&self, start: &str) -> Result<Vec<PathBuf>> {
        let repo = self.git_repository()?;
        let start = repo
            .revparse_single(start)
            .map_err(|err| Error::new("failed to parse Git revision").with_source(err))?
            .as_commit()
            .ok_or_else(|| Error::new("reference is not a commit"))?
            .tree()
            .unwrap();

        let diff = repo
            .diff_tree_to_workdir(Some(&start), None)
            .map_err(|err| Error::new("failed to generate diff").with_source(err))?;

        let prefix = repo
            .path()
            .parent()
            .ok_or_else(|| Error::new("failed to determine Git repository path"))?;

        let mut result = Vec::new();

        diff.print(git2::DiffFormat::NameOnly, |_, _, l| {
            let path = prefix.join(PathBuf::from(
                std::str::from_utf8(l.content()).unwrap().trim_end(),
            ));

            result.push(path);

            true
        })
        .map_err(|err| Error::new("failed to print diff").with_source(err))?;

        Ok(result)
    }
}
