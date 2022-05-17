use lgn_telemetry_proto::analytics::LogEntry;

pub(crate) trait Searchable<Needle> {
    fn matches(&self, needle: Needle) -> bool;
}

impl<'a, Needles> Searchable<Needles> for LogEntry
where
    Needles: 'a + AsRef<[String]>,
{
    fn matches(&self, needle: Needles) -> bool {
        for needle in needle.as_ref() {
            if self.target.to_lowercase().contains(needle)
                || self.msg.to_lowercase().contains(needle)
            {
                return true;
            }
        }

        false
    }
}
