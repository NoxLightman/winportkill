use eframe::App as EframeApp;
use eframe::egui::{self, Align, Color32, Key, RichText, TextEdit};
use egui_extras::{Column, TableBuilder};
use std::time::{Duration, Instant};
use winportkill_core::{
    PortBinding, PortViewEntry, PortViewStats, ProcessViewEntry, ProcessViewStats, filter_ports,
    filter_processes, kill, port_stats, process_stats, scan_ports, scan_processes,
};

const BG: Color32 = Color32::from_rgb(18, 22, 28);
const PANEL_BG: Color32 = Color32::from_rgb(24, 29, 36);
const HEADER_BG: Color32 = Color32::from_rgb(31, 38, 47);
const SELECTED_BG: Color32 = Color32::from_rgb(34, 66, 96);
const ACCENT: Color32 = Color32::from_rgb(255, 210, 72);
const TCP: Color32 = Color32::from_rgb(93, 201, 255);
const UDP: Color32 = Color32::from_rgb(111, 214, 146);
const MUTED: Color32 = Color32::from_rgb(128, 138, 150);
const MEM: Color32 = Color32::from_rgb(214, 133, 255);
const ERROR: Color32 = Color32::from_rgb(255, 106, 106);
const SUCCESS: Color32 = Color32::from_rgb(99, 214, 104);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ViewMode {
    Ports,
    Processes,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PortRowKey {
    pid: u32,
    proto: String,
    local_addr: String,
    port: String,
}

impl PortRowKey {
    fn from_entry(entry: &PortViewEntry) -> Self {
        Self {
            pid: entry.pid,
            proto: entry.proto.clone(),
            local_addr: entry.local_addr.clone(),
            port: entry.port.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProcessRowKey {
    pid: u32,
}

impl ProcessRowKey {
    fn from_entry(entry: &ProcessViewEntry) -> Self {
        Self { pid: entry.pid }
    }
}

pub struct App {
    view_mode: ViewMode,
    port_entries: Vec<PortViewEntry>,
    filtered_ports: Vec<PortViewEntry>,
    process_entries: Vec<ProcessViewEntry>,
    filtered_processes: Vec<ProcessViewEntry>,
    port_stats: PortViewStats,
    process_stats: ProcessViewStats,
    filter_text: String,
    selected_port_row: Option<PortRowKey>,
    selected_process_row: Option<ProcessRowKey>,
    pending_kill_pid: Option<u32>,
    status_msg: String,
    status_is_error: bool,
    filter_has_focus: bool,
    scroll_to_selected_once: bool,
    last_refresh: Instant,
    refresh_interval: Duration,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            view_mode: ViewMode::Ports,
            port_entries: Vec::new(),
            filtered_ports: Vec::new(),
            process_entries: Vec::new(),
            filtered_processes: Vec::new(),
            port_stats: PortViewStats {
                total_rows: 0,
                total_procs: 0,
                tcp_count: 0,
                udp_count: 0,
                total_mem_bytes: 0,
            },
            process_stats: ProcessViewStats {
                total_procs: 0,
                procs_with_ports: 0,
                total_port_bindings: 0,
                tcp_count: 0,
                udp_count: 0,
                total_mem_bytes: 0,
            },
            filter_text: String::new(),
            selected_port_row: None,
            selected_process_row: None,
            pending_kill_pid: None,
            status_msg: String::new(),
            status_is_error: false,
            filter_has_focus: false,
            scroll_to_selected_once: false,
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(10),
        };
        app.refresh_data(false);
        app
    }

    fn refresh_data(&mut self, preserve_status: bool) {
        self.port_entries = scan_ports();
        self.process_entries = scan_processes();
        self.apply_filter();
        self.port_stats = port_stats(&self.filtered_ports);
        self.process_stats = process_stats(&self.filtered_processes);
        self.last_refresh = Instant::now();
        if !preserve_status {
            self.clear_status();
        }
        self.reconcile_selection();
    }

    fn apply_filter(&mut self) {
        self.filtered_ports = filter_ports(&self.port_entries, &self.filter_text);
        self.filtered_processes = filter_processes(&self.process_entries, &self.filter_text);
    }

    fn reconcile_selection(&mut self) {
        let port_selection_valid = self.selected_port_row.as_ref().is_some_and(|row| {
            self.filtered_ports
                .iter()
                .any(|entry| PortRowKey::from_entry(entry) == *row)
        });
        if !port_selection_valid {
            self.selected_port_row = self.filtered_ports.first().map(PortRowKey::from_entry);
        }

        let process_selection_valid = self.selected_process_row.as_ref().is_some_and(|row| {
            self.filtered_processes
                .iter()
                .any(|entry| ProcessRowKey::from_entry(entry) == *row)
        });
        if !process_selection_valid {
            self.selected_process_row = self
                .filtered_processes
                .first()
                .map(ProcessRowKey::from_entry);
        }

        self.scroll_to_selected_once = self.current_len() > 0;
    }

    fn current_len(&self) -> usize {
        match self.view_mode {
            ViewMode::Ports => self.filtered_ports.len(),
            ViewMode::Processes => self.filtered_processes.len(),
        }
    }

    fn selected_port_index(&self) -> Option<usize> {
        let row = self.selected_port_row.as_ref()?;
        self.filtered_ports
            .iter()
            .position(|entry| PortRowKey::from_entry(entry) == *row)
    }

    fn selected_process_index(&self) -> Option<usize> {
        let row = self.selected_process_row.as_ref()?;
        self.filtered_processes
            .iter()
            .position(|entry| ProcessRowKey::from_entry(entry) == *row)
    }

    fn selected_port_entry(&self) -> Option<&PortViewEntry> {
        let row = self.selected_port_row.as_ref()?;
        self.filtered_ports
            .iter()
            .find(|entry| PortRowKey::from_entry(entry) == *row)
    }

    fn selected_process_entry(&self) -> Option<&ProcessViewEntry> {
        let row = self.selected_process_row.as_ref()?;
        self.filtered_processes
            .iter()
            .find(|entry| ProcessRowKey::from_entry(entry) == *row)
    }

    fn selected_pid(&self) -> Option<u32> {
        match self.view_mode {
            ViewMode::Ports => self.selected_port_entry().map(|entry| entry.pid),
            ViewMode::Processes => self.selected_process_entry().map(|entry| entry.pid),
        }
    }

    fn move_selection(&mut self, delta: isize) {
        match self.view_mode {
            ViewMode::Ports => {
                if self.filtered_ports.is_empty() {
                    self.selected_port_row = None;
                    return;
                }
                let current = self.selected_port_index().unwrap_or(0) as isize;
                let max = self.filtered_ports.len().saturating_sub(1) as isize;
                let next = (current + delta).clamp(0, max) as usize;
                self.selected_port_row = Some(PortRowKey::from_entry(&self.filtered_ports[next]));
            }
            ViewMode::Processes => {
                if self.filtered_processes.is_empty() {
                    self.selected_process_row = None;
                    return;
                }
                let current = self.selected_process_index().unwrap_or(0) as isize;
                let max = self.filtered_processes.len().saturating_sub(1) as isize;
                let next = (current + delta).clamp(0, max) as usize;
                self.selected_process_row =
                    Some(ProcessRowKey::from_entry(&self.filtered_processes[next]));
            }
        }
        self.scroll_to_selected_once = true;
    }

    fn select_first(&mut self) {
        match self.view_mode {
            ViewMode::Ports => {
                self.selected_port_row = self.filtered_ports.first().map(PortRowKey::from_entry);
            }
            ViewMode::Processes => {
                self.selected_process_row = self
                    .filtered_processes
                    .first()
                    .map(ProcessRowKey::from_entry);
            }
        }
        self.scroll_to_selected_once = self.current_len() > 0;
    }

    fn select_last(&mut self) {
        match self.view_mode {
            ViewMode::Ports => {
                self.selected_port_row = self.filtered_ports.last().map(PortRowKey::from_entry);
            }
            ViewMode::Processes => {
                self.selected_process_row = self
                    .filtered_processes
                    .last()
                    .map(ProcessRowKey::from_entry);
            }
        }
        self.scroll_to_selected_once = self.current_len() > 0;
    }

    fn switch_view(&mut self, view_mode: ViewMode) {
        if self.view_mode != view_mode {
            self.view_mode = view_mode;
            self.scroll_to_selected_once = true;
            self.reconcile_selection();
        }
    }

    fn request_kill_selected(&mut self) {
        if let Some(pid) = self.selected_pid() {
            self.pending_kill_pid = Some(pid);
        }
    }

    fn confirm_kill(&mut self) {
        let Some(pid) = self.pending_kill_pid.take() else {
            return;
        };

        match kill(pid) {
            Ok(name) => {
                self.status_msg = format!("Killed PID {} ({})", pid, name);
                self.status_is_error = false;
                self.refresh_data(true);
            }
            Err(msg) => {
                self.status_msg = msg;
                self.status_is_error = true;
                self.refresh_data(true);
            }
        }
    }

    fn cancel_kill(&mut self) {
        self.pending_kill_pid = None;
    }

    fn clear_status(&mut self) {
        self.status_msg.clear();
        self.status_is_error = false;
    }

    fn selected_details_text(&self) -> String {
        match self.view_mode {
            ViewMode::Ports => {
                let Some(entry) = self.selected_port_entry() else {
                    return "No selection".to_string();
                };
                format!(
                    "{}  {}:{}  PID {}  {:.1} MB",
                    entry.proto,
                    entry.local_addr,
                    entry.port,
                    entry.pid,
                    entry.memory as f64 / 1024.0 / 1024.0
                )
            }
            ViewMode::Processes => {
                let Some(entry) = self.selected_process_entry() else {
                    return "No selection".to_string();
                };
                format!(
                    "PID {}  TCP {}  UDP {}  {:.1} MB",
                    entry.pid,
                    entry.tcp_ports,
                    entry.udp_ports,
                    entry.memory as f64 / 1024.0 / 1024.0
                )
            }
        }
    }

    fn current_stats_line(&self) -> (String, String, String, String) {
        match self.view_mode {
            ViewMode::Ports => {
                let stats = &self.port_stats;
                let mem_mb = stats.total_mem_bytes as f64 / 1024.0 / 1024.0;
                let mem = if mem_mb >= 1024.0 {
                    format!("Mem {:.1} GB", mem_mb / 1024.0)
                } else {
                    format!("Mem {:.0} MB", mem_mb)
                };
                (
                    format!("Rows {}", stats.total_rows),
                    format!("TCP {}", stats.tcp_count),
                    format!("UDP {}", stats.udp_count),
                    format!("Proc {}", stats.total_procs),
                )
                    .pipe(|(a, b, c, d)| (a, b, c, format!("{d}  {mem}")))
            }
            ViewMode::Processes => {
                let stats = &self.process_stats;
                let mem_mb = stats.total_mem_bytes as f64 / 1024.0 / 1024.0;
                let mem = if mem_mb >= 1024.0 {
                    format!("Mem {:.1} GB", mem_mb / 1024.0)
                } else {
                    format!("Mem {:.0} MB", mem_mb)
                };
                (
                    format!("Proc {}", stats.total_procs),
                    format!("With Ports {}", stats.procs_with_ports),
                    format!("TCP {}", stats.tcp_count),
                    format!("Bindings {}  {mem}", stats.total_port_bindings),
                )
            }
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if self.pending_kill_pid.is_some() {
            if ctx.input(|i| i.key_pressed(Key::Escape)) {
                self.cancel_kill();
            }
            if ctx.input(|i| i.key_pressed(Key::Enter)) {
                self.confirm_kill();
            }
            return;
        }

        if self.filter_has_focus || ctx.egui_wants_keyboard_input() {
            return;
        }

        if ctx.input(|i| i.key_pressed(Key::Tab)) {
            let next = match self.view_mode {
                ViewMode::Ports => ViewMode::Processes,
                ViewMode::Processes => ViewMode::Ports,
            };
            self.switch_view(next);
            return;
        }

        if ctx.input(|i| i.key_pressed(Key::ArrowDown)) {
            self.move_selection(1);
        }
        if ctx.input(|i| i.key_pressed(Key::ArrowUp)) {
            self.move_selection(-1);
        }
        if ctx.input(|i| i.key_pressed(Key::PageDown)) {
            self.move_selection(10);
        }
        if ctx.input(|i| i.key_pressed(Key::PageUp)) {
            self.move_selection(-10);
        }
        if ctx.input(|i| i.key_pressed(Key::Home)) {
            self.select_first();
        }
        if ctx.input(|i| i.key_pressed(Key::End)) {
            self.select_last();
        }
        if ctx.input(|i| i.key_pressed(Key::F5)) {
            self.refresh_data(false);
            self.status_msg = "Refreshed".to_string();
        }
        if ctx.input(|i| i.key_pressed(Key::K)) {
            self.request_kill_selected();
        }
        if ctx.input(|i| i.key_pressed(Key::Enter)) {
            self.request_kill_selected();
        }
    }

    fn configure_visuals(&self, ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = BG;
        visuals.window_fill = PANEL_BG;
        visuals.extreme_bg_color = HEADER_BG;
        visuals.faint_bg_color = PANEL_BG;
        visuals.selection.bg_fill = SELECTED_BG;
        visuals.selection.stroke.color = ACCENT;
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(37, 45, 55);
        visuals.widgets.active.bg_fill = Color32::from_rgb(47, 57, 70);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(27, 33, 40);
        visuals.widgets.noninteractive.bg_fill = PANEL_BG;
        ctx.set_visuals(visuals);
    }

    fn draw_top_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("top_bar")
            .frame(egui::Frame::new().fill(PANEL_BG).inner_margin(10))
            .show_inside(ui, |ui| {
                let (s1, s2, s3, s4) = self.current_stats_line();

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("WinPortKill")
                            .color(ACCENT)
                            .size(22.0)
                            .strong(),
                    );
                    ui.separator();

                    let ports_selected = self.view_mode == ViewMode::Ports;
                    if ui.selectable_label(ports_selected, "Ports").clicked() {
                        self.switch_view(ViewMode::Ports);
                    }
                    if ui.selectable_label(!ports_selected, "Processes").clicked() {
                        self.switch_view(ViewMode::Processes);
                    }

                    ui.separator();
                    ui.label(RichText::new(s1).color(Color32::WHITE).strong());
                    ui.label(RichText::new(s2).color(TCP).strong());
                    ui.label(RichText::new(s3).color(UDP).strong());
                    ui.label(RichText::new(s4).color(MEM).strong());

                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("Kill").clicked() {
                            self.request_kill_selected();
                        }
                        if ui.button("Refresh").clicked() {
                            self.refresh_data(false);
                            self.status_msg = "Refreshed".to_string();
                        }
                    });
                });
            });
    }

    fn draw_bottom_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("bottom_bar")
            .frame(egui::Frame::new().fill(PANEL_BG).inner_margin(10))
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Filter").color(MUTED));
                    let response = ui.add(
                        TextEdit::singleline(&mut self.filter_text)
                            .hint_text("tcp, 8080, node, 127.0.0.1...")
                            .desired_width(260.0),
                    );
                    self.filter_has_focus = response.has_focus();
                    if response.changed() {
                        self.apply_filter();
                        self.port_stats = port_stats(&self.filtered_ports);
                        self.process_stats = process_stats(&self.filtered_processes);
                        self.reconcile_selection();
                    }

                    ui.separator();
                    ui.label(
                        RichText::new(self.selected_details_text())
                            .color(Color32::from_rgb(180, 190, 205)),
                    );

                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        let status_text = if self.status_msg.is_empty() {
                            "Tab switch view  Up/Down navigate  Enter/K kill  F5 refresh"
                                .to_string()
                        } else {
                            self.status_msg.clone()
                        };
                        let status_color = if self.status_msg.is_empty() {
                            MUTED
                        } else if self.status_is_error {
                            ERROR
                        } else {
                            SUCCESS
                        };
                        ui.label(RichText::new(status_text).color(status_color));
                    });
                });
            });
    }

    fn draw_table(&mut self, ui: &mut egui::Ui) {
        match self.view_mode {
            ViewMode::Ports => self.draw_ports_table(ui),
            ViewMode::Processes => self.draw_processes_table(ui),
        }
    }

    fn draw_ports_table(&mut self, ui: &mut egui::Ui) {
        let scroll_row = if self.scroll_to_selected_once {
            self.selected_port_index()
        } else {
            None
        };

        let mut table = TableBuilder::new(ui)
            .id_salt("ports_table")
            .striped(true)
            .resizable(true)
            .sense(egui::Sense::click())
            .cell_layout(egui::Layout::left_to_right(Align::Center))
            .column(Column::exact(70.0))
            .column(Column::initial(180.0).at_least(120.0))
            .column(Column::exact(70.0))
            .column(Column::exact(80.0))
            .column(Column::exact(90.0))
            .column(Column::remainder().at_least(180.0));

        if let Some(row) = scroll_row {
            table = table.scroll_to_row(row, Some(Align::Center));
        }

        table
            .header(26.0, |mut header| {
                for title in ["Proto", "Addr", "Port", "PID", "Mem(MB)", "Process"] {
                    header.col(|ui| {
                        ui.label(RichText::new(title).color(Color32::WHITE).strong());
                    });
                }
            })
            .body(|body| {
                let entries = &self.filtered_ports;
                let selected_row = self.selected_port_row.clone();
                let mut clicked_row = None;
                let mut double_clicked_row = None;

                body.rows(24.0, entries.len(), |mut row| {
                    let entry = &entries[row.index()];
                    let row_key = PortRowKey::from_entry(entry);
                    row.set_selected(selected_row.as_ref() == Some(&row_key));

                    let proto_color = if entry.proto.starts_with("TCP") {
                        TCP
                    } else {
                        UDP
                    };
                    let mem_mb = entry.memory as f64 / 1024.0 / 1024.0;

                    row.col(|ui| {
                        ui.label(RichText::new(&entry.proto).color(proto_color));
                    });
                    row.col(|ui| {
                        ui.label(RichText::new(&entry.local_addr).color(Color32::WHITE));
                    });
                    row.col(|ui| {
                        ui.label(RichText::new(&entry.port).color(ACCENT));
                    });
                    row.col(|ui| {
                        ui.label(RichText::new(entry.pid.to_string()).color(Color32::WHITE));
                    });
                    row.col(|ui| {
                        ui.label(RichText::new(format!("{mem_mb:.1}")).color(MEM));
                    });
                    row.col(|ui| {
                        ui.label(RichText::new(&entry.name).color(Color32::WHITE));
                    });

                    let response = row.response();
                    if response.clicked() {
                        clicked_row = Some(row_key.clone());
                    }
                    if response.double_clicked() {
                        double_clicked_row = Some(row_key);
                    }
                });

                if let Some(row) = clicked_row {
                    self.selected_port_row = Some(row);
                }
                if let Some(row) = double_clicked_row {
                    self.pending_kill_pid = Some(row.pid);
                    self.selected_port_row = Some(row);
                }
            });

        self.scroll_to_selected_once = false;
    }

    fn draw_processes_table(&mut self, ui: &mut egui::Ui) {
        let scroll_row = if self.scroll_to_selected_once {
            self.selected_process_index()
        } else {
            None
        };

        let mut table = TableBuilder::new(ui)
            .id_salt("processes_table")
            .striped(true)
            .resizable(true)
            .sense(egui::Sense::click())
            .cell_layout(egui::Layout::left_to_right(Align::Center))
            .column(Column::exact(80.0))
            .column(Column::exact(90.0))
            .column(Column::exact(90.0))
            .column(Column::exact(90.0))
            .column(Column::remainder().at_least(240.0));

        if let Some(row) = scroll_row {
            table = table.scroll_to_row(row, Some(Align::Center));
        }

        table
            .header(26.0, |mut header| {
                for title in ["PID", "TCP", "UDP", "Mem(MB)", "Process / Ports"] {
                    header.col(|ui| {
                        ui.label(RichText::new(title).color(Color32::WHITE).strong());
                    });
                }
            })
            .body(|body| {
                let entries = &self.filtered_processes;
                let selected_row = self.selected_process_row.clone();
                let mut clicked_row = None;
                let mut double_clicked_row = None;

                body.rows(26.0, entries.len(), |mut row| {
                    let entry = &entries[row.index()];
                    let row_key = ProcessRowKey::from_entry(entry);
                    row.set_selected(selected_row.as_ref() == Some(&row_key));
                    let mem_mb = entry.memory as f64 / 1024.0 / 1024.0;

                    row.col(|ui| {
                        ui.label(RichText::new(entry.pid.to_string()).color(Color32::WHITE));
                    });
                    row.col(|ui| {
                        ui.label(RichText::new(entry.tcp_ports.to_string()).color(TCP));
                    });
                    row.col(|ui| {
                        ui.label(RichText::new(entry.udp_ports.to_string()).color(UDP));
                    });
                    row.col(|ui| {
                        ui.label(RichText::new(format!("{mem_mb:.1}")).color(MEM));
                    });
                    row.col(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&entry.name).color(Color32::WHITE));
                            ui.label(
                                RichText::new(format_port_summary(&entry.ports))
                                    .color(MUTED)
                                    .small(),
                            );
                        });
                    });

                    let response = row.response();
                    if response.clicked() {
                        clicked_row = Some(row_key.clone());
                    }
                    if response.double_clicked() {
                        double_clicked_row = Some(row_key);
                    }
                });

                if let Some(row) = clicked_row {
                    self.selected_process_row = Some(row);
                }
                if let Some(row) = double_clicked_row {
                    self.pending_kill_pid = Some(row.pid);
                    self.selected_process_row = Some(row);
                }
            });

        self.scroll_to_selected_once = false;
    }

    fn draw_kill_dialog(&mut self, ctx: &egui::Context) {
        let Some(pid) = self.pending_kill_pid else {
            return;
        };

        let name = self
            .process_entries
            .iter()
            .find(|entry| entry.pid == pid)
            .map(|entry| entry.name.clone())
            .unwrap_or_else(|| "unknown".to_string());

        egui::Window::new("Confirm Kill")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .default_width(360.0)
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(format!("Terminate PID {} ({})?", pid, name))
                        .color(Color32::WHITE)
                        .strong(),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new("This will kill the whole process, not just release a port.")
                        .color(MUTED),
                );
                ui.add_space(12.0);
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Kill").clicked() {
                        self.confirm_kill();
                    }
                    if ui.button("Cancel").clicked() {
                        self.cancel_kill();
                    }
                });
            });
    }
}

impl EframeApp for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.configure_visuals(ctx);
        ctx.request_repaint_after(Duration::from_secs(1));

        if self.last_refresh.elapsed() >= self.refresh_interval && self.pending_kill_pid.is_none() {
            self.refresh_data(true);
        }

        self.handle_shortcuts(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        self.draw_top_bar(ui);
        self.draw_bottom_bar(ui);

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BG).inner_margin(10))
            .show_inside(ui, |ui| {
                self.draw_table(ui);
            });

        self.draw_kill_dialog(&ctx);
    }
}

fn format_port_summary(ports: &[PortBinding]) -> String {
    if ports.is_empty() {
        return "No listening ports".to_string();
    }

    ports
        .iter()
        .take(3)
        .map(|binding| format!("{} {}:{}", binding.proto, binding.local_addr, binding.port))
        .collect::<Vec<_>>()
        .join("  |  ")
        .pipe(|text| {
            if ports.len() > 3 {
                format!("{text}  |  +{} more", ports.len() - 3)
            } else {
                text
            }
        })
}

trait Pipe: Sized {
    fn pipe<R>(self, f: impl FnOnce(Self) -> R) -> R {
        f(self)
    }
}

impl<T> Pipe for T {}
