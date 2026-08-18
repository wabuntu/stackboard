mod auth;
mod client;
mod nova;

use clap::{Parser, Subcommand};
use client::Session;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use nova::Server;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Cell, Clear, Paragraph, Row, Table, TableState};
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Interactively configure OpenStack credentials and save them to
    /// ~/.config/stackboard/clouds.yaml.
    Setup,
}

fn main() {
    let args = Args::parse();

    if matches!(args.command, Some(Command::Setup)) {
        if let Err(e) = auth::run_setup_wizard() {
            eprintln!("Setup failed: {e}");
            std::process::exit(1);
        }
        return;
    }

    let cloud_auth = match auth::discover() {
        Some(a) => a,
        None => match auth::run_setup_wizard() {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Setup failed: {e}");
                std::process::exit(1);
            }
        },
    };

    eprintln!("Logging in to {} ...", cloud_auth.auth_url);
    let session = match Session::login(&cloud_auth) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Login failed: {e}");
            std::process::exit(1);
        }
    };

    let mut app = App::new(session);
    app.refresh();

    let mut terminal = ratatui::init();
    let res = run(&mut terminal, &mut app);
    ratatui::restore();

    if let Err(e) = res {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

/// Which resource type is currently shown. Only Servers is implemented in
/// this version — the `:` command bar and this enum exist so adding
/// volumes/networks/images later is just a new match arm, matching k9s's
/// resource-switching model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResourceKind {
    Servers,
}

impl ResourceKind {
    fn from_command(s: &str) -> Option<ResourceKind> {
        match s {
            "servers" | "server" | "vm" | "vms" => Some(ResourceKind::Servers),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            ResourceKind::Servers => "servers",
        }
    }
}

struct App {
    session: Session,
    kind: ResourceKind,
    servers: Vec<Server>,
    table_state: TableState,
    command_mode: bool,
    command_buf: String,
    show_detail: Option<usize>,
    status: Option<String>,
    error: Option<String>,
    last_refresh: Instant,
}

impl App {
    fn new(session: Session) -> App {
        App {
            session,
            kind: ResourceKind::Servers,
            servers: Vec::new(),
            table_state: TableState::default(),
            command_mode: false,
            command_buf: String::new(),
            show_detail: None,
            status: None,
            error: None,
            last_refresh: Instant::now(),
        }
    }

    fn refresh(&mut self) {
        match self.kind {
            ResourceKind::Servers => match nova::list_servers(&self.session) {
                Ok(mut servers) => {
                    servers.sort_by(|a, b| a.name.cmp(&b.name));
                    self.servers = servers;
                    self.error = None;
                    if self.table_state.selected().is_none() && !self.servers.is_empty() {
                        self.table_state.select(Some(0));
                    }
                }
                Err(e) => self.error = Some(e),
            },
        }
        self.last_refresh = Instant::now();
    }

    fn move_selection(&mut self, forward: bool) {
        let len = self.servers.len();
        if len == 0 {
            return;
        }
        let i = self.table_state.selected().unwrap_or(0);
        let next = if forward {
            (i + 1) % len
        } else {
            (i + len - 1) % len
        };
        self.table_state.select(Some(next));
    }
}

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> std::io::Result<()> {
    let refresh_interval = Duration::from_secs(15);

    loop {
        terminal.draw(|frame| draw(frame, app))?;

        if event::poll(Duration::from_millis(200))?
            && let Event::Key(key) = event::read()?
        {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if handle_key(app, key.code) {
                return Ok(());
            }
        }

        if !app.command_mode
            && app.show_detail.is_none()
            && app.last_refresh.elapsed() >= refresh_interval
        {
            app.refresh();
        }
    }
}

/// Returns true if the app should quit.
fn handle_key(app: &mut App, code: KeyCode) -> bool {
    if app.command_mode {
        match code {
            KeyCode::Enter => {
                let cmd = app.command_buf.trim().to_string();
                app.command_mode = false;
                app.command_buf.clear();
                match ResourceKind::from_command(&cmd) {
                    Some(kind) => {
                        app.kind = kind;
                        app.table_state.select(None);
                        app.refresh();
                    }
                    None if !cmd.is_empty() => {
                        app.status = Some(format!(
                            "unknown resource: {cmd} (only 'servers' is available so far)"
                        ));
                    }
                    None => {}
                }
            }
            KeyCode::Esc => {
                app.command_mode = false;
                app.command_buf.clear();
            }
            KeyCode::Backspace => {
                app.command_buf.pop();
            }
            KeyCode::Char(c) => app.command_buf.push(c),
            _ => {}
        }
        return false;
    }

    if app.show_detail.is_some() {
        app.show_detail = None;
        return false;
    }

    match code {
        KeyCode::Char('q') | KeyCode::Esc => return true,
        KeyCode::Down => app.move_selection(true),
        KeyCode::Up => app.move_selection(false),
        KeyCode::Enter => app.show_detail = app.table_state.selected(),
        KeyCode::Char(':') => app.command_mode = true,
        KeyCode::Char('r') => app.refresh(),
        _ => {}
    }
    false
}

fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(area);

    draw_header(frame, app, layout[0]);
    match app.kind {
        ResourceKind::Servers => draw_servers(frame, app, layout[1]),
    }
    draw_status(frame, app, layout[2]);

    if let Some(i) = app.show_detail
        && let Some(s) = app.servers.get(i)
    {
        draw_server_detail(frame, s);
    }
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    use ratatui::text::Span;

    let active = app.servers.iter().filter(|s| s.status == "ACTIVE").count();
    let error = app.servers.iter().filter(|s| s.status == "ERROR").count();
    let other = app.servers.len() - active - error;

    let cmd_span = if app.command_mode {
        Span::styled(
            format!(" :{}_ ", app.command_buf),
            Style::default()
                .fg(Color::Rgb(40, 42, 54))
                .bg(Color::Rgb(241, 250, 140))
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            format!(" :{} ", app.kind.label()),
            Style::default()
                .fg(Color::Rgb(40, 42, 54))
                .bg(Color::Rgb(189, 147, 249))
                .add_modifier(Modifier::BOLD),
        )
    };

    let mut spans = vec![
        Span::styled(
            " stackboard ",
            Style::default()
                .fg(Color::Rgb(40, 42, 54))
                .bg(Color::Rgb(255, 121, 198))
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        cmd_span,
        Span::raw("   "),
        Span::styled(
            format!("{active} active"),
            Style::default().fg(Color::Rgb(80, 250, 123)),
        ),
    ];
    if error > 0 {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("{error} error"),
            Style::default()
                .fg(Color::Rgb(255, 85, 85))
                .add_modifier(Modifier::BOLD),
        ));
    }
    if other > 0 {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("{other} other"),
            Style::default().fg(Color::Rgb(98, 114, 164)),
        ));
    }

    frame.render_widget(Line::from(spans), area);
}

fn status_color(status: &str) -> Color {
    match status {
        "ACTIVE" => Color::Rgb(80, 250, 123),
        "ERROR" => Color::Rgb(255, 85, 85),
        "BUILD" | "REBOOT" | "MIGRATING" => Color::Rgb(241, 250, 140),
        "SHUTOFF" | "SUSPENDED" | "PAUSED" => Color::Rgb(98, 114, 164),
        _ => Color::Rgb(248, 248, 242),
    }
}

fn draw_servers(frame: &mut Frame, app: &mut App, area: Rect) {
    let rows: Vec<Row> = app
        .servers
        .iter()
        .map(|s| {
            let color = status_color(&s.status);
            Row::new(vec![
                Cell::from(s.name.clone()).style(Style::default().fg(Color::Rgb(248, 248, 242))),
                Cell::from(format!("● {}", s.status))
                    .style(Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Cell::from(s.flavor.clone()).style(Style::default().fg(Color::Rgb(139, 233, 253))),
                Cell::from(s.addresses.join(", "))
                    .style(Style::default().fg(Color::Rgb(241, 250, 140))),
                Cell::from(s.host.clone().unwrap_or_else(|| "-".to_string()))
                    .style(Style::default().fg(Color::Rgb(189, 147, 249))),
            ])
        })
        .collect();

    let widths = [
        Constraint::Min(20),
        Constraint::Length(12),
        Constraint::Length(14),
        Constraint::Length(28),
        Constraint::Length(16),
    ];
    let table = Table::new(rows, widths)
        .header(
            Row::new(vec!["NAME", "STATUS", "FLAVOR", "ADDRESSES", "HOST"]).style(
                Style::default()
                    .fg(Color::Rgb(255, 121, 198))
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .block(
            Block::bordered()
                .border_style(Style::default().fg(Color::Rgb(98, 114, 164)))
                .title(
                    Line::from(" servers ").style(
                        Style::default()
                            .fg(Color::Rgb(189, 147, 249))
                            .add_modifier(Modifier::BOLD),
                    ),
                ),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::Rgb(68, 71, 90))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(table, area, &mut app.table_state);
}

fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
    let text = if let Some(err) = &app.error {
        format!("error: {err}")
    } else if let Some(status) = &app.status {
        status.clone()
    } else {
        "↑/↓ select  Enter details  r refresh  : switch resource  q quit".to_string()
    };
    let color = if app.error.is_some() {
        Color::Rgb(255, 85, 85)
    } else {
        Color::Rgb(98, 114, 164)
    };
    let style = if app.error.is_some() {
        Style::default().fg(color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(color)
    };
    frame.render_widget(Line::from(text).style(style), area);
}

fn draw_server_detail(frame: &mut Frame, s: &Server) {
    let area = centered_rect(60, 11, frame.area());
    frame.render_widget(Clear, area);
    let color = status_color(&s.status);
    let text = format!(
        "Name:      {}\nID:        {}\nStatus:    ● {}\nFlavor:    {}\nHost:      {}\nAddresses: {}\nCreated:   {}\n\nany key: close",
        s.name,
        s.id,
        s.status,
        s.flavor,
        s.host.as_deref().unwrap_or("-"),
        s.addresses.join(", "),
        s.created,
    );
    let popup = Paragraph::new(text)
        .style(Style::default().fg(Color::Rgb(248, 248, 242)))
        .block(
            Block::bordered()
                .border_style(Style::default().fg(color))
                .title(
                    Line::from(" Server details ")
                        .style(Style::default().fg(color).add_modifier(Modifier::BOLD)),
                ),
        );
    frame.render_widget(popup, area);
}

fn centered_rect(pct_x: u16, rows: u16, area: Rect) -> Rect {
    let rows = rows.min(area.height);
    let vertical = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(rows),
        Constraint::Fill(1),
    ])
    .split(area);
    Layout::horizontal([
        Constraint::Percentage((100 - pct_x) / 2),
        Constraint::Percentage(pct_x),
        Constraint::Percentage((100 - pct_x) / 2),
    ])
    .split(vertical[1])[1]
}
