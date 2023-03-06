//! Lockers for sonar data
use std::path::{Path, PathBuf};

/// A representation of an on-disk sonar data set
pub struct Locker {
    path: PathBuf,
}

impl Locker {
    /// Open a locker at the given path
    pub fn open<P>(path: P) -> Self
    where
        PathBuf: From<P>,
    {
        Locker {
            path: PathBuf::from(path),
        }
    }

    /// Return a reference to the path of the locker
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_locker() {
        let locker = Locker::open("/home/wkearn/Documents/data/PANGAEA/HE501");
        assert_eq!(
            locker.path(),
            Path::new("/home/wkearn/Documents/data/PANGAEA/HE501")
        )
    }
}
