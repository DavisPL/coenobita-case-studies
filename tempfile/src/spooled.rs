use crate::file::tempfile;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

/// A wrapper for the two states of a `SpooledTempFile`.
#[derive(Debug)]
pub enum SpooledData {
    InMemory(Cursor<Vec<u8>>),
    OnDisk(File),
}

/// An object that behaves like a regular temporary file, but keeps data in
/// memory until it reaches a configured size, at which point the data is
/// written to a temporary file on disk, and further operations use the file
/// on disk.
#[derive(Debug)]
pub struct SpooledTempFile<J: Fn(&OsStr, &OsStr) -> PathBuf> {
    max_size: usize,
    inner: SpooledData,
    join: J
}

/// Create a new spooled temporary file.
///
/// # Security
///
/// This variant is secure/reliable in the presence of a pathological temporary
/// file cleaner.
///
/// # Resource Leaking
///
/// The temporary file will be automatically removed by the OS when the last
/// handle to it is closed. This doesn't rely on Rust destructors being run, so
/// will (almost) never fail to clean up the temporary file.
///
/// # Examples
///
/// ```
/// use tempfile::spooled_tempfile;
/// use std::io::Write;
///
/// let mut file = spooled_tempfile(15);
///
/// writeln!(file, "short line")?;
/// assert!(!file.is_rolled());
///
/// // as a result of this write call, the size of the data will exceed
/// // `max_size` (15), so it will be written to a temporary file on disk,
/// // and the in-memory buffer will be dropped
/// writeln!(file, "marvin gardens")?;
/// assert!(file.is_rolled());
/// # Ok::<(), std::io::Error>(())
/// ```
#[inline]
pub fn spooled_tempfile<J: Fn(&OsStr, &OsStr) -> PathBuf>(max_size: usize, join: J) -> SpooledTempFile<J> {
    SpooledTempFile::new(max_size, join)
}

impl<J: Fn(&OsStr, &OsStr) -> PathBuf> SpooledTempFile<J> {
    #[must_use]
    pub fn new(max_size: usize, join: J) -> SpooledTempFile<J> {
        SpooledTempFile {
            max_size,
            inner: SpooledData::InMemory(Cursor::new(Vec::new())),
            join
        }
    }

    /// Returns true if the file has been rolled over to disk.
    #[must_use]
    pub fn is_rolled(&self) -> bool {
        match self.inner {
            SpooledData::InMemory(_) => false,
            SpooledData::OnDisk(_) => true,
        }
    }

    /// Rolls over to a file on disk, regardless of current size. Does nothing
    /// if already rolled over.
    pub fn roll(&mut self) -> io::Result<()> {
        if !self.is_rolled() {
            let mut file = tempfile(&self.join)?;
            if let SpooledData::InMemory(cursor) = &mut self.inner {
                file.write_all(cursor.get_ref())?;
                file.seek(SeekFrom::Start(cursor.position()))?;
            }
            self.inner = SpooledData::OnDisk(file);
        }
        Ok(())
    }

    pub fn set_len(&mut self, size: u64) -> Result<(), io::Error> {
        if size > self.max_size as u64 {
            self.roll()?; // does nothing if already rolled over
        }
        match &mut self.inner {
            SpooledData::InMemory(cursor) => {
                cursor.get_mut().resize(size as usize, 0);
                Ok(())
            }
            SpooledData::OnDisk(file) => file.set_len(size),
        }
    }

    /// Consumes and returns the inner `SpooledData` type.
    #[must_use]
    pub fn into_inner(self) -> SpooledData {
        self.inner
    }
}

impl<J: Fn(&OsStr, &OsStr) -> PathBuf> Read for SpooledTempFile<J> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match &mut self.inner {
            SpooledData::InMemory(cursor) => cursor.read(buf),
            SpooledData::OnDisk(file) => file.read(buf),
        }
    }

    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        match &mut self.inner {
            SpooledData::InMemory(cursor) => cursor.read_vectored(bufs),
            SpooledData::OnDisk(file) => file.read_vectored(bufs),
        }
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        match &mut self.inner {
            SpooledData::InMemory(cursor) => cursor.read_to_end(buf),
            SpooledData::OnDisk(file) => file.read_to_end(buf),
        }
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        match &mut self.inner {
            SpooledData::InMemory(cursor) => cursor.read_to_string(buf),
            SpooledData::OnDisk(file) => file.read_to_string(buf),
        }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        match &mut self.inner {
            SpooledData::InMemory(cursor) => cursor.read_exact(buf),
            SpooledData::OnDisk(file) => file.read_exact(buf),
        }
    }
}

impl<J: Fn(&OsStr, &OsStr) -> PathBuf> Write for SpooledTempFile<J> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // roll over to file if necessary
        if matches! {
            &self.inner, SpooledData::InMemory(cursor)
            if cursor.position().saturating_add(buf.len() as u64) > self.max_size as u64
        } {
            self.roll()?;
        }

        // write the bytes
        match &mut self.inner {
            SpooledData::InMemory(cursor) => cursor.write(buf),
            SpooledData::OnDisk(file) => file.write(buf),
        }
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        if matches! {
            &self.inner, SpooledData::InMemory(cursor)
            // Borrowed from the rust standard library.
            if bufs
                .iter()
                .fold(cursor.position(), |a, b| a.saturating_add(b.len() as u64))
                > self.max_size as u64
        } {
            self.roll()?;
        }
        match &mut self.inner {
            SpooledData::InMemory(cursor) => cursor.write_vectored(bufs),
            SpooledData::OnDisk(file) => file.write_vectored(bufs),
        }
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        match &mut self.inner {
            SpooledData::InMemory(cursor) => cursor.flush(),
            SpooledData::OnDisk(file) => file.flush(),
        }
    }
}

impl<J: Fn(&OsStr, &OsStr) -> PathBuf> Seek for SpooledTempFile<J> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match &mut self.inner {
            SpooledData::InMemory(cursor) => cursor.seek(pos),
            SpooledData::OnDisk(file) => file.seek(pos),
        }
    }
}
