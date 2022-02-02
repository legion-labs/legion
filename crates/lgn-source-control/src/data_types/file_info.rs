#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileInfo {
    pub hash: String,
    pub size: u64,
}

impl From<FileInfo> for lgn_source_control_proto::FileInfo {
    fn from(file_info: FileInfo) -> Self {
        Self {
            hash: file_info.hash,
            size: file_info.size,
        }
    }
}

impl From<lgn_source_control_proto::FileInfo> for FileInfo {
    fn from(file_info: lgn_source_control_proto::FileInfo) -> Self {
        Self {
            hash: file_info.hash,
            size: file_info.size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fi(hash: &str, size: u64) -> FileInfo {
        FileInfo {
            hash: hash.to_string(),
            size,
        }
    }

    #[test]
    fn test_file_info_comparison() {
        assert!(fi("abc", 123) == fi("abc", 123));
        assert!(fi("abc", 123) <= fi("abc", 123));
        assert!(fi("abc", 123) <= fi("abd", 100));
        assert!(fi("abc", 123) < fi("abd", 100));
    }
}
