#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ModuleVersion {
    major: usize,
    minor: usize,
    patch: usize,
}

impl From<(usize, usize, usize)> for ModuleVersion {
    fn from((major, minor, patch): (usize, usize, usize)) -> Self {
        Self::new(major, minor, patch)
    }
}

impl From<(usize, usize)> for ModuleVersion {
    fn from((major, minor): (usize, usize)) -> Self {
        Self::new(major, minor, 0)
    }
}

impl From<(usize,)> for ModuleVersion {
    fn from((major,): (usize,)) -> Self {
        Self::new(major, 0, 0)
    }
}

impl ModuleVersion {
    pub fn new(major: usize, minor: usize, patch: usize) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn major(&self) -> usize {
        self.major
    }

    pub fn minor(&self) -> usize {
        self.minor
    }

    pub fn patch(&self) -> usize {
        self.patch
    }
}
