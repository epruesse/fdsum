use anyhow::Result;
use clap::Parser;
use rayon::ThreadPoolBuilder;
use std::{io::IsTerminal, process::ExitCode};

mod algo;
mod config;
mod hash;
mod stats;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {}", err);
            ExitCode::FAILURE
        }
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

    if std::io::stdout().is_terminal() {
        config.stats.clone().spawn_display_thread();
    }

    let res = hash::hash_entry(&config, &config.path)?;
    let stats = config.stats.snapshot();

    let js = serde_json::json!({
        "name": config.path.file_name(),
        "entries": stats.entries_total,
        "bytes": stats.bytes_total,
        "elapsed_seconds": (stats.elapsed.as_secs_f64() * 100.0).round()/100.0,
        "hash": hex::encode(res),
    });

    println!("{}", serde_json::to_string_pretty(&js)?);

    Ok(())
}
