//! Lockers for sonar data
use std::fs::{read_dir, ReadDir};
use std::io;
use std::path::{Path, PathBuf};

/// A representation of an on-disk sonar data set
pub struct Locker {
    path: PathBuf,
    dir: ReadDir,
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
    /// # Errors
    ///
    /// This function returns an error when `read_dir` errors
    pub fn open<P>(path: P) -> io::Result<Self>
    where
        PathBuf: From<P>,
    {
        let path = PathBuf::from(path);
        let dir = read_dir(&path)?;
        Ok(Locker { path, dir })
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

    /// Return the directory iterator
    ///
    /// ```
    /// # use sdw::locker::Locker;
    /// # use std::path::Path;
    /// # fn main() -> Result<(),Box<dyn std::error::Error>> {
    /// let locker = Locker::open("/home/wkearn/Documents/data/PANGAEA/HE501")?;
    /// assert_eq!(122,locker.dir().count());
    /// # Ok(()) }
    /// ```
    pub fn dir(self) -> ReadDir {
        self.dir
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
