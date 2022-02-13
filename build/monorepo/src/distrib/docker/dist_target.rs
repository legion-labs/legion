use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    process::{Command, Stdio},
};

use camino::{Utf8Path, Utf8PathBuf};
use lgn_tracing::{debug, warn};
use monorepo_base::{action_step, skip_step};
use regex::Regex;
use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::{
    cargo::{target_config, target_dir},
    context::Context,
    distrib::{
        self,
        dist_target::{clean, copy_binaries, copy_extra_files},
        DistPackage,
    },
    Error, ErrorContext, Result,
};

use super::DockerMetadata;

pub const DEFAULT_DOCKER_REGISTRY_ENV_VAR_NAME: &str = "MONOREPO_DOCKER_REGISTRY";

pub struct DockerDistTarget<'g> {
    pub package: &'g DistPackage<'g>,
    pub metadata: DockerMetadata,
}

impl Display for DockerDistTarget<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "docker[{}]", self.package.name())
    }
}

impl<'g> DockerDistTarget<'g> {
    pub fn name(&self) -> &str {
        // we are supposed to have filled the name with a default value
        self.metadata.name.as_ref().unwrap()
    }

    pub fn build(&self, ctx: &Context, args: &distrib::Args) -> Result<()> {
        if target_config(ctx, &args.build_args)?.contains("windows") {
            skip_step!(
                "Unsupported",
                "Docker publish is not supported for windows targets"
            );
            return Ok(());
        }

        let root = self.docker_root(ctx, args)?;
        let docker_target_bin_dir = self.docker_target_bin_dir(ctx, args)?;

        clean(&root)?;

        let binaries = self.package.build_binaries(ctx, args)?;
        copy_binaries(&docker_target_bin_dir, binaries.values())?;
        copy_extra_files(&self.metadata.extra_files, self.package.root(), &root)?;

        let dockerfile = self.write_dockerfile(ctx, args, &binaries)?;
        self.build_dockerfile(args, &dockerfile)?;

        Ok(())
    }

    pub fn publish(&self, _ctx: &Context, args: &distrib::Args) -> Result<()> {
        if args.build_args.mode() == "debug" && !args.force {
            skip_step!(
                "Unsupported",
                "Docker images can't be published in debug mode unless `--force` is specified"
            );
            return Ok(());
        }

        self.docker_login()?;
        self.docker_push(args)?;

        Ok(())
    }

    fn docker_pull(args: &distrib::Args, docker_image_name: &str) -> Result<bool> {
        let mut cmd = Command::new("docker");

        debug!(
            "Will now pull docker image `{}` to check for existence",
            docker_image_name
        );

        let docker_args = vec!["pull", docker_image_name];

        action_step!("Running", "`docker {}`", docker_args.join(" "),);

        cmd.args(docker_args);

        if args.build_args.verbose > 0 {
            let status = cmd.status().map_err(Error::from_source).with_full_context(
                "failed to pull Docker image",
                "The pull of the Docker image failed which could indicate a configuration problem.",
            )?;

            Ok(status.success())
        } else {
            let output = cmd.output().map_err(Error::from_source).with_full_context(
                "failed to pull Docker image",
                "The pull of the Docker image failed which could indicate a configuration problem. You may want to re-run the command with `--verbose` to get more information.",
            )?;

            Ok(output.status.success())
        }
    }

    fn docker_push(&self, args: &distrib::Args) -> Result<()> {
        let mut cmd = Command::new("docker");
        let docker_image_name = self.docker_image_name()?;

        if args.force {
            debug!("`--force` specified: not checking for Docker image existence before pushing");
        } else if Self::docker_pull(args, &docker_image_name)? {
            debug!("Up to date image `{}` already exists", docker_image_name);
            skip_step!(
                "Up-to-date",
                "Docker image `{}` already exists",
                docker_image_name,
            );
            return Ok(());
        }

        debug!("Will now push docker image `{}`", docker_image_name);
        if let Some(aws_ecr_information) = self.get_aws_ecr_information()? {
            debug!("AWS ECR information found: assuming the image is hosted on AWS ECR in account `{}` and region `{}`", aws_ecr_information.account_id, aws_ecr_information.region);

            if self.metadata.allow_aws_ecr_creation {
                debug!("AWS ECR repository creation is allowed for this target");

                if args.dry_run {
                    warn!(
                        "`--dry-run` specified, will not really ensure the ECR repository exists"
                    );
                } else {
                    self.ensure_aws_ecr_repository_exists(&aws_ecr_information)?;
                }
            } else {
                debug!("AWS ECR repository creation is not allowed for this target - if this is not intended, specify `allows_aws_ecr_creation` in `Cargo.toml`");
            }
        } else {
            debug!(
                "No AWS ECR information found - assuming the image is hosted on another provider"
            );
        }

        let docker_args = vec!["push", &docker_image_name];

        if args.dry_run {
            warn!("Would now execute: docker {}", docker_args.join(" "));
            warn!("`--dry-run` specified: not continuing for real");

            return Ok(());
        }

        action_step!("Running", "`docker {}`", docker_args.join(" "),);

        cmd.args(docker_args);

        if args.build_args.verbose > 0 {
            let status = cmd.status().map_err(Error::from_source).with_full_context(
                "failed to push Docker image",
                "The push of the Docker image failed which could indicate a configuration problem.",
            )?;

            if !status.success() {
                return Err(Error::new("failed to push Docker image").with_explanation(
                    "The push of the Docker image failed. Check the logs above to determine the cause.",
                ));
            }
        } else {
            let output = cmd.output().map_err(Error::from_source).with_full_context(
                "failed to push Docker image",
                "The push of the Docker image failed which could indicate a configuration problem. You may want to re-run the command with `--verbose` to get more information.",
            )?;

            if !output.status.success() {
                return Err(Error::new("failed to push Docker image")
                    .with_explanation("The push of the Docker image failed. Check the logs below to determine the cause.")
                    .with_output(String::from_utf8_lossy(&output.stderr)));
            };
        }

        Ok(())
    }

    fn docker_login(&self) -> Result<()> {
        let aws_ecr_information = self
            .get_aws_ecr_information()?
            .ok_or_else(|| Error::new("AWS ECR information not found"))?;

        let mut cmd = Command::new("aws");
        cmd.args(&[
            "ecr",
            "get-login-password",
            "--region",
            &aws_ecr_information.region,
        ]);
        let child = cmd.stdout(Stdio::piped()).spawn().map_err(|err| {
            Error::new("failed to run `aws ecr get-login-password`").with_source(err)
        })?;

        let mut cmd = Command::new("docker");
        cmd.args(&[
            "login",
            "--username",
            "AWS",
            "--password-stdin",
            &aws_ecr_information.to_string(),
        ]);
        cmd.stdin(child.stdout.unwrap());
        let exit_status = cmd
            .status()
            .map_err(|err| Error::new("failed to run `docker login`").with_source(err))?;
        if exit_status.success() {
            debug!("Successfully logged in to Docker");
            Ok(())
        } else {
            Err(Error::new("failed to login to the docker registry"))
        }
    }

    fn ensure_aws_ecr_repository_exists(
        &self,
        aws_ecr_information: &AwsEcrInformation,
    ) -> Result<()> {
        debug!(
            "Ensuring AWS ECR repository exists for `{}`",
            aws_ecr_information.to_string()
        );
        let mut cmd = Command::new("aws");
        let package_name_tag = format!("Key=PackageName,Value={}", self.package.name());
        cmd.args(&[
            "ecr",
            "create-repository",
            "--repository-name",
            &aws_ecr_information.repository_name,
            "--tags",
            "Key=CreatedBy,Value=monorepo",
            &package_name_tag,
            "--region",
            &aws_ecr_information.region,
        ]);
        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|err| Error::new("failed to run `aws s3 cp`").with_source(err))?;

        if output.status.success() {
            debug!(
                "AWS ECR repository `{}` created",
                aws_ecr_information.to_string()
            );
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("RepositoryAlreadyExistsException") {
                debug!(
                    "AWS ECR repository `{}` already exists",
                    aws_ecr_information.to_string()
                );
                Ok(())
            } else {
                Err(Error::new("failed to create ecr registry").with_output(stderr))
            }
        }
    }

    fn build_dockerfile(&self, args: &distrib::Args, docker_file: &Utf8Path) -> Result<()> {
        if cfg!(windows) {
            skip_step!("Unsupported", "Docker build is not supported on Windows");
            return Ok(());
        }

        let mut cmd = Command::new("docker");
        let docker_image_name = self.docker_image_name()?;

        let docker_root = docker_file
            .parent()
            .ok_or_else(|| Error::new("failed to determine Docker root"))?;

        debug!("Moving to: {}", docker_root);

        cmd.current_dir(docker_root);

        let docker_args = vec!["build", "-t", &docker_image_name, "."];

        action_step!("Running", "`docker {}`", docker_args.join(" "),);

        cmd.args(docker_args);

        // Disable the annoying `Use 'docker scan' to run Snyk tests` message.
        cmd.env("DOCKER_SCAN_SUGGEST", "false");

        if args.build_args.verbose > 0 {
            let status = cmd.status().map_err(Error::from_source).with_full_context(
                "failed to build Docker image",
                "The build of the Docker image failed which could indicate a configuration problem.",
            )?;

            if !status.success() {
                return Err(Error::new("failed to build Docker image").with_explanation(
                    "The build of the Docker image failed. Check the logs above to determine the cause.",
                ));
            }
        } else {
            let output = cmd.output().map_err(Error::from_source).with_full_context(
                "failed to build Docker image",
                "The build of the Docker image failed which could indicate a configuration problem. You may want to re-run the command with `--verbose` to get more information.",
            )?;

            if !output.status.success() {
                return Err(Error::new("failed to build Docker image")
                    .with_explanation("The build of the Docker image failed. Check the logs below to determine the cause.")
                    .with_output(String::from_utf8_lossy(&output.stderr)));
            };
        }

        Ok(())
    }

    fn registry(&self) -> Result<String> {
        match self.metadata.registry {
            Some(ref registry) => Ok(registry.clone()),
            None => {
                if let Ok(registry) = std::env::var(DEFAULT_DOCKER_REGISTRY_ENV_VAR_NAME) {
                    Ok(registry)
                } else {
                    Err(
                        Error::new("failed to determine Docker registry").with_explanation(
                            format!(
                        "The field registry is empty and the environment variable {} was not set",
                        DEFAULT_DOCKER_REGISTRY_ENV_VAR_NAME
                    ),
                        ),
                    )
                }
            }
        }
    }

    fn docker_image_name(&self) -> Result<String> {
        Ok(format!(
            "{}/{}:{}",
            self.registry()?,
            self.name(),
            self.package.version(),
        ))
    }

    fn get_aws_ecr_information(&self) -> Result<Option<AwsEcrInformation>> {
        Ok(AwsEcrInformation::from_string(&format!(
            "{}/{}",
            self.registry()?,
            self.name(),
        )))
    }

    fn docker_root(&self, ctx: &Context, args: &distrib::Args) -> Result<Utf8PathBuf> {
        target_dir(ctx, &args.build_args).map(|dir| dir.join("docker").join(self.name()))
    }

    fn docker_target_bin_dir(&self, ctx: &Context, args: &distrib::Args) -> Result<Utf8PathBuf> {
        let relative_target_bin_dir = self
            .metadata
            .target_bin_dir
            .strip_prefix("/")
            .unwrap_or(&self.metadata.target_bin_dir);

        self.docker_root(ctx, args)
            .map(|dir| dir.join(relative_target_bin_dir))
    }

    fn write_dockerfile(
        &self,
        ctx: &Context,
        args: &distrib::Args,
        binaries: &HashMap<String, Utf8PathBuf>,
    ) -> Result<Utf8PathBuf> {
        let dockerfile = self.generate_dockerfile(binaries)?;

        debug!("Generated Dockerfile:\n{}", dockerfile);

        let dockerfile_path = self.dockerfile_name(ctx, args)?;
        let dockerfile_root = dockerfile_path.parent();

        std::fs::create_dir_all(dockerfile_root.unwrap())
            .map_err(Error::from_source)
            .with_full_context(
        "could not create Dockerfile path",
        format!("The build process needed to create `{}` but it could not. You may want to verify permissions.", dockerfile_root.unwrap()),
            )?;

        debug!("Writing Dockerfile to: {}", dockerfile_path);

        std::fs::write(&dockerfile_path, dockerfile)
            .map_err(Error::from_source)
            .with_context("failed to write Dockerfile")?;

        Ok(dockerfile_path)
    }

    fn dockerfile_name(&self, ctx: &Context, args: &distrib::Args) -> Result<Utf8PathBuf> {
        self.docker_root(ctx, args)
            .map(|dir| dir.join("Dockerfile"))
    }

    fn generate_context(&self, binaries: &HashMap<String, Utf8PathBuf>) -> TemplateContext {
        let package_name = self.package.name().to_owned();
        let package_version = self.package.version().to_string();

        let binaries: HashMap<_, _> = binaries
            .iter()
            .map(|(name, binary)| {
                (
                    name.clone(),
                    self.metadata
                        .target_bin_dir
                        .join(binary.file_name().unwrap()),
                )
            })
            .collect();

        let extra_files: HashSet<String> = self
            .metadata
            .extra_files
            .iter()
            .map(|cc| cc.destination.to_string())
            .collect();

        let copy_all_binaries = {
            let binaries: Vec<_> = binaries
                .iter()
                .map(|(name, binary)| TemplateBinary {
                    name: name.clone(),
                    binary: binary.clone(),
                })
                .collect();
            let mut tt = TinyTemplate::new();
            tt.add_template(
                "copy_all_binaries",
                "
# Copy all binaries to the Docker image.
{{ for bin in binaries }}
# Copy the binary `{ bin.name }`.
ADD { bin.binary } { bin.binary }
{{ endfor }}
# End of copy.",
            )
            .unwrap();
            tt.render("copy_all_binaries", &TemplateBinaries { binaries })
                .unwrap()
                .trim()
                .to_owned()
        };

        let copy_all_extra_files = {
            let extra_files: Vec<_> = extra_files.iter().cloned().collect();
            let mut tt = TinyTemplate::new();
            tt.add_template(
                "copy_all_extra_files",
                "
# Copy all extra files to the Docker image.
{{ for extra_file in extra_files }}
ADD { extra_file } { extra_file }
{{ endfor }}
# End of copy.",
            )
            .unwrap();
            tt.render("copy_all_extra_files", &TemplateExtraFiles { extra_files })
                .unwrap()
                .trim()
                .to_owned()
        };

        let copy_all = [copy_all_binaries.clone(), copy_all_extra_files.clone()].join("\n");

        TemplateContext {
            package_name,
            package_version,
            binaries,
            extra_files,
            copy_all_binaries,
            copy_all_extra_files,
            copy_all,
        }
    }

    fn generate_dockerfile(&self, binaries: &HashMap<String, Utf8PathBuf>) -> Result<String> {
        let context = self.generate_context(binaries);
        let mut tt = TinyTemplate::new();
        tt.add_template("__template", &self.metadata.template)
        .map_err(Error::from_source).with_full_context(
            "failed to add template",
            "The specified Dockerfile template could not be processed properly, which may indicate a possible syntax error."
        )?;
        tt.render("__template", &context).map_err(Error::from_source).with_full_context(
            "failed to render Dockerfile template",
            "The specified Dockerfile template could not rendered properly, which may indicate a possible syntax error."
        )
    }
}

#[derive(Serialize, Clone)]
struct TemplateBinary {
    name: String,
    binary: Utf8PathBuf,
}

#[derive(Serialize, Clone)]
struct TemplateBinaries {
    binaries: Vec<TemplateBinary>,
}

#[derive(Serialize)]
struct TemplateExtraFiles {
    extra_files: Vec<String>,
}

#[derive(Serialize)]
struct TemplateContext {
    package_name: String,
    package_version: String,
    binaries: HashMap<String, Utf8PathBuf>,
    extra_files: HashSet<String>,
    copy_all_binaries: String,
    copy_all_extra_files: String,
    copy_all: String,
}

struct AwsEcrInformation {
    pub account_id: String,
    pub region: String,
    pub repository_name: String,
}

impl AwsEcrInformation {
    pub fn from_string(input: &str) -> Option<Self> {
        let re =
            Regex::new(r"^(\d+)\.dkr\.ecr\.([a-z0-9-]+).amazonaws.com/([a-zA-Z0-9-_/]+)$").unwrap();

        let captures = re.captures_iter(input).next();

        captures.map(|captures| Self {
            account_id: captures[1].to_string(),
            region: captures[2].to_string(),
            repository_name: captures[3].to_string(),
        })
    }
}

impl Display for AwsEcrInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.dkr.ecr.{}.amazonaws.com/{}",
            self.account_id, self.region, self.repository_name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_ecr_information_valid() {
        let s = "550877636976.dkr.ecr.ca-central-1.amazonaws.com/my/repo-si_tory";
        let info = AwsEcrInformation::from_string(s);

        assert!(info.is_some());
        assert_eq!(info.as_ref().unwrap().account_id, "550877636976");
        assert_eq!(info.as_ref().unwrap().region, "ca-central-1");
        assert_eq!(info.as_ref().unwrap().repository_name, "my/repo-si_tory");
        assert_eq!(info.as_ref().unwrap().to_string(), s);
    }

    #[test]
    fn test_aws_ecr_information_wrong_prefix() {
        let info =
            AwsEcrInformation::from_string("foo.550877636976.dkr.ecr.ca-central-1.amazonaws.com/");

        assert!(info.is_none());
    }

    #[test]
    fn test_aws_ecr_information_wrong_suffix() {
        let info = AwsEcrInformation::from_string(
            "550877636976.dkr.ecr.ca-central-1.amazonaws.com/foo#bar",
        );

        assert!(info.is_none());
    }
}
