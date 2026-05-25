use clap::Parser;
use color_eyre::eyre::{eyre, Result};

mod cli;

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = cli::Cli::parse();

    if let Some(cli::Command::Completions { shell }) = &args.command {
        cli::print_completions(*shell);
        return Ok(());
    }

    let file = args
        .file_path()
        .ok_or_else(|| eyre!("No file specified. Usage: kmlcli <file.kml>"))?;
    let doc = kmlcli::parser::parse_file(file)?;

    match &args.command {
        Some(cli::Command::Info { .. }) => kmlcli::commands::info::run(&doc)?,
        Some(cli::Command::List { .. }) => kmlcli::commands::list::run(&doc)?,
        Some(cli::Command::Tree { .. }) => kmlcli::commands::tree::run(&doc),
        _ => {
            kmlcli::tui::app::App::new(doc).run()?;
        }
    }
    Ok(())
}
