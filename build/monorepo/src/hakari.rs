use camino::Utf8PathBuf;
use guppy::graph::cargo::CargoResolverVersion;
use hakari::{HakariBuilder, HakariOutputOptions};
use lgn_telemetry::trace_scope;
use toml_edit::Document;

use crate::{action_step, context::Context, Error, Result};

pub fn run(ctx: &Context) -> Result<()> {
    trace_scope!();
    action_step!("Monorepo", "Running rules determination");

    // Use this workspace's PackageGraph for these tests.
    let package_graph = ctx.package_graph()?;

    // The second argument to HakariBuilder::new specifies a Hakari (workspace-hack) package.
    // In this repository, the package is called "guppy-workspace-hack".
    let mut hakari_builder = HakariBuilder::new(
        package_graph,
        Some(
            package_graph
                .workspace()
                .member_by_name("lgn-workspace-hack")
                .map_err(|_err| Error::new("Exclude package was not found"))?
                .id(),
        ),
    )
    .map_err(|err| Error::new("Failed to build hakari graph").with_source(err))?;

    hakari_builder
        .set_platforms(["x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc"])
        .map_err(|err| Error::new("Failed to set platforms").with_source(err))?;
    hakari_builder.set_resolver(CargoResolverVersion::V2);
    hakari_builder
        .add_traversal_excludes([package_graph
            .workspace()
            .member_by_name("lgn-monorepo")
            .map_err(|_err| Error::new("Exclude package was not found"))?
            .id()])
        .map_err(|err| Error::new("Failed to set platforms").with_source(err))?;

    // HakariBuilder has a number of config options. For this example, use the defaults.
    let hakari = hakari_builder.compute();
    let existing_toml = hakari
        .read_toml()
        .expect("hakari package specified by builder")
        .map_err(|_err| Error::new("Exclude package was not found"))?;
    let new_toml = hakari
        .to_toml_string(&HakariOutputOptions::default())
        .map_err(|_err| Error::new("Exclude package was not found"))?;

    // hakari can be used to build a TOML representation that forms part of a Cargo.toml file.
    // Existing Cargo.toml files can be managed using Hakari::read_toml.
    existing_toml
        .write_to_file(&new_toml)
        .map_err(|_err| Error::new("Exclude package was not found"))?;

    for package in package_graph.workspace().iter() {
        if package.name() == "lgn-workspace-hack" || package.name() == "lgn-monorepo" {
            continue;
        }
        let crate_dir = package.source().workspace_path().unwrap();
        let mut toml_path = Utf8PathBuf::from(ctx.workspace_root());
        toml_path.push(crate_dir);
        toml_path.push("Cargo.toml");
        let contents = match std::fs::read_to_string(&toml_path) {
            Ok(contents) => contents,
            Err(error) => return Err(Error::new("").with_source(error)),
        };

        let mut doc = contents.parse::<Document>().expect("invalid doc");
        doc["dependencies"]["lgn-workspace-hack"]["path"] = toml_edit::value(
            path_to_workspace_hack(package.source().workspace_path().unwrap().to_owned()),
        );
        doc["dependencies"]["lgn-workspace-hack"]["optional"] = toml_edit::value(true);
        std::fs::write(&toml_path, doc.to_string()).expect("write");
    }

    Ok(())
}

fn path_to_workspace_hack(mut path: Utf8PathBuf) -> String {
    let mut relative_path = String::new();
    while path.pop() {
        relative_path.push_str("../");
    }
    relative_path.push_str("build/workspace-hack");
    relative_path
}
