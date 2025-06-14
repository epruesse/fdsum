use crate::algo;
use clap::{Parser, ValueEnum};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

#[derive(ValueEnum, Clone, Debug)]
pub enum HashAlgorithm {
    Sha256,
    Blake3,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Calculate checksums on files and directories recursively
pub struct Args {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// The path to checksum
    #[arg(value_name = "PATH")]
    path: PathBuf,

    /// Hash algorithm to use
    #[arg(short = 'm', long, default_value = "sha256")]
    algorithm: HashAlgorithm,

    /// Block size for reading files
    #[arg(short = 'b', long, default_value_t = 128)]
    block_size: usize,

    /// Number of parallel threads
    #[arg(short = 't', long)]
    threads: Option<usize>,

    /// Exclude file content
    #[arg(short = 'C', long)]
    no_content: bool,

    /// Exclude all permissions (equal to -MOP)
    #[arg(short = 'P', long)]
    no_perms: bool,

    /// Exclude mode bits
    #[arg(short = 'M', long)]
    no_mode: bool,

    /// Exclude owner
    #[arg(short = 'O', long)]
    no_owner: bool,

    /// Exclude group
    #[arg(short = 'G', long)]
    no_group: bool,

    /// Exclude timestamps
    #[arg(short = 'T', long)]
    no_timestamps: bool,
}

#[derive(Debug)]
pub struct Config {
    pub path: PathBuf,
    pub verbose: bool,
    pub algorithm: HashAlgorithm,
    pub block_size: usize,
    pub threads: usize,

    pub include_file_content: bool,
    pub include_mode: bool,  // -p
    pub include_uid: bool,   // -o and -g
    pub include_gid: bool,   // -o and -g
    pub include_ctime: bool, // -t
    pub include_mtime: bool, // -t
    pub include_atime: bool, // -t
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
            threads: args.threads.unwrap_or(num_cpus::get().min(8)),
            include_file_content: !args.no_content,
            include_mode: !args.no_perms && !args.no_mode,
            include_uid: !args.no_perms && !args.no_owner,
            include_gid: !args.no_perms && !args.no_group,
            include_ctime: !args.no_timestamps,
            include_mtime: !args.no_timestamps,
            include_atime: !args.no_timestamps,
        }
    }
}
