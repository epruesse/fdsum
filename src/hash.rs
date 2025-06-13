use std::fs;
use std::path::Path;
use std::io::BufReader;
use std::io::Read;
use anyhow::{Result, Context};

use rayon::prelude::*;


use crate::config::Config;

pub fn hash_entry(config: &Config, path: &Path, out: &mut [u8; 32]) -> Result<()> {
    let meta = std::fs::symlink_metadata(path)?;
    let filetype = meta.file_type();

    if filetype.is_dir() {
        hash_dir(config, path, out)?;
    } else if filetype.is_file() {
        hash_file(config, path, out)?;
    } else {
        //anyhow::bail!("file type not implemented");
    }
    
    Ok(())
}


pub fn hash_file(config: &Config, path: &Path, out: &mut [u8; 32]) -> Result<()> {
    let file = fs::File::open(path)
        .with_context(|| format!("Failed to open file: {}", path.display()))?;
    let mut reader = BufReader::new(file);
    let mut hasher = config.hasher();
    let mut buf = [0u8; 8192];

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    out.copy_from_slice(&hasher.finalize());
    Ok(())
}

pub fn hash_dir(config: &Config, path: &Path, out: &mut [u8; 32]) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(path)?
        .filter_map(Result::ok)
        .collect();
    entries.par_sort_by_key(|e| e.file_name());

    let hashes: Vec<[u8; 32]> = entries
        .par_iter()
        .map(|entry| {
            let mut entry_hash = [0u8; 32];
            hash_entry(config, &entry.path(), &mut entry_hash)
                .map(|_| entry_hash)
        })
        .collect::<Result<_>>()?;

    let mut hasher = config.hasher();
    for h in hashes {
        hasher.update(&h);
    }
    out.copy_from_slice(&hasher.finalize());

    Ok(())
}

