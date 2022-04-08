use std::{fmt::Display, path::PathBuf};

use serde::{Deserialize, Serialize};

/// The origin for a content.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Origin {
    AwsS3 {
        bucket_name: String,
        key: String,
    },
    AwsDynamoDb {
        region: String,
        table_name: String,
        id: String,
    },
    Redis {
        host: String,
        key: String,
    },
    Memory {},
    Lru {},
    Local {
        path: PathBuf,
    },
    InIdentifier {},
}

impl Origin {
    pub fn name(&self) -> &str {
        match self {
            Self::AwsS3 { .. } => "AWS S3",
            Self::AwsDynamoDb { .. } => "AWS DynamoDB",
            Self::Redis { .. } => "Redis",
            Self::Memory { .. } => "a memory cache",
            Self::Lru { .. } => "a LRU cache",
            Self::Local { .. } => "a local file",
            Self::InIdentifier { .. } => "the identifier",
        }
    }
}

impl Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Origin::AwsS3 {
                bucket_name: bucket,
                key,
            } => write!(f, "s3://{}/{}", bucket, key),
            Origin::AwsDynamoDb {
                region,
                table_name,
                id,
            } => write!(f, "dynamodb://{}/{}/{}", region, table_name, id),
            Origin::Redis { host, key } => write!(f, "redis://{}/{}", host, key),
            Origin::Memory {} => write!(f, "in-memory-cache"),
            Origin::Lru {} => write!(f, "in-lru-cache"),
            Origin::Local { path } => write!(f, "{}", path.display()),
            Origin::InIdentifier {} => write!(f, "in-identifier"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Origin;

    #[test]
    fn test_origin_to_string() {
        assert_eq!(
            Origin::AwsS3 {
                bucket_name: "my-bucket".to_string(),
                key: "some/key".to_string()
            }
            .to_string(),
            "s3://my-bucket/some/key"
        );
        assert_eq!(
            Origin::AwsDynamoDb {
                region: "ca-central-1".to_string(),
                table_name: "legionlabs-content-store".to_string(),
                id: "my-id".to_string(),
            }
            .to_string(),
            "dynamodb://ca-central-1/legionlabs-content-store/my-id"
        );
        assert_eq!(
            Origin::Redis {
                host: "my-host:123".to_string(),
                key: "some:key".to_string(),
            }
            .to_string(),
            "redis://my-host:123/some:key"
        );
        assert_eq!(Origin::Memory {}.to_string(), "in-memory-cache");
        assert_eq!(Origin::Lru {}.to_string(), "in-lru-cache");
        assert_eq!(
            Origin::Local {
                path: "some/path".into()
            }
            .to_string(),
            "some/path"
        );
        assert_eq!(Origin::InIdentifier {}.to_string(), "in-identifier");
    }
}
