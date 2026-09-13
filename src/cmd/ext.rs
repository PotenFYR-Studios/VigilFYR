//! `vigil extension` lifecycle.

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};

use vigil::ext::{extensions_root, install_extension, load_extensions, remove_extension};

#[derive(Parser)]
pub struct ExtensionArgs {
    #[command(subcommand)]
    pub command: ExtensionCommands,
}

#[derive(Subcommand)]
pub enum ExtensionCommands {
    List,
    Install { source: String },
    Remove { name: String },
}

pub fn run(args: ExtensionArgs) -> Result<()> {
    match args.command {
        ExtensionCommands::List => {
            for extension in load_extensions(extensions_root()) {
                println!(
                    "{}\t{}\t{}",
                    extension.manifest.name,
                    extension.manifest.version,
                    extension.manifest.description
                );
            }
        }
        ExtensionCommands::Install { source } => {
            let path = install_extension(&source, extensions_root())?;
            println!("installed extension at {}", path.display());
        }
        ExtensionCommands::Remove { name } => {
            if name.contains('/') || name.contains("..") {
                bail!("invalid extension name");
            }
            remove_extension(&name, extensions_root())?;
            println!("removed extension {name}");
        }
    }
    Ok(())
}
