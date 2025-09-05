use clap::{Parser, Subcommand};
use std::path::PathBuf;
use acli::ctag;

#[derive(Parser, Debug)]
#[clap(
    author = "Olive Casazza",
    version,
    about = "Atlassian command line utility tools for engineers and administrators."
)]
/// Command-line interface for the acli
struct Args {
    /// Log actions instead of executing actions
    #[arg(short, long)]
    dry_run: bool,
    /// Pretty-print the JSON output
    #[arg(short, long)]
    pretty: bool,
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
    /// interactive mode
    #[arg(short, long)]
    interactive: bool,

    /// Input JSON file path. Use '-' to read from stdin.
    #[arg(long)]
    input: Option<PathBuf>,

    /// Root page URL to construct a minimal tree if no input provided.
    #[arg(long)]
    root: Option<String>,

    /// Subcommand to run
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Operate on Confluence page labels
    Ctag(ctag::CtagCmd),
}

/// todo: define action structs in their own files which will do interfacing
/// to the actual request code in the shared library
///
/// actiosn:
/// ctag <list,add,update,remove> <CQL epression> labels from confluence pages
/// atag ctag list "parent = "1asd333e41" --tree # shows the page tree(s) matched by the CQL expression
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.verbose {
        eprintln!("acli v{}", env!("CARGO_PKG_VERSION"));
    }
    // Dispatch subcommands
    match args.command {
        Some(Commands::Ctag(ref cmd)) => {
            ctag::run(cmd, args.dry_run, args.pretty, args.verbose)?;
        }
        _ => {
            // todo: throw error command not provided and list --help
        }
    }

    Ok(())
}
