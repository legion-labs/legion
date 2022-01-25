/// A change type for a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Edit = 1,
    Add = 2,
    Delete = 3,
}

impl From<ChangeType> for lgn_source_control_proto::ChangeType {
    fn from(change_type: ChangeType) -> Self {
        match change_type {
            ChangeType::Edit => Self::Edit,
            ChangeType::Add => Self::Add,
            ChangeType::Delete => Self::Delete,
        }
    }
}

impl From<lgn_source_control_proto::ChangeType> for ChangeType {
    fn from(change_type: lgn_source_control_proto::ChangeType) -> Self {
        match change_type {
            lgn_source_control_proto::ChangeType::Edit => Self::Edit,
            lgn_source_control_proto::ChangeType::Add => Self::Add,
            lgn_source_control_proto::ChangeType::Delete => Self::Delete,
        }
    }
}

impl ChangeType {
    pub fn from_int(i: i64) -> anyhow::Result<Self> {
        match i {
            1 => Ok(Self::Edit),
            2 => Ok(Self::Add),
            3 => Ok(Self::Delete),
            _ => anyhow::bail!("invalid change type {}", i),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_type_from_int() {
        let i = 1;

        let change_type = ChangeType::from_int(i).unwrap();

        assert_eq!(change_type, ChangeType::Edit);
    }

    #[test]
    fn test_change_type_from_int_invalid() {
        let i = 4;

        let result = ChangeType::from_int(i);

        assert!(result.is_err());
    }

    #[test]
    fn test_change_type_from_proto() {
        let proto = lgn_source_control_proto::ChangeType::Edit;

        let change_type = ChangeType::from(proto);

        assert_eq!(change_type, ChangeType::Edit);
    }

    #[test]
    fn test_change_type_into_proto() {
        let change_type = ChangeType::Edit;

        let proto: lgn_source_control_proto::ChangeType = change_type.into();

        assert_eq!(proto, lgn_source_control_proto::ChangeType::Edit);
    }
}
