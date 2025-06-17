use crate::{algo, stats::SharedStats};
use clap::{Parser, ValueEnum};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(ValueEnum, Clone, Debug)]
pub enum HashAlgorithm {
    Sha256,
    Blake3,
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
    path: PathBuf,

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
}

#[derive(Debug)]
pub struct Config {
    pub path: PathBuf,
    pub verbose: bool,
    pub algorithm: HashAlgorithm,
    pub block_size: usize,
    pub threads: usize,

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
}

impl From<Args> for Config {
    fn from(args: Args) -> Self {
        Self {
            path: args.path,
            verbose: args.verbose,
            algorithm: args.algorithm,
            block_size: args.block_size * 1024,
            threads: args.num_threads.unwrap_or(num_cpus::get().min(8)),
            include_file_content: !args.no_content,
            include_size: !args.no_size,
            include_mode: !args.no_perms && !args.no_mode,
            include_uid: !args.no_perms && !args.no_owner,
            include_gid: !args.no_perms && !args.no_group,
            include_mtime: !args.no_mtime,
            include_ctime: args.ctime,
            include_atime: args.atime,

            stats: Arc::new(SharedStats::new()),
        }
    }
}
