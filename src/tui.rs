//! TUI interactif façon lazygit : navigateur de fichiers + panneau d'options + barre d'aide.

use std::path::{Path, PathBuf};

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::{DefaultTerminal, Frame};

use crate::convert::{self, ConvertOptions};
use crate::filters::{ColorFilter, Dither};

/// Extensions vidéo reconnues (seules celles-ci sont sélectionnables).
const VIDEO_EXTS: &[&str] = &[
    "mp4", "mov", "mkv", "avi", "webm", "m4v", "flv", "wmv", "mpg", "mpeg", "ts", "m2ts", "3gp",
    "ogv",
];

const ACCENT: Color = Color::Magenta;

pub fn run() -> Result<()> {
    let mut terminal = ratatui::init();
    let res = run_app(&mut terminal);
    ratatui::restore();
    res
}

// ---------------------------------------------------------------------------
// État
// ---------------------------------------------------------------------------

#[derive(PartialEq, Clone, Copy)]
enum Focus {
    Browser,
    Options,
}

struct App {
    focus: Focus,
    browser: Browser,
    options: OptionsForm,
    selected_video: Option<PathBuf>,
    status: String,
    should_quit: bool,
}

fn run_app(terminal: &mut DefaultTerminal) -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut app = App {
        focus: Focus::Browser,
        browser: Browser::new(cwd),
        options: OptionsForm::default(),
        selected_video: None,
        status: "Choisis une vidéo, règle les options, puis 'c' pour convertir.".into(),
        should_quit: false,
    };

    while !app.should_quit {
        terminal.draw(|f| ui(f, &mut app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                handle_key(terminal, &mut app, key.code)?;
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Navigateur de fichiers
// ---------------------------------------------------------------------------

struct Entry {
    name: String,
    path: PathBuf,
    is_dir: bool,
}

struct Browser {
    cwd: PathBuf,
    entries: Vec<Entry>,
    state: ListState,
}

impl Browser {
    fn new(cwd: PathBuf) -> Self {
        let mut b = Browser {
            cwd,
            entries: Vec::new(),
            state: ListState::default(),
        };
        b.reload();
        b
    }

    fn reload(&mut self) {
        self.entries = read_entries(&self.cwd);
        self.state.select((!self.entries.is_empty()).then_some(0));
    }

    fn selected(&self) -> Option<&Entry> {
        self.state.selected().and_then(|i| self.entries.get(i))
    }

    fn next(&mut self) {
        move_sel(&mut self.state, self.entries.len(), 1);
    }

    fn prev(&mut self) {
        move_sel(&mut self.state, self.entries.len(), -1);
    }

    /// Ouvre l'élément sélectionné : entre dans un dossier, ou renvoie la vidéo choisie.
    fn open(&mut self) -> Option<PathBuf> {
        let entry = self.selected()?;
        if entry.is_dir {
            self.cwd = entry.path.clone();
            self.reload();
            None
        } else {
            Some(entry.path.clone())
        }
    }

    fn up(&mut self) {
        if let Some(parent) = self.cwd.parent() {
            self.cwd = parent.to_path_buf();
            self.reload();
        }
    }
}

fn read_entries(dir: &Path) -> Vec<Entry> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let path = entry.path();
            if path.is_dir() {
                dirs.push(Entry {
                    name,
                    path,
                    is_dir: true,
                });
            } else if is_video(&path) {
                files.push(Entry {
                    name,
                    path,
                    is_dir: false,
                });
            }
        }
    }
    dirs.sort_by_key(|e| e.name.to_lowercase());
    files.sort_by_key(|e| e.name.to_lowercase());
    dirs.extend(files);
    dirs
}

fn is_video(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .map(|e| VIDEO_EXTS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Formulaire d'options
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum Field {
    Fps,
    Width,
    Start,
    Duration,
    Colors,
    Dither,
    Filter,
    Loop,
    Output,
}

const FIELDS: [Field; 9] = [
    Field::Fps,
    Field::Width,
    Field::Start,
    Field::Duration,
    Field::Colors,
    Field::Dither,
    Field::Filter,
    Field::Loop,
    Field::Output,
];

struct OptionsForm {
    fps: u32,
    width: u32,
    start: String,
    duration: String,
    colors: u32,
    dither: Dither,
    filter: ColorFilter,
    loop_count: i32,
    output: String,
    sel: usize,
    editing: bool,
    buffer: String,
}

impl Default for OptionsForm {
    fn default() -> Self {
        OptionsForm {
            fps: 15,
            width: 1080,
            start: String::new(),
            duration: String::new(),
            colors: 256,
            dither: Dither::Sierra2,
            filter: ColorFilter::None,
            loop_count: 0,
            output: String::new(),
            sel: 0,
            editing: false,
            buffer: String::new(),
        }
    }
}

impl OptionsForm {
    fn field(&self) -> Field {
        FIELDS[self.sel]
    }

    fn is_cycle(f: Field) -> bool {
        matches!(f, Field::Dither | Field::Filter)
    }

    fn next(&mut self) {
        if self.sel + 1 < FIELDS.len() {
            self.sel += 1;
        }
    }

    fn prev(&mut self) {
        self.sel = self.sel.saturating_sub(1);
    }

    fn label(f: Field) -> &'static str {
        match f {
            Field::Fps => "FPS",
            Field::Width => "Largeur",
            Field::Start => "Début",
            Field::Duration => "Durée",
            Field::Colors => "Couleurs",
            Field::Dither => "Tramage",
            Field::Filter => "Filtre",
            Field::Loop => "Boucle",
            Field::Output => "Sortie",
        }
    }

    /// Renvoie la valeur affichée et `true` si c'est un texte indicatif (champ vide).
    fn display(&self, f: Field) -> (String, bool) {
        let placeholder = |s: &str, hint: &str| {
            if s.trim().is_empty() {
                (hint.to_string(), true)
            } else {
                (s.trim().to_string(), false)
            }
        };
        match f {
            Field::Fps => (self.fps.to_string(), false),
            Field::Width => (format!("{} px", self.width), false),
            Field::Start => placeholder(&self.start, "s ou hh:mm:ss"),
            Field::Duration => placeholder(&self.duration, "vidéo entière"),
            Field::Colors => (self.colors.to_string(), false),
            Field::Dither => (self.dither.label().to_string(), false),
            Field::Filter => (self.filter.label().to_string(), false),
            Field::Loop => (
                match self.loop_count {
                    0 => "infini".into(),
                    -1 => "aucune".into(),
                    n => format!("{n}×"),
                },
                false,
            ),
            Field::Output => placeholder(&self.output, "auto (même nom en .gif)"),
        }
    }

    /// Description contextuelle du champ : [rôle, type attendu, exemple/défaut].
    fn describe(f: Field) -> [&'static str; 3] {
        match f {
            Field::Fps => [
                "Images par seconde du gif.",
                "Type : entier > 0",
                "Ex : 15, 24   ·   défaut 15",
            ],
            Field::Width => [
                "Largeur en pixels (hauteur calculée automatiquement).",
                "Type : entier > 0 (pixels)",
                "Ex : 480, 1080   ·   défaut 1080",
            ],
            Field::Start => [
                "Instant de début du segment à extraire.",
                "Type : durée — secondes ou hh:mm:ss",
                "Ex : 3, 00:00:03   ·   défaut : début de la vidéo",
            ],
            Field::Duration => [
                "Durée du segment à extraire depuis le début.",
                "Type : durée — secondes ou hh:mm:ss",
                "Ex : 5, 00:00:05   ·   défaut : vidéo entière",
            ],
            Field::Colors => [
                "Nombre maximum de couleurs de la palette.",
                "Type : entier 2-256",
                "Ex : 128, 256   ·   défaut 256",
            ],
            Field::Dither => [
                "Méthode de tramage (h/l pour changer).",
                "Valeurs : Aucun, Bayer, Floyd-Steinberg, Sierra2",
                "Réduit le banding, alourdit un peu le fichier",
            ],
            Field::Filter => [
                "Filtre couleur appliqué avant la palette (h/l pour changer).",
                "Valeurs : Aucun, Noir & blanc, Sépia, Chaud, Froid…",
                "Défaut : Aucun (couleurs d'origine)",
            ],
            Field::Loop => [
                "Nombre de répétitions du gif.",
                "Type : entier (0 = infini, -1 = aucune)",
                "Défaut : infini",
            ],
            Field::Output => [
                "Chemin du fichier gif de sortie.",
                "Type : chemin de fichier",
                "Défaut : même nom que la source, extension .gif",
            ],
        }
    }

    fn cycle(&mut self, forward: bool) {
        match self.field() {
            Field::Dither => self.dither = self.dither.cycle(forward),
            Field::Filter => self.filter = self.filter.cycle(forward),
            _ => {}
        }
    }

    fn begin_edit(&mut self) {
        let f = self.field();
        if Self::is_cycle(f) {
            self.cycle(true);
            return;
        }
        self.buffer = match f {
            Field::Fps => self.fps.to_string(),
            Field::Width => self.width.to_string(),
            Field::Colors => self.colors.to_string(),
            Field::Loop => self.loop_count.to_string(),
            Field::Start => self.start.clone(),
            Field::Duration => self.duration.clone(),
            Field::Output => self.output.clone(),
            _ => String::new(),
        };
        self.editing = true;
    }

    fn commit(&mut self) {
        let value = self.buffer.trim().to_string();
        match self.field() {
            Field::Fps => {
                if let Ok(v) = value.parse::<u32>() {
                    if v > 0 {
                        self.fps = v;
                    }
                }
            }
            Field::Width => {
                if let Ok(v) = value.parse::<u32>() {
                    if v > 0 {
                        self.width = v;
                    }
                }
            }
            Field::Colors => {
                if let Ok(v) = value.parse::<u32>() {
                    self.colors = v.clamp(2, 256);
                }
            }
            Field::Loop => {
                if let Ok(v) = value.parse::<i32>() {
                    self.loop_count = v.max(-1);
                }
            }
            Field::Start => self.start = value,
            Field::Duration => self.duration = value,
            Field::Output => self.output = value,
            _ => {}
        }
        self.editing = false;
        self.buffer.clear();
    }

    /// Démarre une édition « fraîche » (tampon vide) déclenchée par la frappe d'un caractère.
    fn type_char(&mut self, ch: char) {
        if Self::is_cycle(self.field()) {
            return;
        }
        if !self.editing {
            self.editing = true;
            self.buffer.clear();
        }
        self.buffer.push(ch);
    }

    fn cancel(&mut self) {
        self.editing = false;
        self.buffer.clear();
    }

    fn to_options(&self) -> ConvertOptions {
        ConvertOptions {
            fps: self.fps,
            width: self.width,
            start: nonempty(&self.start),
            duration: nonempty(&self.duration),
            colors: self.colors,
            dither: self.dither,
            filter: self.filter,
            loop_count: self.loop_count,
            output: nonempty(&self.output).map(PathBuf::from),
        }
    }
}

// ---------------------------------------------------------------------------
// Gestion des touches
// ---------------------------------------------------------------------------

fn handle_key(terminal: &mut DefaultTerminal, app: &mut App, code: KeyCode) -> Result<()> {
    // En mode édition, le clavier alimente le tampon de saisie.
    if app.options.editing {
        match code {
            KeyCode::Char(c) => app.options.buffer.push(c),
            KeyCode::Backspace => {
                app.options.buffer.pop();
            }
            KeyCode::Enter => app.options.commit(),
            KeyCode::Esc => app.options.cancel(),
            _ => {}
        }
        return Ok(());
    }

    match code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Tab | KeyCode::BackTab => {
            app.focus = match app.focus {
                Focus::Browser => Focus::Options,
                Focus::Options => Focus::Browser,
            }
        }
        KeyCode::Char('c') => do_convert(terminal, app)?,
        _ => match app.focus {
            Focus::Browser => handle_browser(app, code),
            Focus::Options => handle_options(app, code),
        },
    }
    Ok(())
}

fn handle_browser(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('j') | KeyCode::Down => app.browser.next(),
        KeyCode::Char('k') | KeyCode::Up => app.browser.prev(),
        KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => app.browser.up(),
        KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
            if let Some(video) = app.browser.open() {
                app.status = format!("Vidéo : {}", video.display());
                app.selected_video = Some(video);
                app.focus = Focus::Options;
            }
        }
        _ => {}
    }
}

fn handle_options(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('j') | KeyCode::Down => app.options.next(),
        KeyCode::Char('k') | KeyCode::Up => app.options.prev(),
        KeyCode::Char('h') | KeyCode::Left => app.options.cycle(false),
        KeyCode::Char('l') | KeyCode::Right => app.options.cycle(true),
        KeyCode::Enter => app.options.begin_edit(),
        // Taper directement un chiffre (ou : / .) démarre l'édition d'un champ texte.
        KeyCode::Char(ch) if ch.is_ascii_digit() || ch == ':' || ch == '.' => {
            app.options.type_char(ch)
        }
        _ => {}
    }
}

/// Suspend le TUI, lance la conversion ffmpeg (avec progression), puis reprend.
fn do_convert(terminal: &mut DefaultTerminal, app: &mut App) -> Result<()> {
    let Some(video) = app.selected_video.clone() else {
        app.status = "Aucune vidéo sélectionnée.".into();
        return Ok(());
    };
    let opts = app.options.to_options();

    ratatui::restore();
    println!("Conversion de {} …\n", video.display());
    let result = convert::convert(&video, &opts, true);
    match &result {
        Ok(out) => println!("\n✓ gif créé : {}", out.display()),
        Err(e) => eprintln!("\n✗ échec : {e:#}"),
    }
    println!("\nAppuie sur Entrée pour revenir au TUI…");
    let mut buf = String::new();
    let _ = std::io::stdin().read_line(&mut buf);
    *terminal = ratatui::init();

    app.status = match &result {
        Ok(out) => format!("✓ gif créé : {}", out.display()),
        Err(_) => "✗ échec de la conversion (voir le message).".into(),
    };
    Ok(())
}

// ---------------------------------------------------------------------------
// Rendu
// ---------------------------------------------------------------------------

fn ui(f: &mut Frame, app: &mut App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // titre
            Constraint::Min(0),    // corps
            Constraint::Length(1), // statut
            Constraint::Length(1), // aide
        ])
        .split(f.area());

    let title = Line::from(vec![
        Span::styled(
            " gifust ",
            Style::default()
                .fg(Color::Black)
                .bg(ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  vidéo → gif"),
    ]);
    f.render_widget(Paragraph::new(title), rows[0]);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(rows[1]);

    draw_browser(f, app, cols[0]);
    draw_options(f, app, cols[1]);

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!(" {}", app.status),
            Style::default().fg(Color::Yellow),
        ))),
        rows[2],
    );
    draw_help(f, app, rows[3]);
}

fn draw_browser(f: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Focus::Browser;
    let items: Vec<ListItem> = app
        .browser
        .entries
        .iter()
        .map(|e| {
            if e.is_dir {
                ListItem::new(format!("{}/", e.name)).style(
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ListItem::new(format!("  {}", e.name)).style(Style::default().fg(Color::Green))
            }
        })
        .collect();

    let list = List::new(items)
        .block(panel_block(
            &format!(" Dossier : {} ", app.browser.cwd.display()),
            focused,
        ))
        .highlight_style(highlight(focused))
        .highlight_symbol(if focused { "❯ " } else { "  " });
    f.render_stateful_widget(list, area, &mut app.browser.state);
}

fn draw_options(f: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Focus::Options;
    let vid = app
        .selected_video
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "aucune vidéo".into());

    let parts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(5)])
        .split(area);

    let items: Vec<ListItem> = FIELDS
        .iter()
        .enumerate()
        .map(|(i, &field)| {
            let editing = app.options.editing && i == app.options.sel;
            let (value, value_style) = if editing {
                (
                    format!("{}▏", app.options.buffer),
                    Style::default().add_modifier(Modifier::BOLD),
                )
            } else {
                let (text, placeholder) = app.options.display(field);
                let style = if placeholder {
                    // Texte indicatif grisé : ce n'est pas une valeur saisie.
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC)
                } else {
                    Style::default().add_modifier(Modifier::BOLD)
                };
                (text, style)
            };
            let mut spans = vec![
                Span::styled(
                    format!("{:<9}", OptionsForm::label(field)),
                    Style::default().fg(Color::Gray),
                ),
                Span::raw(" "),
                Span::styled(value, value_style),
            ];
            if OptionsForm::is_cycle(field) {
                spans.push(Span::styled("  ‹ ›", Style::default().fg(Color::DarkGray)));
            }
            ListItem::new(Line::from(spans))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.options.sel));
    let list = List::new(items)
        .block(panel_block(&format!(" Options — {vid} "), focused))
        .highlight_style(highlight(focused))
        .highlight_symbol(if focused { "❯ " } else { "  " });
    f.render_stateful_widget(list, parts[0], &mut state);

    // Panneau de description du champ sélectionné.
    let [role, kind, hint] = OptionsForm::describe(app.options.field());
    let desc = vec![
        Line::from(Span::styled(
            role,
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(kind, Style::default().fg(Color::Cyan))),
        Line::from(Span::styled(hint, Style::default().fg(Color::DarkGray))),
    ];
    let title = format!(" {} ", OptionsForm::label(app.options.field()));
    f.render_widget(
        Paragraph::new(desc)
            .block(panel_block(&title, focused))
            .wrap(ratatui::widgets::Wrap { trim: true }),
        parts[1],
    );
}

fn draw_help(f: &mut Frame, app: &App, area: Rect) {
    let help = if app.options.editing {
        "Entrée valider · Échap annuler"
    } else {
        match app.focus {
            Focus::Browser => {
                "j/k naviguer · l/↵ ouvrir · h/⌫ parent · Tab options · c convertir · q quitter"
            }
            Focus::Options => {
                "j/k champ · 0-9 saisir · ↵ éditer · h/l changer · Tab dossier · c convertir · q quitter"
            }
        }
    };
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!(" {help}"),
            Style::default().fg(Color::DarkGray),
        ))),
        area,
    );
}

fn panel_block(title: &str, focused: bool) -> Block<'static> {
    let color = if focused { ACCENT } else { Color::DarkGray };
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(Span::styled(
            title.to_string(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ))
}

fn highlight(focused: bool) -> Style {
    if focused {
        Style::default()
            .bg(ACCENT)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::REVERSED)
    }
}

// ---------------------------------------------------------------------------
// Utilitaires
// ---------------------------------------------------------------------------

fn move_sel(state: &mut ListState, len: usize, delta: i32) {
    if len == 0 {
        return;
    }
    let cur = state.selected().unwrap_or(0) as i32;
    let next = (cur + delta).rem_euclid(len as i32);
    state.select(Some(next as usize));
}

fn nonempty(s: &str) -> Option<String> {
    let t = s.trim();
    (!t.is_empty()).then(|| t.to_string())
}
