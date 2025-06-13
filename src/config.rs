use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use sha2::{Digest, Sha256};
use crate::algo;


#[derive(ValueEnum, Clone, Debug)]
pub enum HashAlgorithm {
    Sha256,
    Blake3,
}

#[derive(Parser)]
#[command(name = "fdsum", version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, global = true)]
    verbose: bool,

    #[arg(value_name = "PATH")]
    path: PathBuf,

    #[arg(short = 'm', long, default_value = "blake3")]
    algorithm: HashAlgorithm,

    #[arg(short = 'b', long, default_value_t = 128)]
    block_size: usize,
}

#[derive(Debug)]
pub struct Config {
    pub path: PathBuf,
    pub verbose: bool,
    pub algorithm: HashAlgorithm,
    pub block_size: usize,
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
        }
    }
}
