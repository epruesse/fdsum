use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use sha2::{Digest, Sha256};
use num_cpus;
use crate::algo;


#[derive(ValueEnum, Clone, Debug)]
pub enum HashAlgorithm {
    Sha256,
    Blake3,
}

#[derive(Parser)]
#[command(name = "fdsum", version, about, long_about = None)]
pub struct Args {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// The path to checksum
    #[arg(value_name = "PATH")]
    path: PathBuf,

    /// Hash algorithm to use
    #[arg(short = 'm', long, default_value = "blake3")]
    algorithm: HashAlgorithm,

    /// Block size for reading files
    #[arg(short = 'b', long, default_value_t = 128)]
    block_size: usize,

    /// Number of parallel threads
    #[arg(short = 't', long)]
    threads: Option<usize>
}

#[derive(Debug)]
pub struct Config {
    pub path: PathBuf,
    pub verbose: bool,
    pub algorithm: HashAlgorithm,
    pub block_size: usize,
    pub threads: usize,
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
        }
    }
}
