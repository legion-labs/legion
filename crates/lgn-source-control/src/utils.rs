use anyhow::{Context, Result};
use lgn_tracing::imetric;
use std::cell::RefCell;
use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use url::Url;

pub(crate) enum UrlOrPath {
    Url(Url),
    Path(PathBuf),
}

/// Parse an URL or a raw path and returns it.
///
/// If the URL is a local file path, it is converted to a path.
pub(crate) fn parse_url_or_path(s: &str) -> Result<UrlOrPath> {
    let validation = RefCell::new(None);

    let violation_cb = |violation| {
        *validation.borrow_mut() = Some(violation);
    };

    let options = Url::options().syntax_violation_callback(Some(&violation_cb));

    let result = options.parse(s);
    let validation = *validation.borrow();

    match (result, validation) {
        (_, Some(url::SyntaxViolation::ExpectedFileDoubleSlash)) => {
            Err(anyhow::anyhow!("expected file://"))
        }
        (Ok(url), _) => match url.scheme() {
            "file" => {
                Ok(UrlOrPath::Path(url.to_file_path().map_err(|_err| {
                    anyhow::anyhow!("failed to convert URL to path")
                })?))
            }
            scheme => {
                // Assume 1 character schemes are Windows drives.
                if scheme.len() == 1 {
                    Ok(UrlOrPath::Path(s.into()))
                } else {
                    Ok(UrlOrPath::Url(url))
                }
            }
        },
        (Err(_), Some(validation)) => Err(anyhow::anyhow!("{}", validation)),
        (Err(_), None) => Ok(UrlOrPath::Path(s.into())),
    }
}

pub(crate) fn write_file(path: &Path, contents: &[u8]) -> Result<()> {
    fs::create_dir_all(path.parent().unwrap())
        .context(format!("error creating directory: {}", path.display()))?;

    let mut file =
        fs::File::create(path).context(format!("error creating file: {}", path.display()))?;

    file.write_all(contents)
        .context(format!("error writing: {}", path.display()))
}

pub(crate) fn read_text_file(path: &Path) -> Result<String> {
    let contents =
        fs::read_to_string(path).context(format!("error reading file: {}", path.display()))?;
    imetric!("read file size", "bytes", contents.len() as u64);
    Ok(contents)
}

pub(crate) fn read_bin_file(path: &Path) -> Result<Vec<u8>> {
    let contents = fs::read(path).context(format!("error reading file {}", path.display()))?;
    imetric!("read file size", "bytes", contents.len() as u64);
    Ok(contents)
}

pub(crate) fn make_path_absolute(path: impl AsRef<Path>) -> Result<PathBuf> {
    //fs::canonicalize is a trap - it generates crazy unusable "extended length" paths
    let path = path.as_ref();

    Ok(if path.is_absolute() {
        PathBuf::from(path_clean::clean(
            path.to_str().context("failed to convert path to string")?,
        ))
    } else {
        PathBuf::from(path_clean::clean(
            std::env::current_dir()
                .context("failed to get current directory")?
                .join(path)
                .to_str()
                .context("failed to convert path to string")?,
        ))
    })
}

pub(crate) fn path_relative_to(p: &Path, base: &Path) -> Result<PathBuf> {
    p.strip_prefix(base).map(Path::to_path_buf).context(format!(
        "error stripping prefix: {} is not relative to {}",
        p.display(),
        base.display()
    ))
}

pub(crate) fn make_canonical_relative_path(
    workspace_root: &Path,
    path_specified: impl AsRef<Path>,
) -> Result<String> {
    let abs_path = make_path_absolute(path_specified)?;
    let relative_path = path_relative_to(&abs_path, workspace_root)?;
    let canonical_relative_path = relative_path.to_str().unwrap().replace("\\", "/");
    Ok(canonical_relative_path)
}

pub(crate) fn make_file_read_only(file_path: &Path, readonly: bool) -> Result<()> {
    let meta = fs::metadata(&file_path)
        .context(format!("error reading metadata: {}", file_path.display()))?;

    let mut permissions = meta.permissions();
    permissions.set_readonly(readonly);

    fs::set_permissions(&file_path, permissions).context(format!(
        "error setting permissions: {}",
        file_path.display()
    ))
}
