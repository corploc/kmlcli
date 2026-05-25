use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
use std::io;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "kmlcli", about = "KML/KMZ terminal viewer")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
    /// KML/KMZ file to view (launches TUI)
    pub file: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Open file in TUI viewer
    View { file: PathBuf },
    /// Dump document metadata
    Info {
        file: PathBuf,
        #[arg(short, long, default_value = "json")]
        format: OutputFormat,
    },
    /// List all placemarks and folders
    List {
        file: PathBuf,
        #[arg(short, long, default_value = "json")]
        format: OutputFormat,
    },
    /// Print document structure as a tree
    Tree { file: PathBuf },
    /// Generate shell completions
    Completions { shell: Shell },
}

#[derive(ValueEnum, Debug, Clone)]
pub enum OutputFormat {
    Json,
    Table,
}

impl Cli {
    pub fn file_path(&self) -> Option<&PathBuf> {
        match &self.command {
            Some(Command::View { file }) => Some(file),
            Some(Command::Info { file, .. }) => Some(file),
            Some(Command::List { file, .. }) => Some(file),
            Some(Command::Tree { file }) => Some(file),
            Some(Command::Completions { .. }) => None,
            None => self.file.as_ref(),
        }
    }
}

pub fn print_completions(shell: Shell) {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut io::stdout());
}
