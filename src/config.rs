use crate::{algo, stats::SharedStats};
use anyhow::{Result, anyhow};
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::convert::TryFrom;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(ValueEnum, Clone, Debug)]
pub enum HashAlgorithm {
    Sha256,
    Blake3,
}

impl fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            HashAlgorithm::Sha256 => "sha256",
            HashAlgorithm::Blake3 => "blake3",
        };
        write!(f, "{}", s)
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None, max_term_width=100)]
/// Calculate checksums on files and directories recursively
pub struct Args {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// The path to checksum
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,

    /// Hash algorithm
    #[arg(short = 'm', long, default_value = "sha256")]
    algorithm: HashAlgorithm,

    /// Block size for reading files in kiB.
    #[arg(short = 'b', long, default_value_t = 128)]
    block_size: usize,

    /// Number of parallel threads [default: lesser of 8 and #cores]
    #[arg(short = 't', long)]
    num_threads: Option<usize>,

    /// Exclude file contents
    #[arg(short = 'C', long)]
    no_content: bool,

    /// Exclude size
    #[arg(short = 'S', long)]
    no_size: bool,

    /// Exclude all permissions (equal to -MOG)
    #[arg(short = 'P', long)]
    no_perms: bool,

    /// Exclude mode bits
    #[arg(short = 'M', long)]
    no_mode: bool,

    /// Exclude owner UID
    #[arg(short = 'O', long)]
    no_owner: bool,

    /// Exclude owner GID
    #[arg(short = 'G', long)]
    no_group: bool,

    /// Exclude mtime (last data modification)
    #[arg(short = 'T', long)]
    no_mtime: bool,

    /// Include atime (last access). We may cause a change to the
    /// atime ourselves while reading files.
    #[arg(long)]
    atime: bool,

    /// Include ctime (last status change). The ctime cannot be set by
    /// tools such as rsync and may be updated unexpectedly (e.g. by
    /// creating a hard link on a file).
    #[arg(long)]
    ctime: bool,

    /// Set via flags string. This overrides all other settings.
    #[arg(long, value_name = "STRING")]
    flags: Option<String>,

    /// Verify mode: provide fdsum json to validate
    #[arg(long, short = 'c', value_name = "FILE")]
    pub verify: Option<String>,
}

#[derive(Debug)]
pub struct Config {
    pub path: Option<PathBuf>,
    pub verbose: bool,
    pub algorithm: HashAlgorithm,
    pub block_size: usize,
    pub threads: usize,
    pub verify: Option<String>,

    pub include_file_content: bool,
    pub include_size: bool,
    pub include_mode: bool,
    pub include_uid: bool,
    pub include_gid: bool,
    pub include_ctime: bool,
    pub include_mtime: bool,
    pub include_atime: bool,

    pub stats: Arc<SharedStats>,
}

impl Config {
    pub fn hasher(&self) -> Box<dyn algo::Hasher> {
        match self.algorithm {
            HashAlgorithm::Blake3 => Box::new(Sha256::new()),
            HashAlgorithm::Sha256 => Box::new(algo::Blake3Wrapper::new()),
        }
    }

    pub fn flags_string(&self) -> String {
        let mut flags = String::new();
        if self.include_file_content {
            flags.push('c');
        }
        if self.include_size {
            flags.push('s');
        }
        if self.include_mode {
            flags.push('p');
        }
        if self.include_uid {
            flags.push('u');
        }
        if self.include_gid {
            flags.push('g');
        }
        if self.include_ctime {
            flags.push('t');
        }
        if self.include_mtime {
            flags.push('m');
        }
        if self.include_atime {
            flags.push('a');
        }

        format!("v1:{}:{}", self.algorithm, flags)
    }

    pub fn set_flags_from_string(&mut self, flags: &str) -> Result<()> {
        let parts: Vec<&str> = flags.split(':').collect();
        if parts.len() != 3 || parts[0] != "v1" {
            return Err(anyhow!("Unsupported config flags string format"));
        }

        self.algorithm = HashAlgorithm::from_str(parts[1], false)
            .expect("unknown algorithm in config flag string");

        self.include_file_content = parts[2].contains('c');
        self.include_size = parts[2].contains('s');
        self.include_mode = parts[2].contains('p');
        self.include_uid = parts[2].contains('u');
        self.include_gid = parts[2].contains('g');
        self.include_ctime = parts[2].contains('t');
        self.include_mtime = parts[2].contains('m');
        self.include_atime = parts[2].contains('a');

        Ok(())
    }
}

impl TryFrom<Args> for Config {
    type Error = anyhow::Error;

    fn try_from(args: Args) -> Result<Self> {
        let mut obj = Self {
            path: args.path,
            verbose: args.verbose,
            algorithm: args.algorithm,
            block_size: args.block_size * 1024,
            threads: args.num_threads.unwrap_or(num_cpus::get().min(8)),
            verify: args.verify,
            include_file_content: !args.no_content,
            include_size: !args.no_size,
            include_mode: !args.no_perms && !args.no_mode,
            include_uid: !args.no_perms && !args.no_owner,
            include_gid: !args.no_perms && !args.no_group,
            include_mtime: !args.no_mtime,
            include_ctime: args.ctime,
            include_atime: args.atime,

            stats: Arc::new(SharedStats::new()),
        };
        if let Some(flags) = args.flags {
            obj.set_flags_from_string(flags.as_str())?;
        }
        if obj.path.is_none() && obj.verify.is_none() {
            return Err(anyhow!("Neither PATH nor verify FILE specified"));
        }
        Ok(obj)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HashResultJson {
    pub name: PathBuf,
    pub hash: String,
    pub flags: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_seconds: Option<f64>,
}

impl HashResultJson {
    pub fn from_result(config: &Config, hash: &[u8]) -> Self {
        let stats = config.stats.snapshot();
        let elapsed = (stats.elapsed.as_secs_f64() * 100.0).round() / 100.0;

        HashResultJson {
            name: config.path.clone().unwrap(),
            hash: hex::encode(hash),
            flags: config.flags_string(),

            entries: Some(stats.entries_total),
            bytes: Some(stats.bytes_total),
            elapsed_seconds: Some(elapsed),
        }
    }
}
