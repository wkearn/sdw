//! Lockers for sonar data
use crate::model::{Channel, SonarDataRecord};
use crate::parser::jsf;
use binrw::{io::BufReader, BinRead};
use std::collections::{btree_map, BTreeMap};
use std::fs::{read_dir, File};
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};
use time::OffsetDateTime;

use std::sync::mpsc;
use std::thread;

type LockerKey = (String, OffsetDateTime, Channel);
type LockerValue = (PathBuf, u64);

/// A representation of an on-disk sonar data set
///
/// A `Locker` contains an in-memory [`BTreeMap`] index that maps
/// keys to a file path and byte offset within that file where the desired
/// record can be found.
/// Keys are a tuple consisting of a string representation of the [`SonarDataRecord`]
/// enum variant, an [`OffsetDateTime`] representing the acquisition time
/// of the measurement and a [`Channel`]. Due to this key organization,
/// queries such as finding all `SonarDataRecord::Ping` records from the
/// `Channel::Port` between two times are fast. The channel is after the time
/// because it is assumed that typical applications (i.e. mosaicking) will want to process
/// starboard and port pings simultaneously.
///
/// The channel key only has meaning for the sonar data (`SonarDataRecord::Ping`). All
/// other records default to `Channel::Other`.
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
    ///
    /// This scans through every file and creates an entry in
    /// the locker tree for each record. This can take a while.
    ///
    /// # Errors
    ///
    /// This function returns an error when `read_dir` errors
    pub fn open<P>(path: P) -> binrw::BinResult<Self>
    where
        PathBuf: From<P>,
    {
        let tree = BTreeMap::new();
        let path = PathBuf::from(path);

        let mut locker = Locker { path, tree };

        locker.build_index()?;

        Ok(locker)
    }

    /// Scan the `Locker` directory to build the index
    ///
    /// This will clear the current index and rescan all of the files in
    /// the directory.
    pub fn build_index(&mut self) -> binrw::BinResult<()> {
        // Clear the tree
        self.tree.clear();

        let dir = read_dir(&self.path)?;

        // Open a channel for storing key-value pairs read out of the files
        let (tx, rx) = mpsc::channel();

        for entry in dir {
            let tx1 = tx.clone();
            thread::spawn(move || -> binrw::BinResult<()> {
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
                    tx1.send((key, value)).map_err(|_| {
                        std::io::Error::new(std::io::ErrorKind::Other, "Channel sending error")
                    })?;
                }
                Ok(())
            });
        }

        // Explicitly drop the Sender to close the channel
        drop(tx);

        // Read pairs off the channel and insert them in the
        // tree
        for rcv in rx {
            let (key, value) = rcv;
            if let Some(key) = key {
                self.tree.insert(key, value);
            };
        }

        Ok(())
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

    /// Return a reference to the underlying [`BTreeMap`]
    pub fn tree(&self) -> &BTreeMap<LockerKey, LockerValue> {
        &self.tree
    }

    /// Get an iterator over the entries of the locker, sorted by key
    pub fn iter(&self) -> Iter {
        let iter = self.tree.iter();
        Iter { iter }
    }

    /// Get the SonarDataRecord identified by the key
    ///
    /// ```
    /// # use sdw::locker::{create_key,Locker};
    /// # fn get_test() -> Result<(), Box<dyn std::error::Error>> {
    /// let locker = Locker::open("assets/HE501")?;
    /// let (k, _) = locker.tree().first_key_value().ok_or(std::io::Error::new(
    ///     std::io::ErrorKind::Other,
    ///     "Key not found",
    /// ))?;
    /// let rec = locker.get(k)?;    
    /// let c = create_key(rec).ok_or(std::io::Error::new(
    ///         std::io::ErrorKind::Other,
    ///         "Unknown record retrieved",
    ///         ))?;

    /// assert_eq!(c.0, k.0);
    /// assert_eq!(c.1, k.1);
    /// assert_eq!(c.2, k.2);
    /// # Ok(()) }
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
        SonarDataRecord::Ping(data) => Some(("Ping".to_string(), data.timestamp, data.channel)),
        SonarDataRecord::Course(data) => {
            Some(("Course".to_string(), data.timestamp, Channel::default()))
        }
        SonarDataRecord::Position(data) => {
            Some(("Position".to_string(), data.timestamp, Channel::default()))
        }
        SonarDataRecord::Orientation(data) => Some((
            "Orientation".to_string(),
            data.timestamp,
            Channel::default(),
        )),
        SonarDataRecord::Unknown => None,
    }
}

/// An iterator over the entries of the locker
///
/// This should be created by calling `iter` on a `Locker`.
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
