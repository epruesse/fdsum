use anyhow::Result;
use clap::Parser;
use rayon::ThreadPoolBuilder;
mod algo;
mod config;
mod hash;

fn main() -> Result<()> {
    let args = config::Args::try_parse()?;
    let config: config::Config = args.into();

    if config.verbose {
        todo!("verbose mode not implemented");
    }

    ThreadPoolBuilder::new()
        .num_threads(config.threads)
        .build_global()?;

    let mut res = [0u8; 32];
    hash::hash_entry(&config, &config.path, &mut res)?;

    println!("{{hash: '{}'}}", hex::encode(res));
    Ok(())
}
