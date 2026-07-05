//! Conversion vidéo → gif via ffmpeg (méthode palette en deux passes).

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};

use crate::filters::{ColorFilter, Dither};

/// Options de conversion partagées par la CLI et le TUI.
#[derive(Debug, Clone)]
pub struct ConvertOptions {
    pub fps: u32,
    pub width: u32,
    pub start: Option<String>,
    pub duration: Option<String>,
    pub colors: u32,
    pub dither: Dither,
    pub filter: ColorFilter,
    pub loop_count: i32,
    pub output: Option<PathBuf>,
}

impl ConvertOptions {
    /// Chaîne de filtres commune aux deux passes (fps, échelle, filtre couleur).
    fn base_chain(&self) -> String {
        let mut chain = format!(
            "fps={},scale={}:-1:flags=lanczos",
            self.fps.max(1),
            self.width.max(1)
        );
        if let Some(f) = self.filter.ffmpeg() {
            chain.push(',');
            chain.push_str(f);
        }
        chain
    }
}

/// Chemin de sortie effectif (option `output`, sinon même nom que la source en `.gif`).
pub fn output_path(src: &Path, opts: &ConvertOptions) -> PathBuf {
    opts.output
        .clone()
        .unwrap_or_else(|| src.with_extension("gif"))
}

/// Vérifie que ffmpeg est disponible dans le PATH.
pub fn ensure_ffmpeg() -> Result<()> {
    let ok = Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if ok {
        Ok(())
    } else {
        bail!("ffmpeg introuvable dans le PATH. Installe-le : brew install ffmpeg");
    }
}

/// Convertit `src` en gif selon `opts`. Renvoie le chemin du gif créé.
///
/// Si `verbose`, ffmpeg écrit sa progression sur le terminal ; sinon sa sortie
/// est capturée et n'apparaît qu'en cas d'erreur.
pub fn convert(src: &Path, opts: &ConvertOptions, verbose: bool) -> Result<PathBuf> {
    if !src.is_file() {
        bail!("fichier introuvable : {}", src.display());
    }
    let out = output_path(src, opts);
    let palette = std::env::temp_dir().join(format!("gifust-palette-{}.png", std::process::id()));
    let base = opts.base_chain();

    // Passe 1 : génération de la palette.
    let mut pass1 = base_cmd(verbose);
    add_trim(&mut pass1, opts);
    pass1.arg("-i").arg(src);
    pass1.arg("-vf").arg(format!(
        "{base},palettegen=max_colors={}",
        opts.colors.clamp(2, 256)
    ));
    pass1.arg("-update").arg("1");
    pass1.arg(&palette);
    run(&mut pass1, verbose).context("génération de la palette")?;

    // Passe 2 : encodage du gif en réutilisant la palette.
    let mut pass2 = base_cmd(verbose);
    add_trim(&mut pass2, opts);
    pass2.arg("-i").arg(src);
    pass2.arg("-i").arg(&palette);
    pass2.arg("-lavfi").arg(format!(
        "{base} [x]; [x][1:v] paletteuse=dither={}",
        opts.dither.ffmpeg()
    ));
    pass2.arg("-loop").arg(opts.loop_count.to_string());
    pass2.arg(&out);
    let result = run(&mut pass2, verbose).context("encodage du gif");

    let _ = std::fs::remove_file(&palette);
    result?;
    Ok(out)
}

fn base_cmd(verbose: bool) -> Command {
    let mut c = Command::new("ffmpeg");
    c.arg("-y").arg("-hide_banner");
    if verbose {
        c.arg("-loglevel").arg("error").arg("-stats");
    } else {
        c.arg("-loglevel").arg("error").arg("-nostats");
    }
    c
}

fn add_trim(c: &mut Command, opts: &ConvertOptions) {
    if let Some(s) = opts.start.as_deref().filter(|s| !s.trim().is_empty()) {
        c.arg("-ss").arg(s.trim());
    }
    if let Some(d) = opts.duration.as_deref().filter(|d| !d.trim().is_empty()) {
        c.arg("-t").arg(d.trim());
    }
}

fn run(c: &mut Command, verbose: bool) -> Result<()> {
    if verbose {
        let status = c.status().context("échec du lancement de ffmpeg")?;
        if !status.success() {
            bail!("ffmpeg a échoué (code {:?})", status.code());
        }
    } else {
        let output = c.output().context("échec du lancement de ffmpeg")?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            bail!("ffmpeg a échoué : {}", err.trim());
        }
    }
    Ok(())
}
