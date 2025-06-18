use anyhow::{Context, Result};
use byteorder::{LittleEndian, WriteBytesExt};
use rayon::prelude::*;
use std::fs;
use std::io::{BufReader, Cursor, Read};
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::Path;

use crate::config::Config;

pub fn hash_entry(config: &Config, path: &Path) -> Result<[u8; 32]> {
    let meta = std::fs::symlink_metadata(path)?;
    let filetype = meta.file_type();
    let mut hasher = config.hasher();
    hasher.update(&hash_meta(config, &meta)?);

    if filetype.is_dir() {
        hasher.update(&hash_dir(config, path)?);
    } else if filetype.is_file() {
        config.stats.add_bytes(meta.size());
        if config.include_file_content {
            hasher.update(&hash_file(config, path)?);
        }
    } else if filetype.is_symlink() {
        let target = fs::read_link(path)?;
        hasher.update(target.as_os_str().as_encoded_bytes());
    } else if filetype.is_block_device() || filetype.is_char_device() {
        let rdev = meta.rdev();
        hasher.update(&rdev.to_le_bytes());
    } else if filetype.is_fifo() || filetype.is_socket() {
        // this block intentionally left blank
    } else {
        anyhow::bail!("file type unknown: {}", path.display());
    }
    config.stats.done_entries(1);

    Ok(hasher.finalize())
}

pub fn hash_meta(config: &Config, meta: &std::fs::Metadata) -> Result<[u8; 32]> {
    let mut buf = [0u8; 64];
    let mut cursor = Cursor::new(&mut buf[..]);

    if config.include_mode {
        // mode includes the file type as well, but we
        // don't really care
        cursor.write_u32::<LittleEndian>(meta.mode())?;
    }
    if config.include_size && meta.file_type().is_file() {
        // size used only for regular files since it may vary between
        // file system implementations for other type
        cursor.write_u64::<LittleEndian>(meta.size())?;
    }
    if config.include_uid {
        cursor.write_u32::<LittleEndian>(meta.uid())?;
    }
    if config.include_gid {
        cursor.write_u32::<LittleEndian>(meta.gid())?;
    }
    if config.include_ctime {
        cursor.write_i64::<LittleEndian>(meta.ctime())?;
    }
    if config.include_mtime {
        cursor.write_i64::<LittleEndian>(meta.mtime())?;
    }
    if config.include_atime {
        cursor.write_i64::<LittleEndian>(meta.atime())?;
    }

    let mut hasher = config.hasher();
    let len = cursor.position() as usize;
    hasher.update(&buf[..len]);
    Ok(hasher.finalize())
}

pub fn hash_file(config: &Config, path: &Path) -> Result<[u8; 32]> {
    let file = fs::File::open(path).map_err(|e| {
        let errno = e.raw_os_error().unwrap_or(-1);
        anyhow::anyhow!(
            "Failed to open file: {} (errno {}): {}",
            path.display(),
            errno,
            e
        )
    })?;
    let mut reader = BufReader::new(file);
    let mut hasher = config.hasher();
    let mut buf = vec![0u8; config.block_size];

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        config.stats.done_bytes(n as u64);
    }
    Ok(hasher.finalize())
}

pub fn hash_dir(config: &Config, path: &Path) -> Result<[u8; 32]> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        entries.push(entry?.path());
    }
    entries.sort();
    config.stats.add_entries(entries.len() as u64);

    let hashes: Vec<[u8; 32]> = entries
        .par_iter()
        .map(|entry| hash_entry(config, entry))
        .collect::<Result<_>>()?;

    let mut hasher = config.hasher();
    for h in hashes {
        hasher.update(&h);
    }

    Ok(hasher.finalize())
}
