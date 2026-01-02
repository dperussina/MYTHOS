mod cli;
mod manifest;
mod verify;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Verify {
            pack,
            suite,
            vector,
            fail_fast,
            json,
        } => {
            let manifest = manifest::PackManifest::load(&pack)
                .with_context(|| format!("Failed to load manifest from {:?}", pack))?;

            let results =
                verify::verify_suite(&manifest, &pack, &suite, vector.as_deref(), fail_fast);

            ctvp_runner::report::print_results(&results, json);

            std::process::exit(ctvp_runner::report::exit_code(&results));
        }

        Commands::List { pack, suite } => {
            let manifest = manifest::PackManifest::load(&pack)?;

            let vectors = if let Some(s) = suite {
                manifest.vectors_for_suite(&s)
            } else {
                manifest.vectors.iter().collect()
            };

            for vector in vectors {
                println!("{} - {}", vector.id, vector.description);
            }

            Ok(())
        }

        Commands::Info { pack, vector } => {
            let manifest = manifest::PackManifest::load(&pack)?;

            if let Some(entry) = manifest.get_vector(&vector) {
                println!("ID: {}", entry.id);
                println!("Description: {}", entry.description);
                println!("RFC: {}", entry.rfc_reference);
                println!("\nFiles:");
                for (key, path) in &entry.files {
                    println!("  {}: {}", key, path);
                }
                println!("\nExpected values:");
                for (key, value) in &entry.expected {
                    println!("  {}: {}", key, value);
                }
                Ok(())
            } else {
                anyhow::bail!("Vector {} not found", vector);
            }
        }
    }
}
