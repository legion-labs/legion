//! A crate with modules supporting data compilation process.
//!
//! * [`compiler_api`] provides an interface for implementing a data compiler.
//! * [`compiler_cmd`] provides utilities for interacting with data compilers.

// crate-specific lint exceptions:
#![allow(unsafe_code, clippy::missing_errors_doc)]
#![warn(missing_docs)]

use core::fmt;
use std::str::FromStr;

use compiler_api::CompilerError;
use lgn_content_store::Identifier;
use lgn_data_runtime::ResourcePathId;
use serde::{Deserialize, Serialize};

/// Description of a compiled resource.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, PartialOrd, Ord)]
pub struct CompiledResource {
    /// The path of derived resource.
    pub path: ResourcePathId,
    /// The checksum of the resource.
    pub content_id: Identifier,
}

impl fmt::Display for CompiledResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}^{}", self.content_id, self.path))
    }
}

fn from_str_internal(s: &str) -> Result<(Identifier, ResourcePathId), Box<dyn std::error::Error>> {
    let mut iter = s.split('^');
    let content_id = Identifier::from_str(iter.next().unwrap())?;
    let path = ResourcePathId::from_str(iter.next().unwrap())?;
    Ok((content_id, path))
}

impl FromStr for CompiledResource {
    type Err = CompilerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (content_id, path) = from_str_internal(s).map_err(|_e| CompilerError::Parse)?;
        Ok(Self { path, content_id })
    }
}

/// The output of data compilation.
///
/// `CompiledResources` contains the list of compiled resources.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CompiledResources {
    /// The description of all compiled resources.
    pub compiled_resources: Vec<CompiledResource>,
}

impl CompiledResources {
    /// Prepare for serialization.
    /// Will sort contents to guarantee that the serialization is deterministic
    pub fn pre_serialize(&mut self) {
        self.compiled_resources.sort_by(|a, b| a.path.cmp(&b.path));
    }

    /// Creates a runtime [`lgn_data_runtime::manifest::Manifest`] from an
    /// offline [`CompiledResources`].
    ///
    /// Provided filter functor will be used to determine if a given asset
    /// should be included in the manifest.
    ///
    /// This is a temporary solution that will be replaced by a **packaging**
    /// process. For now, we simply create a runtime manifest by filtering
    /// out non-asset resources and by identifying content by `ResourceId` -
    /// which runtime operates on.
    pub fn into_rt_manifest(
        self,
        filter: fn(&ResourcePathId) -> bool,
    ) -> lgn_data_runtime::manifest::Manifest {
        let output = lgn_data_runtime::manifest::Manifest::default();

        let runtime_resources = self
            .compiled_resources
            .into_iter()
            .filter(|resource| filter(&resource.path))
            .collect::<Vec<_>>();

        for resource in runtime_resources {
            output.insert(resource.path.resource_id(), resource.content_id);
        }
        output
    }
}

/// Build target enumeration.
///
/// `TODO`: This needs to be more extensible.
#[derive(Clone, Copy)]
pub enum Target {
    /// Game client.
    Game,
    /// Server.
    Server,
    /// Backend service.
    Backend,
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Game => write!(f, "game"),
            Self::Server => write!(f, "server"),
            Self::Backend => write!(f, "backend"),
        }
    }
}

impl FromStr for Target {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "game" => Ok(Self::Game),
            "server" => Ok(Self::Server),
            "backend" => Ok(Self::Backend),
            _ => Err(()),
        }
    }
}

/// Build platform enumeration.
#[derive(Clone, Copy)]
pub enum Platform {
    /// Windows
    Windows,
    /// Unix
    Unix,
    /// Game Console X
    ConsoleX,
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Platform::Windows => write!(f, "windows"),
            Platform::Unix => write!(f, "unix"),
            Platform::ConsoleX => write!(f, "consolex"),
        }
    }
}

impl FromStr for Platform {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "windows" => Ok(Self::Windows),
            "unix" => Ok(Self::Unix),
            "consolex" => Ok(Self::ConsoleX),
            _ => Err(()),
        }
    }
}

/// Defines user's language/region.
#[derive(Clone)]
pub struct Locale(String);

impl Locale {
    /// Creates a new Locale.
    pub fn new(v: &str) -> Self {
        Self(String::from(v))
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub mod compiler_api;
pub mod compiler_cmd;
pub mod compiler_node;
pub mod compiler_reflection;
pub mod compiler_utils;
