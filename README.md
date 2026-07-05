# gifust — video to GIF converter (CLI + TUI)

**Fast command-line video-to-GIF converter.** Convert MP4, MOV, MKV, WebM, AVI and more into high-quality animated GIFs from the terminal — either directly via the CLI or through an interactive TUI. Written in Rust, powered by `ffmpeg` (palette method) for crisp, lightweight GIFs. macOS & Linux, installable with Homebrew.

---

Convertisseur **vidéo → GIF** rapide, en ligne de commande ou via un TUI interactif façon [lazygit](https://github.com/jesseduffield/lazygit). Écrit en Rust, s'appuie sur `ffmpeg` avec la méthode palette (deux passes) pour un rendu net et léger.

![Démo de gifust en CLI et en TUI](docs/demo.gif)

## Installation

### Homebrew

```sh
brew tap lau-sam/gifust https://github.com/lau-sam/gifust
brew install gifust
```

`ffmpeg` est installé automatiquement comme dépendance.

### Depuis les sources

```sh
git clone https://github.com/lau-sam/gifust
cd gifust
cargo install --path .
```

Prérequis : [Rust](https://rustup.rs) et [ffmpeg](https://ffmpeg.org) dans le `PATH`.

## Utilisation

### Ligne de commande

```sh
gifust ma-video.mp4                          # gif à côté de la source, 15 fps, 1080 px
gifust ma-video.mp4 --fps 24 --width 640     # personnalisé
gifust clip.mov --start 3 --duration 5       # extrait de 5 s à partir de 3 s
gifust clip.mp4 --filter sepia --colors 128  # filtre couleur + palette réduite
gifust clip.mp4 -o sortie.gif --loop -1      # sortie custom, sans boucle
```

Options :

| Option | Défaut | Description |
| --- | --- | --- |
| `-f, --fps` | `15` | Images par seconde |
| `-w, --width` | `1080` | Largeur en pixels (hauteur automatique) |
| `-s, --start` | — | Début du segment (`3` ou `00:00:03`) |
| `-d, --duration` | — | Durée du segment (`5` ou `00:00:05`) |
| `-c, --colors` | `256` | Couleurs max de la palette (2-256) |
| `--dither` | `sierra2` | `none`, `bayer`, `floyd`, `sierra2` |
| `--filter` | `none` | `none`, `black-white`, `sepia`, `warm`, `cold`, `vibrant`, `muted`, `vintage` |
| `--loop` | `0` | Répétitions (`0` = infini, `-1` = aucune) |
| `-o, --output` | — | Chemin de sortie (défaut : même nom en `.gif`) |

### TUI interactif

```sh
gifust tui
```

- **Navigateur** (gauche) : parcours des dossiers ; seules les vidéos reconnues sont sélectionnables.
- **Options** (droite) : FPS, largeur, découpe, couleurs, tramage, filtre, boucle, sortie. Un panneau décrit le champ sélectionné (rôle, type attendu, exemple).
- **Barre d'aide** en bas, contextuelle.

Le TUI démarre en thème sombre ; `t` bascule entre sombre et clair.

Raccourcis : `j`/`k` naviguer · `Tab` changer de panneau · `↵` ouvrir / éditer · `0-9` saisir directement une valeur · `h`/`l` remonter d'un dossier / changer une valeur · `c` convertir · `t` thème sombre/clair · `q` quitter.

## Formats vidéo reconnus

`mp4`, `mov`, `mkv`, `avi`, `webm`, `m4v`, `flv`, `wmv`, `mpg`, `mpeg`, `ts`, `m2ts`, `3gp`, `ogv`.

## Licence

[MIT](LICENSE)
