//! Lockers for sonar data
use std::io;
use std::path::{Path, PathBuf};

/// A representation of an on-disk sonar data set
pub struct Locker {
    path: PathBuf,
}

impl Locker {
    /// Open a locker at the given path
    ///
    /// ```
    /// # use sdw::locker::Locker;
    /// # fn main() -> Result<(),Box<dyn std::error::Error>> {
    /// let locker = Locker::open("/home/wkearn/Documents/data/PANGAEA/HE501")?;
    /// # Ok(()) }
    /// ```
    pub fn open<P>(path: P) -> io::Result<Self>
    where
        PathBuf: From<P>,
    {
        Ok(Locker {
            path: PathBuf::from(path),
        })
    }

    /// Return a reference to the path of the locker
    ///
    /// ```
    /// # use sdw::locker::Locker;
    /// # use std::path::Path;
    /// # fn main() -> Result<(),Box<dyn std::error::Error>> {
    /// let locker = Locker::open("/home/wkearn/Documents/data/PANGAEA/HE501")?;
    /// assert_eq!(locker.path(),Path::new("/home/wkearn/Documents/data/PANGAEA/HE501"));
    /// # Ok(()) }
    /// ```
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
