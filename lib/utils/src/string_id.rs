use lazy_static::lazy_static;
use std::{collections::HashMap, sync::Mutex};

lazy_static! {
    static ref DICTIONARY: Mutex<HashMap<StringId, String>> = Mutex::new(HashMap::<_, _>::new());
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct StringId(u32);

impl StringId {
    const CRC32_ALGO: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_CKSUM);

    pub fn from_raw(id: u32) -> Self {
        Self(id)
    }
    pub fn new(name: &'static str) -> Self {
        let id = Self::compute_sid(name);
        let out = DICTIONARY.lock().unwrap().insert(id, name.to_owned());
        assert!(out.is_none() || out.unwrap() == name);
        id
    }

    pub fn lookup_name(sid: Self) -> Option<String> {
        DICTIONARY.lock().unwrap().get(&sid).cloned()
    }

    pub const fn compute_sid(name: &'static str) -> Self {
        let v = Self::CRC32_ALGO.checksum(name.as_bytes());
        Self(v)
    }
}

#[cfg(test)]
mod tests {
    use super::StringId;

    #[test]
    fn test() {
        let raw = StringId::from_raw(2357529937); // "hello world"

        assert!(StringId::lookup_name(raw).is_none());

        let sid = StringId::new("hello world");
        assert_eq!(StringId::lookup_name(sid).unwrap().as_str(), "hello world");

        assert_eq!(StringId::lookup_name(raw).unwrap().as_str(), "hello world");
    }
}
