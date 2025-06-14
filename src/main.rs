use anyhow::Result;
use clap::Parser;
use rayon::ThreadPoolBuilder;
use std::process::ExitCode;
mod algo;
mod config;
mod hash;


fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {}", err);
            ExitCode::FAILURE
        },
    }
}

fn run() -> Result<()> {
    let args = config::Args::parse();
    let config: config::Config = args.into();

    if config.verbose {
        todo!("verbose mode not implemented");
    }

    ThreadPoolBuilder::new()
        .num_threads(config.threads)
        .build_global()?;

    let res = hash::hash_entry(&config, &config.path)?;

    println!("{{hash: '{}'}}", hex::encode(res));
    Ok(())
}
