#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Version(u32);

impl Version {
    pub fn initial() -> Self {
        Self(1)
    }

    pub fn next(&self) -> Self {
        // TODO: overflow
        Self(self.0 + 1)
    }
}

impl From<Version> for u32 {
    fn from(value: Version) -> Self {
        value.0
    }
}
