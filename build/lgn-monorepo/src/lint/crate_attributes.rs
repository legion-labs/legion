use globset::{Candidate, GlobBuilder, GlobSet, GlobSetBuilder};
use lgn_tracing::span_fn;
use regex::Regex;

use crate::{context::Context, Error, Result};

#[span_fn]
pub fn run(ctx: &Context) -> Result<()> {
    let workspace = ctx.package_graph()?.workspace();
    let names_lints = name_rules(ctx)?;
    let mut failed_name_rules = 0;
    for (path, package) in workspace.iter_by_path() {
        if path.file_name().unwrap() != package.name() {
            eprintln!(
                "{}: package name doesn't match parent dir name: {}",
                package.name(),
                path
            );
            failed_name_rules += 1;
        }
        for (re, neg_re, glob_set, help) in &names_lints {
            let candidate = Candidate::new(path);
            if glob_set.is_match_candidate(&candidate)
                && (!re.is_match(package.name())
                    || (neg_re.is_some() && neg_re.as_ref().unwrap().is_match(package.name())))
            {
                eprintln!("{}: name rule mismatch: {}", package.name(), help);
                failed_name_rules += 1;
            }
        }
    }
    if failed_name_rules > 0 {
        return Err(Error::new(format!(
            "{} names rules failed",
            failed_name_rules,
        )));
    }
    Ok(())
}

type NameRule = (Regex, Option<Regex>, GlobSet, String);
fn name_rules(ctx: &Context) -> Result<Vec<NameRule>> {
    let mut names_lints = vec![];
    // build names lint rules
    for lint in &ctx.config().lints.crate_attributes.name_rules {
        let regex = Regex::new(&lint.pattern).map_err(|e| Error::new(format!("{}", e)))?;
        let negative_regex = lint
            .negative_pattern
            .as_ref()
            .map(|str| Regex::new(str).map_err(|e| Error::new(format!("{}", e))));
        let negative_regex = if let Some(negative_regex) = negative_regex {
            if negative_regex.is_err() {
                return Err(Error::new(format!(
                    "invalid negative pattern for lint {}",
                    lint.pattern,
                ))
                .with_source(negative_regex.err().unwrap()));
            }
            Some(negative_regex.unwrap())
        } else {
            None
        };
        let mut builder = GlobSetBuilder::new();
        for glob in &lint.globs {
            let glob = GlobBuilder::new(glob)
                .literal_separator(lint.glob_literal_separator.unwrap_or_default())
                .build()
                .map_err(|err| Error::new("").with_source(err))?;
            builder.add(glob);
        }
        let glob_set = builder
            .build()
            .map_err(|err| Error::new("").with_source(err))?;
        names_lints.push((regex, negative_regex, glob_set, lint.help.clone()));
    }
    Ok(names_lints)
}
