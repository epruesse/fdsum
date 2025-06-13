use hex;
use anyhow::Result;
use clap::Parser;
mod hash;
mod algo;
mod config;

fn main() -> Result<()> {
    let args = config::Args::try_parse()?;
    let config: config::Config = args.into();

    if config.verbose {
        todo!("verbose mode not implemented");
    }
    
    println!("Scanning path: {}", config.path.display());

    let mut res = [0u8; 32];
    hash::hash_entry(&config, &config.path, &mut res)?;

    println!("Hash: {}", hex::encode(res));
    Ok(())
}


