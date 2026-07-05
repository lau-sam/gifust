//! Presets de tramage et de filtres couleur, avec leur traduction en filtres ffmpeg.

use clap::ValueEnum;

/// Méthode de tramage utilisée par `paletteuse`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Dither {
    /// Aucun tramage (aplats nets, fichier plus léger).
    None,
    /// Motif ordonné de Bayer.
    Bayer,
    /// Diffusion d'erreur Floyd-Steinberg.
    Floyd,
    /// Diffusion d'erreur Sierra2 (défaut, bon compromis).
    Sierra2,
}

impl Dither {
    pub const ALL: [Dither; 4] = [Dither::None, Dither::Bayer, Dither::Floyd, Dither::Sierra2];

    /// Valeur passée à l'option `dither=` de `paletteuse`.
    pub fn ffmpeg(self) -> &'static str {
        match self {
            Dither::None => "none",
            Dither::Bayer => "bayer",
            Dither::Floyd => "floyd_steinberg",
            Dither::Sierra2 => "sierra2",
        }
    }

    /// Libellé affiché dans le TUI.
    pub fn label(self) -> &'static str {
        match self {
            Dither::None => "Aucun",
            Dither::Bayer => "Bayer",
            Dither::Floyd => "Floyd-Steinberg",
            Dither::Sierra2 => "Sierra2",
        }
    }

    pub fn cycle(self, forward: bool) -> Dither {
        cycle_in(&Self::ALL, self, forward)
    }
}

/// Filtre couleur appliqué avant la génération de la palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ColorFilter {
    /// Couleurs d'origine.
    None,
    /// Noir et blanc.
    BlackWhite,
    /// Ton sépia.
    Sepia,
    /// Balance des couleurs chaude (plus de rouge).
    Warm,
    /// Balance des couleurs froide (plus de bleu).
    Cold,
    /// Couleurs plus saturées.
    Vibrant,
    /// Couleurs adoucies (désaturées).
    Muted,
    /// Rendu vintage.
    Vintage,
}

impl ColorFilter {
    pub const ALL: [ColorFilter; 8] = [
        ColorFilter::None,
        ColorFilter::BlackWhite,
        ColorFilter::Sepia,
        ColorFilter::Warm,
        ColorFilter::Cold,
        ColorFilter::Vibrant,
        ColorFilter::Muted,
        ColorFilter::Vintage,
    ];

    /// Fragment de chaîne de filtres ffmpeg, ou `None` pour laisser l'image telle quelle.
    pub fn ffmpeg(self) -> Option<&'static str> {
        match self {
            ColorFilter::None => None,
            ColorFilter::BlackWhite => Some("hue=s=0"),
            ColorFilter::Sepia => {
                Some("colorchannelmixer=.393:.769:.189:0:.349:.686:.168:0:.272:.534:.131")
            }
            ColorFilter::Warm => Some("colorbalance=rm=0.3:gm=0.1:bm=-0.2"),
            ColorFilter::Cold => Some("colorbalance=rm=-0.2:gm=0.0:bm=0.3"),
            ColorFilter::Vibrant => Some("eq=saturation=1.6:contrast=1.05"),
            ColorFilter::Muted => Some("eq=saturation=0.6"),
            ColorFilter::Vintage => Some("curves=preset=vintage"),
        }
    }

    /// Libellé affiché dans le TUI.
    pub fn label(self) -> &'static str {
        match self {
            ColorFilter::None => "Aucun",
            ColorFilter::BlackWhite => "Noir & blanc",
            ColorFilter::Sepia => "Sépia",
            ColorFilter::Warm => "Chaud",
            ColorFilter::Cold => "Froid",
            ColorFilter::Vibrant => "Vibrant",
            ColorFilter::Muted => "Adouci",
            ColorFilter::Vintage => "Vintage",
        }
    }

    pub fn cycle(self, forward: bool) -> ColorFilter {
        cycle_in(&Self::ALL, self, forward)
    }
}

fn cycle_in<T: Copy + PartialEq>(all: &[T], cur: T, forward: bool) -> T {
    let n = all.len();
    let i = all.iter().position(|&x| x == cur).unwrap_or(0);
    let j = if forward {
        (i + 1) % n
    } else {
        (i + n - 1) % n
    };
    all[j]
}
