//! Lockers for sonar data
use crate::model::{Channel, SonarDataRecord};
use crate::parser::jsf;
use binrw::{io::BufReader, BinRead};
use std::collections::{btree_map, BTreeMap};
use std::fs::{read_dir, File};
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};
use time::OffsetDateTime;

type LockerKey = (String, Channel, OffsetDateTime);
type LockerValue = (PathBuf, u64);

/// A representation of an on-disk sonar data set
pub struct Locker {
    path: PathBuf,
    tree: BTreeMap<LockerKey, LockerValue>,
}

impl Locker {
    /// Open a locker at the given path
    ///
    /// ```
    /// # use sdw::locker::Locker;
    /// # fn main() -> Result<(),Box<dyn std::error::Error>> {
    /// let locker = Locker::open("assets/HE501")?;
    /// # Ok(()) }
    /// ```
    /// # Errors
    ///
    /// This function returns an error when `read_dir` errors
    pub fn open<P>(path: P) -> binrw::BinResult<Self>
    where
        PathBuf: From<P>,
    {
        let mut tree = BTreeMap::new();
        let path = PathBuf::from(path);
        let dir = read_dir(&path)?;
        for entry in dir {
            let filepath = entry?.path();
            let reader = BufReader::new(File::open(&filepath)?);
            let mut jsf = jsf::File::new(reader);
            loop {
                let pos = jsf.stream_position()?;
                let msg = match jsf.next() {
                    Some(val) => val,
                    None => break,
                };
                let key = create_key(SonarDataRecord::from(msg?));
                let value = (filepath.clone(), pos);
                match key {
                    Some(k) => tree.insert(k, value),
                    None => None,
                };
            }
        }
        Ok(Locker { path, tree })
    }

    /// Return a reference to the path of the locker
    ///
    /// ```
    /// # use sdw::locker::Locker;
    /// # use std::path::Path;
    /// # fn main() -> Result<(),Box<dyn std::error::Error>> {
    /// let locker = Locker::open("assets/HE501")?;
    /// assert_eq!(locker.path(),Path::new("assets/HE501"));
    /// # Ok(()) }
    /// ```
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Return a reference to the underlying BTreeMap
    pub fn tree(&self) -> &BTreeMap<LockerKey, LockerValue> {
        &self.tree
    }

    /// Get an iterator over the entries of the B-tree, sorted by key
    pub fn iter(&self) -> Iter {
        let iter = self.tree.iter();
        Iter { iter }
    }

    /// Get the SonarDataRecord identified by the key
    ///
    /// ```
    /// # use sdw::locker::{create_key,Locker};
    /// # fn get_test() -> Result<(), Box<dyn std::error::Error>> {
    ///     let locker = Locker::open("assets/HE501")?;
    ///     let (k, _) = locker.tree().first_key_value().ok_or(std::io::Error::new(
    ///         std::io::ErrorKind::Other,
    ///         "Key not found",
    ///     ))?;
    ///     let rec = locker.get(k)?;    
    ///     let c = create_key(rec).ok_or(std::io::Error::new(
    ///             std::io::ErrorKind::Other,
    ///             "Unknown record retrieved",
    ///             ))?;

    ///     assert_eq!(c.0, k.0);
    ///     assert_eq!(c.1, k.1);
    ///     assert_eq!(c.2, k.2);
    /// #    Ok(()) }
    /// ```
    /// # Errors
    ///
    /// This method returns an error if the key is not found in the index tree or
    /// if there is an error reading the record from the file.
    pub fn get(&self, key: &LockerKey) -> binrw::BinResult<SonarDataRecord<u16>> {
        let (path, offset) = self.tree.get(key).ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Key not found",
        ))?;
        let mut f = File::open(path)?;
        f.seek(SeekFrom::Start(*offset))?;
        let msg = jsf::Message::read(&mut f)?;
        Ok(SonarDataRecord::from(msg))
    }
}

/// Create a LockerKey from a SonarDataRecord
///
/// Returns `None` if the rec is `SonarDataRecord::Unknown`, otherwise returns
/// `Some(key)` with an appropriately formatted key.
pub fn create_key<T>(rec: SonarDataRecord<T>) -> Option<LockerKey> {
    match rec {
        SonarDataRecord::Ping(data) => Some(("Ping".to_string(), data.channel, data.timestamp)),
        SonarDataRecord::Course(data) => {
            Some(("Course".to_string(), Channel::Other, data.timestamp))
        }
        SonarDataRecord::Position(data) => {
            Some(("Position".to_string(), Channel::Other, data.timestamp))
        }
        SonarDataRecord::Orientation(data) => {
            Some(("Orientation".to_string(), Channel::Other, data.timestamp))
        }
        SonarDataRecord::Unknown => None,
    }
}

/// An iterator over the entries of the BTree
///
/// This is created by calling `iter` on a `Locker`.
///
/// ```
/// # use sdw::locker::Locker;
/// # use std::path::Path;
/// # fn main() -> Result<(),Box<dyn std::error::Error>> {
/// let locker = Locker::open("assets/HE501")?;
/// let c = locker.iter().count();
/// assert_eq!(436719,c);
/// # Ok(()) }
/// ```
pub struct Iter<'a> {
    iter: btree_map::Iter<'a, LockerKey, LockerValue>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a LockerKey, &'a LockerValue);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {}

#[cfg(test)]
mod test {}
