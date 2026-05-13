use crate::app::{App, ViewMode};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, HighlightSpacing, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState, Widget,
    },
};

const ACCENT: Color = Color::Cyan;
const ACCENT_BG: Color = Color::Rgb(30, 60, 90);
const DIM_FG: Color = Color::Rgb(80, 80, 80);
const HEADER_BG: Color = Color::Rgb(40, 44, 52);
const HEADER_FG: Color = Color::White;
const BORDER_ACTIVE: Color = Color::Cyan;
const BORDER_IDLE: Color = Color::Rgb(60, 64, 72);
const TOP_BAR_BG: Color = Color::Rgb(20, 22, 28);
const TCP_COLOR: Color = Color::Cyan;
const UDP_COLOR: Color = Color::Green;
const PORT_COLOR: Color = Color::Yellow;
const MEM_COLOR: Color = Color::Magenta;

struct BulbLogo;

const BULB_BODY: Color = Color::Rgb(255, 220, 50);
const BULB_GLOW: Color = Color::Rgb(255, 255, 150);
const BULB_DARK: Color = Color::Rgb(180, 140, 20);
const BULB_BASE: Color = Color::Rgb(120, 100, 60);
const BULB_BASE_DARK: Color = Color::Rgb(80, 70, 40);

impl Widget for BulbLogo {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        const ART: [&str; 7] = [
            "  xxxx  ",
            " xXxXXx ",
            "xXXXXXXx",
            "xXxXxXXx",
            " xXxXXx ",
            "  xBBx  ",
            "  xDDx  ",
        ];
        const COLORS: [[Color; 8]; 7] = [
            [TOP_BAR_BG, TOP_BAR_BG, BULB_DARK, BULB_BODY, BULB_BODY, BULB_DARK, TOP_BAR_BG, TOP_BAR_BG],
            [TOP_BAR_BG, BULB_DARK, BULB_BODY, BULB_GLOW, BULB_BODY, BULB_GLOW, BULB_DARK, TOP_BAR_BG],
            [BULB_DARK, BULB_BODY, BULB_GLOW, BULB_GLOW, BULB_GLOW, BULB_GLOW, BULB_BODY, BULB_DARK],
            [BULB_DARK, BULB_BODY, BULB_GLOW, BULB_BODY, BULB_GLOW, BULB_BODY, BULB_GLOW, BULB_DARK],
            [TOP_BAR_BG, BULB_DARK, BULB_BODY, BULB_GLOW, BULB_BODY, BULB_GLOW, BULB_DARK, TOP_BAR_BG],
            [TOP_BAR_BG, TOP_BAR_BG, BULB_BASE, BULB_BASE, BULB_BASE, BULB_BASE, TOP_BAR_BG, TOP_BAR_BG],
            [TOP_BAR_BG, TOP_BAR_BG, BULB_BASE_DARK, BULB_BASE_DARK, BULB_BASE_DARK, BULB_BASE_DARK, TOP_BAR_BG, TOP_BAR_BG],
        ];

        for (row_idx, line) in ART.iter().enumerate() {
            let y = area.y + row_idx as u16;
            if y >= area.bottom() {
                break;
            }
            for (col_idx, ch) in line.chars().enumerate() {
                let x = area.x + col_idx as u16;
                if x >= area.right() {
                    break;
                }
                if ch != ' ' {
                    let cell = buf.cell_mut((x, y)).unwrap();
                    cell.set_style(Style::default().bg(COLORS[row_idx][col_idx]));
                    cell.set_char(' ');
                }
            }
        }
    }
}

struct TitleAsciiArt;

impl Widget for TitleAsciiArt {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        const ART: [&str; 4] = [
            "__      ___      ___         _   _  ___ _ _ ",
            " \\ \\    / (_)_ _ | _ \\___ _ _| |_| |/ (_| | |",
            "  \\ \\/\\/ /| | ' \\|  _/ _ | '_|  _| ' <| | | |",
            "   \\_/\\_/ |_|_||_|_| \\___|_|  \\__|_|\\_|_|_|_|",
        ];
        let v_off = if area.height > 4 { (area.height - 4) / 2 + 1 } else { 0 };
        let art_w = ART[0].chars().count() as u16;
        let h_off = if area.width > art_w { (area.width - art_w) / 2 } else { 0 };

        for (row_idx, line) in ART.iter().enumerate() {
            let y = area.y + v_off + row_idx as u16;
            if y >= area.bottom() {
                break;
            }
            let len = line.chars().count();
            for (col_idx, ch) in line.chars().enumerate() {
                let x = area.x + h_off + col_idx as u16;
                if x >= area.right() {
                    break;
                }
                if ch != ' ' {
                    let cell = buf.cell_mut((x, y)).unwrap();
                    let t = col_idx as f32 / len.max(1) as f32;
                    let r = (180.0 + (255.0 - 180.0) * t) as u8;
                    let g = (140.0 + (240.0 - 140.0) * t) as u8;
                    let b = (20.0 + (100.0 - 20.0) * t) as u8;
                    cell.set_style(
                        Style::default()
                            .fg(Color::Rgb(r, g, b))
                            .add_modifier(Modifier::BOLD),
                    );
                    cell.set_char(ch);
                }
            }
        }
    }
}

pub fn draw(f: &mut Frame, app: &App) {
    f.render_widget(
        Block::new().style(Style::default().bg(TOP_BAR_BG)),
        f.area(),
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(f.area());

    draw_top_bar(f, app, chunks[0]);
    draw_filter(f, app, chunks[1]);
    draw_table(f, app, chunks[2]);
    draw_status(f, app, chunks[3]);
}

fn draw_top_bar(f: &mut Frame, app: &App, area: Rect) {
    let stats_lines = match app.view_mode {
        ViewMode::Ports => {
            let mem_mb = app.port_stats.total_mem_bytes as f64 / 1024.0 / 1024.0;
            vec![
                Line::from(vec![
                    Span::styled("\u{1F527} ", Style::default()),
                    Span::styled(
                        format!("Rows:{}", app.port_stats.total_rows),
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("\u{1F4CB} ", Style::default()),
                    Span::styled(
                        format!("Proc:{}", app.port_stats.total_procs),
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("\u{1F310} ", Style::default()),
                    Span::styled(
                        format!("TCP:{}", app.port_stats.tcp_count),
                        Style::default().fg(TCP_COLOR).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled(
                        format!("UDP:{}", app.port_stats.udp_count),
                        Style::default().fg(UDP_COLOR).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("\u{1F4BE} ", Style::default()),
                    Span::styled(
                        format!("Mem:{:.1} GB", mem_mb / 1024.0),
                        Style::default().fg(MEM_COLOR).add_modifier(Modifier::BOLD),
                    ),
                ]),
            ]
        }
        ViewMode::Processes => {
            let mem_mb = app.process_stats.total_mem_bytes as f64 / 1024.0 / 1024.0;
            vec![
                Line::from(vec![
                    Span::styled("\u{1F527} ", Style::default()),
                    Span::styled(
                        format!("Procs:{}", app.process_stats.total_procs),
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("\u{1F4CB} ", Style::default()),
                    Span::styled(
                        format!("Active:{}", app.process_stats.procs_with_ports),
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("\u{1F310} ", Style::default()),
                    Span::styled(
                        format!("TCP:{}", app.process_stats.tcp_count),
                        Style::default().fg(TCP_COLOR).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled(
                        format!("UDP:{}", app.process_stats.udp_count),
                        Style::default().fg(UDP_COLOR).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("\u{1F4BE} ", Style::default()),
                    Span::styled(
                        format!("Mem:{:.1} GB", mem_mb / 1024.0),
                        Style::default().fg(MEM_COLOR).add_modifier(Modifier::BOLD),
                    ),
                ]),
            ]
        }
    };

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(10),
            Constraint::Length(48),
            Constraint::Length(20),
            Constraint::Length(56),
        ])
        .split(area);

    let y_off = if cols[0].height > 7 { (cols[0].height - 7) / 2 } else { 0 };
    let logo_area = Rect::new(cols[0].x + 1, cols[0].y + y_off, 8, 7);
    BulbLogo.render(logo_area, f.buffer_mut());

    TitleAsciiArt.render(cols[1], f.buffer_mut());

    let mid_y = if cols[2].height > 4 { (cols[2].height - 4) / 2 + 1 } else { 0 };
    let stats = ratatui::widgets::Paragraph::new(stats_lines)
        .style(Style::default().bg(TOP_BAR_BG))
        .alignment(Alignment::Left);
    let stats_area = Rect::new(
        cols[2].x + 1,
        cols[2].y + mid_y,
        cols[2].width.saturating_sub(1),
        4,
    );
    f.render_widget(stats, stats_area);

    render_figlet_clock(cols[3], f.buffer_mut());
}

fn render_figlet_clock(area: Rect, buf: &mut ratatui::buffer::Buffer) {
    if area.width < 20 || area.height < 3 {
        return;
    }

    use std::sync::LazyLock;
    use std::time::{SystemTime, UNIX_EPOCH};

    static FONT: LazyLock<figlet_rs::FIGlet> =
        LazyLock::new(|| figlet_rs::FIGlet::standard().unwrap());

    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let s = d.as_secs();
    let h = ((s / 3600 % 24) + 8) % 24;
    let m = s / 60 % 60;
    let sec = s % 60;
    let time_str = format!("{:02}:{:02}:{:02}", h, m, sec);

    let figure = match FONT.convert(&time_str) {
        Some(f) => f,
        None => return,
    };
    let ascii = figure.to_string();
    let lines: Vec<&str> = ascii.lines().collect();
    let max_width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1);

    let art_h = lines.len() as u16;
    let y_off = if area.height > art_h { (area.height - art_h) / 2 + 1 } else { 0 };
    let x_off = if area.width as usize > max_width {
        (area.width as usize - max_width) / 2
    } else {
        0
    };

    for (row_idx, line) in lines.iter().enumerate() {
        let y = area.y + y_off + row_idx as u16;
        if y >= area.bottom() {
            break;
        }
        for (col_idx, ch) in line.chars().enumerate() {
            if ch == ' ' {
                continue;
            }
            let x = area.x + (x_off + col_idx) as u16;
            if x >= area.right() {
                break;
            }
            let cell = buf.cell_mut((x, y)).unwrap();
            let ratio = col_idx as f32 / max_width as f32;
            let hue = ((ratio * 360.0) + sec as f32 * 6.0) % 360.0;
            let h_rad = hue * std::f32::consts::PI / 180.0;
            cell.set_style(Style::default().fg(Color::Rgb(
                (128.0 + 127.0 * h_rad.cos()) as u8,
                (128.0 + 127.0 * (h_rad + 2.094).cos()) as u8,
                (128.0 + 127.0 * (h_rad + 4.189).cos()) as u8,
            )));
            cell.set_char(ch);
        }
    }
}

fn draw_filter(f: &mut Frame, app: &App, area: Rect) {
    let border_color = if app.filter_active {
        BORDER_ACTIVE
    } else {
        BORDER_IDLE
    };

    let (filter_text, text_style) = if app.filter.is_empty() && !app.filter_active {
        (
            "Press / to filter...".to_string(),
            Style::default().fg(DIM_FG),
        )
    } else {
        (
            app.filter.clone(),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let paragraph = ratatui::widgets::Paragraph::new(Line::from(vec![
        Span::styled(" / ", Style::default().fg(ACCENT)),
        Span::styled(filter_text, text_style),
    ]))
    .block(block);
    f.render_widget(paragraph, area);
}

fn draw_table(f: &mut Frame, app: &App, area: Rect) {
    match app.view_mode {
        ViewMode::Ports => draw_ports_table(f, app, area),
        ViewMode::Processes => draw_processes_table(f, app, area),
    }
}

fn draw_ports_table(f: &mut Frame, app: &App, area: Rect) {
    let total = app.filtered_ports.len();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER_IDLE))
        .title(format!(" Ports ({total}) "))
        .title_style(Style::default().fg(Color::White));

    let constraints = [
        Constraint::Length(7),
        Constraint::Length(16),
        Constraint::Length(7),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Min(10),
    ];

    let header = header_row(["Proto", "Addr", "Port", "PID", "Mem(MB)", "Process"]);
    let rows: Vec<Row> = app
        .filtered_ports
        .iter()
        .map(|entry| {
            let mem_mb = entry.memory as f64 / 1024.0 / 1024.0;
            Row::new(vec![
                Cell::new(entry.proto.clone()).style(Style::default().fg(
                    if entry.proto.starts_with("TCP") {
                        TCP_COLOR
                    } else {
                        UDP_COLOR
                    },
                )),
                Cell::new(entry.local_addr.clone()).style(Style::default().fg(Color::White)),
                Cell::new(entry.port.clone()).style(Style::default().fg(PORT_COLOR)),
                Cell::new(entry.pid.to_string()).style(Style::default().fg(Color::White)),
                Cell::new(format!("{mem_mb:.1}")).style(Style::default().fg(MEM_COLOR)),
                Cell::new(entry.name.clone()).style(Style::default().fg(Color::White)),
            ])
        })
        .collect();

    render_table(
        f,
        area,
        block,
        constraints,
        header,
        rows,
        app.selected,
        total,
    );
}

fn draw_processes_table(f: &mut Frame, app: &App, area: Rect) {
    let total = app.filtered_processes.len();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER_IDLE))
        .title(format!(" Processes ({total}) "))
        .title_style(Style::default().fg(Color::White));

    let constraints = [
        Constraint::Length(8),
        Constraint::Length(6),
        Constraint::Length(6),
        Constraint::Length(10),
        Constraint::Min(20),
    ];

    let header = header_row(["PID", "TCP", "UDP", "Mem(MB)", "Process / Ports"]);
    let rows: Vec<Row> = app
        .filtered_processes
        .iter()
        .map(|entry| {
            let mem_mb = entry.memory as f64 / 1024.0 / 1024.0;
            Row::new(vec![
                Cell::new(entry.pid.to_string()).style(Style::default().fg(Color::White)),
                Cell::new(entry.tcp_ports.to_string()).style(Style::default().fg(TCP_COLOR)),
                Cell::new(entry.udp_ports.to_string()).style(Style::default().fg(UDP_COLOR)),
                Cell::new(format!("{mem_mb:.1}")).style(Style::default().fg(MEM_COLOR)),
                Cell::new(format_process_summary(entry)).style(Style::default().fg(Color::White)),
            ])
        })
        .collect();

    render_table(
        f,
        area,
        block,
        constraints,
        header,
        rows,
        app.selected,
        total,
    );
}

fn header_row<'a, const N: usize>(headers: [&'a str; N]) -> Row<'a> {
    let cells = headers.into_iter().map(|header| {
        Cell::new(header).style(
            Style::default()
                .fg(HEADER_FG)
                .bg(HEADER_BG)
                .add_modifier(Modifier::BOLD),
        )
    });
    Row::new(cells).style(Style::default().bg(HEADER_BG))
}

fn render_table<const N: usize>(
    f: &mut Frame,
    area: Rect,
    block: Block<'static>,
    constraints: [Constraint; N],
    header: Row<'static>,
    rows: Vec<Row<'static>>,
    selected: usize,
    total: usize,
) {
    let table = Table::new(rows, constraints)
        .header(header)
        .block(block)
        .highlight_symbol("\u{2503} ")
        .highlight_spacing(HighlightSpacing::Always)
        .row_highlight_style(
            Style::default()
                .bg(ACCENT_BG)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = TableState::default();
    if total > 0 {
        state.select(Some(selected));
        let visible_rows = area.height.saturating_sub(4) as usize;
        if visible_rows > 0 {
            let mid = visible_rows / 2;
            let offset = selected.saturating_sub(mid);
            *state.offset_mut() = offset;
        }
    }
    f.render_stateful_widget(table, area, &mut state);

    if total > 0 {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::default().fg(ACCENT))
            .track_style(Style::default().fg(Color::Rgb(40, 44, 52)));
        let mut scroll_state = ScrollbarState::new(total).position(selected);
        f.render_stateful_widget(scrollbar, area, &mut scroll_state);
    }
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let keys = if app.filter_active {
        vec![("Enter/Esc", "Stop"), ("Type", "Filter")]
    } else {
        vec![
            ("q", "Quit"),
            ("k", "Kill"),
            ("/", "Filter"),
            ("r", "Refresh"),
            ("Tab", "View"),
            ("\u{2191}\u{2193}", "Nav"),
            ("PgUp/PgDn", "Jump"),
        ]
    };

    let view_indicator = match app.view_mode {
        ViewMode::Ports => "[Ports] Processes",
        ViewMode::Processes => "Ports [Processes]",
    };

    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::styled(
        view_indicator,
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::raw("  "));
    for (i, (key, desc)) in keys.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" "));
        }
        spans.push(Span::styled(
            format!("[{key}]"),
            Style::default()
                .fg(Color::Rgb(150, 150, 160))
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {desc}"),
            Style::default().fg(DIM_FG),
        ));
    }

    let line = if app.status_msg.is_empty() {
        Line::from(spans)
    } else {
        let msg_color = if app.status_msg.contains("Failed") || app.status_msg.contains("not found")
        {
            Color::Red
        } else {
            Color::Green
        };
        let mut msg_spans = vec![
            Span::styled(
                format!(" {} ", app.status_msg),
                Style::default().fg(msg_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled("|", Style::default().fg(Color::Rgb(60, 64, 72))),
            Span::raw(" "),
        ];
        msg_spans.extend(spans);
        Line::from(msg_spans)
    };

    let paragraph = ratatui::widgets::Paragraph::new(line)
        .style(Style::default().bg(Color::Rgb(28, 30, 36)))
        .alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

fn format_process_summary(entry: &winportkill_core::ProcessViewEntry) -> String {
    if entry.ports.is_empty() {
        return format!("{}  |  No listening ports", entry.name);
    }

    let preview = entry
        .ports
        .iter()
        .take(2)
        .map(|port| format!("{} {}:{}", port.proto, port.local_addr, port.port))
        .collect::<Vec<_>>()
        .join(" | ");

    if entry.ports.len() > 2 {
        format!(
            "{}  |  {} | +{} more",
            entry.name,
            preview,
            entry.ports.len() - 2
        )
    } else {
        format!("{}  |  {}", entry.name, preview)
    }
}
