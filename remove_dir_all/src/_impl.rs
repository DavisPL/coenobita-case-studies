use std::{
    ffi::OsStr,
    fs::File,
    io::{ErrorKind, Result},
    path::Path,
};

#[cfg(windows)]
use fs_at::os::windows::{FileExt, OpenOptionsExt};
#[cfg(feature = "parallel")]
use rayon::prelude::*;
#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::{DELETE, FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES};

mod io;
mod path_components;

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        mod win;
        pub(crate) use win::WindowsIo as OsIo;
    } else {
        mod unix;
        pub(crate) use unix::UnixIo as OsIo;
    }
}

impl super::RemoveDir for std::fs::File {
    fn remove_dir_contents<G: Fn(&OsStr) -> &Path>(&mut self, debug_root: Option<&Path>, generate: G) -> Result<()> {
        // thunk over to the free version adding in the os-specific IO trait impl
        let debug_root = match debug_root {
            None => PathComponents::Path(generate("".as_ref())),
            Some(debug_root) => PathComponents::Path(debug_root),
        };
        _remove_dir_contents::<OsIo, _>(self, &debug_root, generate)
    }
}

/// Entry point for deprecated function
pub(crate) fn _ensure_empty_dir_path<I: io::Io, P: AsRef<Path>, G: Fn(&OsStr) -> &Path>(path: P, generate: G) -> Result<()> {
    // This is as TOCTOU safe as we can make it. Attacks via link replacements
    // in interior components of the path is still possible. if the create
    // succeeds, mission accomplished. if the create fails, open the dir
    // (subject to races again), and then proceed to delete the contents via the
    // descriptor.
    match std::fs::create_dir(&path) {
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            // Exists and is a dir. Open it
            let mut existing_dir = I::open_dir(path.as_ref())?;
            existing_dir.remove_dir_contents(Some(path.as_ref()), generate)
        }
        otherwise => otherwise,
    }
}

// Deprecated entry point
pub(crate) fn _remove_dir_contents_path<I: io::Io, P: AsRef<Path>, G: Fn(&OsStr) -> &Path>(path: P, generate: G) -> Result<()> {
    let mut d = I::open_dir(path.as_ref())?;
    _remove_dir_contents::<I, _>(&mut d, &PathComponents::Path(path.as_ref()), generate)
}

/// exterior lifetime interface to dir removal
fn _remove_dir_contents<I: io::Io, G: Fn(&OsStr) -> &Path>(d: &mut File, debug_root: &PathComponents<'_>, generate: G) -> Result<()> {
    let owned_handle = I::duplicate_fd(d)?;
    remove_dir_contents_recursive::<I, _>(owned_handle, debug_root, &generate)?;
    Ok(())
}

/// deprecated interface
pub(crate) fn remove_dir_all_path<I: io::Io, P: AsRef<Path>, G: Fn(&OsStr) -> &Path>(path: P, generate: G) -> Result<()> {
    let p = path.as_ref();
    // Opportunity 1 for races
    let d = I::open_dir(p)?;
    let debug_root = PathComponents::Path(if p.has_root() { p } else { generate(".".as_ref()) });
    remove_dir_contents_recursive::<OsIo, _>(d, &debug_root, &generate)?;
    // Opportunity 2 for races
    std::fs::remove_dir(&path)?;
    #[cfg(feature = "log")]
    log::trace!("removed {}", &debug_root);
    Ok(())
}

use crate::RemoveDir;

use self::path_components::PathComponents;

// Core workhorse, heading towards this being able to be tasks.
#[allow(clippy::map_identity)]
fn remove_dir_contents_recursive<I: io::Io, G: Fn(&OsStr) -> &Path>(
    mut d: File,
    debug_root: &PathComponents<'_>,
    generate: &G
) -> Result<File> {
    #[cfg(feature = "log")]
    log::trace!("scanning {}", &debug_root);
    // We take a os level clone of the FD so that there are no lifetime
    // concerns. It would *not* be ok to do readdir on one file twice
    // concurrently because of shared kernel state.
    let dirfd = I::duplicate_fd(&mut d)?;
    cfg_if::cfg_if! {
        if #[cfg(feature = "parallel")] {
            let iter = fs_at::read_dir(&mut d)?;
            let iter = iter.par_bridge();
        } else {
            let mut iter = fs_at::read_dir(&mut d)?;
        }
    }

    iter.try_for_each(|dir_entry| -> Result<()> {
        let dir_entry = dir_entry?;
        let name = dir_entry.name();
        if name == OsStr::new(".") || name == OsStr::new("..") {
            return Ok(());
        }
        let dir_path = generate(name);
        let dir_debug_root = PathComponents::Component(debug_root, dir_path);
        #[cfg(windows)]
        {
            // On windows: open the file and then decide what to do with it.
            let mut opts = fs_at::OpenOptions::default();
            // Could possibly drop a syscall by dropping FILE_READ_ATTRIBUTES
            // and trusting read_dir metadata more. OTOH that would introduce a
            // race :/.
            opts.desired_access(DELETE | FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES);
            let mut child_file = opts.open_path_at(&dirfd, name)?;
            let metadata = child_file.metadata()?;
            let is_dir = metadata.is_dir();
            let is_symlink = metadata.is_symlink();
            if is_dir && !is_symlink {
                remove_dir_contents_recursive::<I>(
                    I::duplicate_fd(&mut child_file)?,
                    &dir_debug_root,
                )?;
            }
            #[cfg(feature = "log")]
            log::trace!("delete: {}", &dir_debug_root);
            child_file.delete_by_handle().map_err(|(_f, e)| {
                #[cfg(feature = "log")]
                log::debug!("error removing {}", dir_debug_root);
                e
            })?;
        }
        #[cfg(not(windows))]
        {
            // Otherwise, open the path safely but normally, fstat to see if its
            // a dir, then either unlink or recursively delete
            let mut opts = fs_at::OpenOptions::default();
            opts.read(true)
                .write(fs_at::OpenOptionsWriteMode::Write)
                .follow(false);
            let child_result = opts.open_dir_at(&dirfd, generate(name));
            let is_dir = match child_result {
                // We expect is_eloop to be the only error
                Err(e) if !I::is_eloop(&e) => return Err(e),
                Err(_) => false,
                Ok(child_file) => {
                    let metadata = child_file.metadata()?;
                    let is_dir = metadata.is_dir();
                    if is_dir {
                        remove_dir_contents_recursive::<I, _>(child_file, &dir_debug_root, generate)?;
                        #[cfg(feature = "log")]
                        log::trace!("rmdir: {}", &dir_debug_root);
                        let opts = fs_at::OpenOptions::default();
                        opts.rmdir_at(&dirfd, generate(name)).map_err(|e| {
                            #[cfg(feature = "log")]
                            log::debug!("error removing {}", dir_debug_root);
                            e
                        })?;
                    }
                    is_dir
                }
            };
            if !is_dir {
                #[cfg(feature = "log")]
                log::trace!("unlink: {}", &dir_debug_root);
                opts.unlink_at(&dirfd, generate(name)).map_err(|e| {
                    #[cfg(feature = "log")]
                    log::debug!("error removing {}", dir_debug_root);
                    e
                })?;
            }
        }
        #[cfg(feature = "log")]
        log::trace!("removed {}", dir_debug_root);

        Ok(())
    })?;
    #[cfg(feature = "log")]
    log::trace!("scanned {}", &debug_root);
    Ok(dirfd)
}
