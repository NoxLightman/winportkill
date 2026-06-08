mod app;
mod ui;

use app::App;
use clap::{ArgGroup, Args, Parser, Subcommand};
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::time::{Duration, Instant};
use winportkill_core::{PortViewEntry, kill, scan_ports};

#[derive(Parser)]
#[command(
    name = "winportkill",
    version,
    about = "Inspect and manage Windows x64 listening ports and processes."
)]
struct Cli {
    /// Start HTTP server mode for IDE integrations.
    #[arg(long)]
    serve: Option<u16>,

    /// Print one JSON snapshot and exit.
    #[arg(long)]
    json: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// List listening ports.
    List(OutputArgs),
    /// Show which process owns a port.
    WhoUses(WhoUsesArgs),
    /// Kill a process by PID or by listening port.
    Kill(KillArgs),
}

#[derive(Args)]
struct OutputArgs {
    /// Print JSON instead of a text table.
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct WhoUsesArgs {
    /// Listening port to inspect.
    port: u16,

    /// Print JSON instead of a text table.
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
#[command(group(ArgGroup::new("target").args(["pid", "port"]).required(true).multiple(false)))]
struct KillArgs {
    /// Kill by PID.
    #[arg(long)]
    pid: Option<u32>,

    /// Kill the process listening on this port.
    #[arg(long)]
    port: Option<u16>,

    /// Print JSON instead of a text result.
    #[arg(long)]
    json: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        handle_command(command)?;
        return Ok(());
    }

    if cli.json {
        print_ports_json(&scan_ports())?;
        return Ok(());
    }

    if let Some(port) = cli.serve {
        run_server(port)?;
        return Ok(());
    }

    run_tui()?;
    Ok(())
}

fn handle_command(command: Command) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Command::List(args) => {
            let entries = scan_ports();
            if args.json {
                print_ports_json(&entries)?;
            } else {
                print_ports_text(&entries);
            }
        }
        Command::WhoUses(args) => {
            let entries = find_entries_by_port(scan_ports(), args.port);
            if args.json {
                println!("{}", serde_json::to_string_pretty(&entries)?);
            } else if entries.is_empty() {
                println!("No listening process found on port {}", args.port);
            } else {
                print_ports_text(&entries);
            }
        }
        Command::Kill(args) => {
            let outcome = if let Some(pid) = args.pid {
                kill_process(pid)
            } else if let Some(port) = args.port {
                kill_process_by_port(port)
            } else {
                unreachable!("clap guarantees one of --pid/--port is provided")
            };

            match (outcome, args.json) {
                (Ok(result), true) => {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
                (Ok(result), false) => {
                    println!("{}", result.message);
                }
                (Err(error), true) => {
                    let result = KillCommandResult {
                        success: false,
                        pid: None,
                        port: args.port,
                        process_name: None,
                        message: error,
                    };
                    println!("{}", serde_json::to_string_pretty(&result)?);
                    std::process::exit(1);
                }
                (Err(error), false) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

fn run_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    println!("WinPortKill server running at http://{}", addr);
    println!("API: /ports  /processes  /stats/ports  /stats/processes  /kill/:pid  /ws");
    tokio::runtime::Runtime::new()?.block_on(async {
        let app = winportkill_server::create_app();
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    })?;
    Ok(())
}

fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    execute!(std::io::stdout(), EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    let ui_tick = Duration::from_secs(1);
    let data_tick = Duration::from_secs(10);
    let mut last_ui_tick = Instant::now();
    let mut last_data_tick = Instant::now();

    while !app.should_quit {
        terminal.draw(|f| ui::draw(f, &app))?;

        let timeout = ui_tick.saturating_sub(last_ui_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key);
                }
            }
        }

        if last_ui_tick.elapsed() >= ui_tick {
            last_ui_tick = Instant::now();
        }

        if last_data_tick.elapsed() >= data_tick {
            app.refresh();
            last_data_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn print_ports_json(entries: &[PortViewEntry]) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(entries)?);
    Ok(())
}

fn print_ports_text(entries: &[PortViewEntry]) {
    if entries.is_empty() {
        println!("No listening ports found.");
        return;
    }

    println!(
        "{:<6} {:<39} {:<7} {:<8} {:<10} Process",
        "Proto", "Addr", "Port", "PID", "Mem(MB)"
    );
    for entry in entries {
        let mem_mb = entry.memory as f64 / 1024.0 / 1024.0;
        println!(
            "{:<6} {:<39} {:<7} {:<8} {:<10.1} {}",
            entry.proto, entry.local_addr, entry.port, entry.pid, mem_mb, entry.name
        );
    }
}

fn find_entries_by_port(entries: Vec<PortViewEntry>, port: u16) -> Vec<PortViewEntry> {
    let port = port.to_string();
    entries
        .into_iter()
        .filter(|entry| entry.port == port)
        .collect()
}

fn resolve_pid_by_port(entries: &[PortViewEntry], port: u16) -> Result<u32, String> {
    let matches = find_entries_by_port(entries.to_vec(), port);
    if matches.is_empty() {
        return Err(format!("No listening process found on port {port}"));
    }

    let pid = matches[0].pid;
    if matches.iter().any(|entry| entry.pid != pid) {
        return Err(format!(
            "Port {port} is associated with multiple processes; refusing to guess a PID"
        ));
    }

    Ok(pid)
}

fn kill_process(pid: u32) -> Result<KillCommandResult, String> {
    kill(pid).map(|process_name| KillCommandResult {
        success: true,
        pid: Some(pid),
        port: None,
        process_name: Some(process_name.clone()),
        message: format!("Killed PID {pid} ({process_name})"),
    })
}

fn kill_process_by_port(port: u16) -> Result<KillCommandResult, String> {
    let entries = scan_ports();
    let pid = resolve_pid_by_port(&entries, port)?;
    let result = kill_process(pid)?;
    Ok(KillCommandResult {
        port: Some(port),
        ..result
    })
}

#[derive(serde::Serialize)]
struct KillCommandResult {
    success: bool,
    pid: Option<u32>,
    port: Option<u16>,
    process_name: Option<String>,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::{find_entries_by_port, resolve_pid_by_port};
    use winportkill_core::PortViewEntry;

    fn entry(pid: u32, port: &str) -> PortViewEntry {
        PortViewEntry {
            proto: "TCP".to_string(),
            local_addr: "127.0.0.1".to_string(),
            port: port.to_string(),
            pid,
            name: format!("proc-{pid}"),
            memory: 0,
        }
    }

    #[test]
    fn filters_entries_by_port() {
        let entries = vec![entry(100, "3000"), entry(101, "8080"), entry(102, "3000")];
        let matched = find_entries_by_port(entries, 3000);
        assert_eq!(matched.len(), 2);
        assert!(matched.iter().all(|entry| entry.port == "3000"));
    }

    #[test]
    fn resolves_pid_when_port_has_single_owner() {
        let entries = vec![entry(100, "3000"), entry(100, "3000")];
        let pid = resolve_pid_by_port(&entries, 3000).unwrap();
        assert_eq!(pid, 100);
    }

    #[test]
    fn rejects_unknown_port() {
        let entries = vec![entry(100, "3000")];
        let error = resolve_pid_by_port(&entries, 8080).unwrap_err();
        assert!(error.contains("No listening process found on port 8080"));
    }

    #[test]
    fn rejects_multi_pid_port_resolution() {
        let entries = vec![entry(100, "3000"), entry(101, "3000")];
        let error = resolve_pid_by_port(&entries, 3000).unwrap_err();
        assert!(error.contains("multiple processes"));
    }
}
