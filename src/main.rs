mod cli;
mod convert;
mod filters;
mod tui;

use clap::{CommandFactory, Parser};

use cli::{Cli, Command};

fn main() {
    if let Err(e) = run() {
        eprintln!("gifust: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Tui) => {
            convert::ensure_ffmpeg()?;
            tui::run()?;
        }
        None => match cli.convert.input.clone() {
            Some(input) => {
                convert::ensure_ffmpeg()?;
                let opts = cli.convert.to_options();
                let out = convert::convert(&input, &opts, true)?;
                println!("gif créé : {}", out.display());
            }
            None => {
                // `gifust` sans argument : on affiche l'aide.
                Cli::command().print_help()?;
                println!();
            }
        },
    }
    Ok(())
}
