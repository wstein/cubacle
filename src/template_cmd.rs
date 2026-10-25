//! The `template` subcommand: work with `.cubacle` definition files.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use cubacle::notation::GenFile;

#[derive(Args)]
pub struct TemplateArgs {
    #[command(subcommand)]
    command: TemplateCommand,
}

#[derive(Subcommand)]
enum TemplateCommand {
    /// Parse a .cubacle file and report its name, size, variables and guard; no search, no counting
    Check {
        /// The .cubacle definition file
        file: PathBuf,
    },
}

pub fn run(args: TemplateArgs) -> Result<(), String> {
    match args.command {
        TemplateCommand::Check { file } => check(&file),
    }
}

fn check(file: &std::path::Path) -> Result<(), String> {
    let shown = file.display();
    let text = std::fs::read_to_string(file).map_err(|e| format!("{shown}: cannot read: {e}"))?;
    let parsed = GenFile::parse(&text)
        .map_err(|e| format!("{shown}:{}:{}: {}", e.line(), e.column(), e.message()))?;
    let guard = if parsed.guard.is_empty() {
        "no guard"
    } else {
        "guard ok"
    };
    let sizes: Vec<String> = parsed
        .templates
        .iter()
        .map(|template| {
            format!(
                "size {}, {} variables",
                template.kind().size(),
                template.variables().len()
            )
        })
        .collect();
    println!("ok: {}, {}, {guard}", parsed.name, sizes.join("; "));
    Ok(())
}
