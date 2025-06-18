use anyhow::{Result, anyhow};
use clap::Parser;
use rayon::ThreadPoolBuilder;
use std::fs::File;
use std::io::{self, Read};
use std::{io::IsTerminal, process::ExitCode};

mod algo;
mod config;
mod hash;
mod stats;

use config::HashResultJson;

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
    let mut config = config::Config::try_from(args)?;

    if config.verbose {
        todo!("verbose mode not implemented");
    }

    ThreadPoolBuilder::new()
        .num_threads(config.threads)
        .build_global()?;

    if std::io::stdout().is_terminal() {
        config.stats.clone().spawn_display_thread();
    }

    let reference: Option<HashResultJson> = match config.verify.as_deref() {
        Some(verify) => {
            let reader: Box<dyn Read> = if verify == "-" {
                Box::new(io::stdin())
            } else {
                Box::new(File::open(verify)?)
            };

            let json: HashResultJson = serde_json::from_reader(reader)?;
            config.set_flags_from_string(&json.flags)?;

            if config.path.is_none() {
                config.path = Some(json.name.clone());
            }

            Some(json)
        }
        None => None,
    };

    let hash = hash::hash_entry(&config, &config.path.clone().unwrap())?;
    let result = HashResultJson::from_result(&config, &hash);

    match reference {
        Some(reference) => {
            if reference.hash == result.hash {
                println!("{}: Ok", result.name.display());
                Ok(())
            } else {
                println!("{}: Mismatch", result.name.display());
                Err(anyhow!("Checksums did not match"))
            }
        }
        None => {
            println!("{}", serde_json::to_string_pretty(&result)?);
            Ok(())
        }
    }
}
