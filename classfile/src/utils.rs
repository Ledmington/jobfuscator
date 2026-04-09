#![forbid(unsafe_code)]

use std::{
    env,
    io::Result,
    path::{Path, PathBuf},
};

pub fn absolute_no_symlinks(p: &Path) -> Result<PathBuf> {
    if p.is_absolute() {
        Ok(p.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(p).canonicalize()?)
    }
}
