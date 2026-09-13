//! `vigil update` CLI surface.

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
pub struct UpdateArgs {
    /// Check without installing
    #[arg(long)]
    pub check: bool,
    /// Install without confirmation
    #[arg(long)]
    pub yes: bool,
}

pub fn run(args: UpdateArgs) -> Result<()> {
    if !args.check {
        return vigil::update::self_update(args.yes);
    }
    match vigil::update::check_update() {
        Ok(status) => {
            println!("{status:?}");
            Ok(())
        }
        Err(error) => {
            println!("update check failed: {error}");
            Ok(())
        }
    }
}
