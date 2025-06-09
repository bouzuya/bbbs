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
