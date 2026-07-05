//! Définition des arguments en ligne de commande (clap).

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::convert::ConvertOptions;
use crate::filters::{ColorFilter, Dither};

#[derive(Parser)]
#[command(
    name = "gifust",
    version,
    about = "Convertit des vidéos en GIF, en ligne de commande ou via un TUI",
    long_about = None,
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[command(flatten)]
    pub convert: ConvertArgs,
}

#[derive(Subcommand)]
pub enum Command {
    /// Lance l'interface interactive (TUI) : navigation, choix de la vidéo et des options
    Tui,
}

#[derive(Args)]
pub struct ConvertArgs {
    /// Vidéo source à convertir
    pub input: Option<PathBuf>,

    /// Images par seconde
    #[arg(short, long, default_value_t = 15)]
    pub fps: u32,

    /// Largeur en pixels (hauteur calculée automatiquement)
    #[arg(short, long, default_value_t = 1080)]
    pub width: u32,

    /// Début du segment (ex : 3 ou 00:00:03)
    #[arg(short = 's', long)]
    pub start: Option<String>,

    /// Durée du segment (ex : 5 ou 00:00:05)
    #[arg(short = 'd', long)]
    pub duration: Option<String>,

    /// Nombre maximum de couleurs de la palette (2-256)
    #[arg(short = 'c', long, default_value_t = 256)]
    pub colors: u32,

    /// Méthode de tramage
    #[arg(long, value_enum, default_value_t = Dither::Sierra2)]
    pub dither: Dither,

    /// Filtre couleur
    #[arg(long, value_enum, default_value_t = ColorFilter::None)]
    pub filter: ColorFilter,

    /// Répétitions du gif (0 = infini, -1 = aucune)
    #[arg(long = "loop", default_value_t = 0, allow_hyphen_values = true)]
    pub loop_count: i32,

    /// Fichier de sortie (défaut : même nom que la source en .gif)
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,
}

impl ConvertArgs {
    pub fn to_options(&self) -> ConvertOptions {
        ConvertOptions {
            fps: self.fps,
            width: self.width,
            start: self.start.clone(),
            duration: self.duration.clone(),
            colors: self.colors,
            dither: self.dither,
            filter: self.filter,
            loop_count: self.loop_count,
            output: self.output.clone(),
        }
    }
}
