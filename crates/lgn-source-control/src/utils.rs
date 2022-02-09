use anyhow::{Context, Result};
use std::cell::RefCell;
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
