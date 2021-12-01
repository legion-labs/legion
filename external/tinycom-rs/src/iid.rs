use std::fmt;

/// Interface identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct IID {
    /// First component, 32-bit value.
    pub data1: u32,
    /// Second component, 16-bit value.
    pub data2: u16,
    /// Third component, 16-bit value.
    pub data3: u16,
    /// Fourth component, array of 8-bit values.
    pub data4: [u8; 8],
}

/// Print IID in Windows registry format {XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}.
impl fmt::Display for IID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{{:08X}-{:04X}-{:04X}-{:02X}{:02X}-\
                   {:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
            self.data1,
            self.data2,
            self.data3,
            self.data4[0],
            self.data4[1],
            self.data4[2],
            self.data4[3],
            self.data4[4],
            self.data4[5],
            self.data4[6],
            self.data4[7]
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{ComInterface, IUnknown};

    #[test]
    fn iid_display() {
        assert_eq!(
            IUnknown::iid().to_string(),
            "{00000000-0000-0000-C000-000000000046}"
        );
    }
}
