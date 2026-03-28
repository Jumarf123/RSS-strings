use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use eframe::egui::{self, Context, RichText};
use egui::containers::menu::MenuButton;
use flume::Receiver;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::gui::{
    ProcessPickerState, ResultsTableState, StringsInputState, apply_theme, process_picker_ui,
    results_table_ui, strings_input_ui,
};
use crate::i18n::{LANGUAGES_LABEL, LANGUAGE_ORDER, OPTIONS_LABEL, Language, UiText};
use crate::process::{ProcessInfo, enumerate_processes};
use crate::scanner::{
    ScanRequest, ScannerEvent, ScannerHandle, ScannerProgress, SearchResult, SearchSettings,
    StartScanError,
};
use crate::utils::{admin, text::parse_input_lines};

pub struct RssStringsApp {
    processes: Vec<ProcessInfo>,
    picker: ProcessPickerState,
    search_settings: SearchSettings,
    enable_debug_privilege: bool,
    input_state: StringsInputState,
    results_state: ResultsTableState,
    results: Vec<SearchResult>,
    status: StatusMessage,
    progress: ScannerProgress,
    scanner: Option<ScannerHandle>,
    process_rx: Receiver<Vec<ProcessInfo>>,
    stop_flag: Arc<AtomicBool>,
    watcher_join: Option<thread::JoinHandle<()>>,
    last_error: Option<String>,
    config_dirty: bool,
    config: AppConfig,
    language: Language,
}

#[derive(Clone, Debug)]
enum StatusMessage {
    Ready,
    SelectedPid(u32),
    StartError,
    ProcessGone,
    Stopped,
    Done,
}

impl StatusMessage {
    fn to_string(&self, text: &UiText) -> String {
        match self {
            StatusMessage::Ready => text.status_ready().to_string(),
            StatusMessage::SelectedPid(pid) => text.status_selected_pid(*pid),
            StatusMessage::StartError => text.status_start_error().to_string(),
            StatusMessage::ProcessGone => text.status_process_gone().to_string(),
            StatusMessage::Stopped => text.status_stopped().to_string(),
            StatusMessage::Done => text.status_done().to_string(),
        }
    }
}

impl RssStringsApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_theme(&cc.egui_ctx);
        cc.egui_ctx.set_theme(egui::Theme::Dark);

        let config = AppConfig::load().unwrap_or_default();
        let language = config
            .language
            .unwrap_or_else(Language::from_system_or_english);
        let (process_rx, stop_flag, watcher_join) = spawn_process_watcher();

        if config.enable_debug_privilege {
            if let Err(err) = admin::enable_debug_privilege(true) {
                tracing::warn!("Не удалось включить SeDebugPrivilege: {err:?}");
            }
        }

        Self {
            processes: Vec::new(),
            picker: ProcessPickerState::default(),
            search_settings: config.search_settings.clone(),
            enable_debug_privilege: config.enable_debug_privilege,
            input_state: StringsInputState::default(),
            results_state: ResultsTableState::default(),
            results: Vec::new(),
            status: StatusMessage::Ready,
            progress: ScannerProgress::default(),
            scanner: None,
            process_rx,
            stop_flag,
            watcher_join: Some(watcher_join),
            last_error: None,
            config_dirty: false,
            config,
            language,
        }
    }

    fn start_scan(&mut self) {
        if self.scanner.is_some() {
            return;
        }
        let pid = match self.picker.selected_pid {
            Some(pid) => pid,
            None => {
                self.status = StatusMessage::Ready;
                return;
            }
        };
        let queries = parse_input_lines(&self.input_state.value, self.input_state.limit);
        if queries.is_empty() {
            self.status = StatusMessage::Ready;
            return;
        }

        let process_name = self
            .processes
            .iter()
            .find(|p| p.pid == pid)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("PID {pid}"));

        let request = ScanRequest {
            pid,
            process_name,
            queries,
            settings: self.search_settings.clone(),
        };

        match crate::scanner::start_scan(request) {
            Ok(handle) => {
                self.results.clear();
                self.results_state.selected.clear();
                self.progress = ScannerProgress::default();
                self.scanner = Some(handle);
                self.status = StatusMessage::Ready;
            }
            Err(err) => {
                let text = UiText::new(self.language);
                self.last_error = Some(Self::start_scan_error_text(&text, &err));
                self.status = StatusMessage::StartError;
            }
        }
    }

    fn stop_scan(&mut self) {
        if let Some(handle) = &self.scanner {
            handle.cancel();
        }
    }

    fn drain_process_updates(&mut self) {
        for update in self.process_rx.try_iter() {
            self.processes = update;
            if let Some(selected) = self.picker.selected_pid {
                if !self.processes.iter().any(|p| p.pid == selected) {
                    self.status = StatusMessage::ProcessGone;
                }
            }
        }
    }

    fn drain_scanner_events(&mut self, ctx: &Context) {
        let mut finished = false;
        let mut cancelled = false;
        if let Some(handle) = &self.scanner {
            for ev in handle.receiver.try_iter() {
                match ev {
                    ScannerEvent::Progress(p) => {
                        self.progress = p;
                    }
                    ScannerEvent::Results(batch) => {
                        self.results.extend(batch);
                    }
                    ScannerEvent::Finished { cancelled: c } => {
                        finished = true;
                        cancelled = c;
                    }
                }
            }
        }
        if finished {
            self.scanner = None;
            self.status = if cancelled {
                StatusMessage::Stopped
            } else {
                StatusMessage::Done
            };
        }
        ctx.request_repaint();
    }

    fn save_config(&mut self) {
        if !self.config_dirty {
            return;
        }
        self.config.search_settings = self.search_settings.clone();
        self.config.enable_debug_privilege = self.enable_debug_privilege;
        if let Err(err) = self.config.save() {
            error!("Не удалось сохранить конфиг: {err:?}");
        } else {
            self.config_dirty = false;
        }
    }

    fn start_scan_error_text(text: &UiText, err: &StartScanError) -> String {
        match err {
            StartScanError::EmptyList => text.error_empty_list().to_string(),
            StartScanError::NoValidQueries => text.error_no_valid_queries().to_string(),
            StartScanError::MatcherBuild(_) => text.error_matcher_build().to_string(),
            StartScanError::OpenProcess { pid, .. } => text.error_open_process(*pid),
            StartScanError::Regions(_) => text.error_regions().to_string(),
        }
    }
}

impl eframe::App for RssStringsApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.drain_process_updates();
        self.drain_scanner_events(ctx);

        let scanning = self.scanner.is_some();
        let mut text = UiText::new(self.language);
        let mut status_line = self.compose_status_line(&text);

        let toolbar_frame = egui::Frame::NONE
            .fill(ctx.style().visuals.panel_fill)
            .stroke(ctx.style().visuals.window_stroke)
            .inner_margin(egui::Margin::symmetric(10, 8));

        egui::TopBottomPanel::top("top_bar")
            .frame(toolbar_frame.clone())
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    let mut language_changed = false;
                    ui.horizontal(|ui| {
                        ui.heading("RSS-strings");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if options_menu(ui, &mut self.language) {
                                language_changed = true;
                            }
                        });
                    });
                    if language_changed {
                        self.config.language = Some(self.language);
                        self.config_dirty = true;
                        text = UiText::new(self.language);
                        status_line = self.compose_status_line(&text);
                    }
                    ui.add_space(6.0);
                    egui::ScrollArea::horizontal()
                        .id_salt("toolbar_controls")
                        .scroll_bar_visibility(
                            egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                        )
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                if toolbar_button(ui, text.button_refresh()).clicked() {
                                    self.refresh_processes_now();
                                }
                                let selected =
                                    process_picker_ui(ui, &mut self.picker, &self.processes, &text);
                                if selected.is_some() {
                                    self.status = StatusMessage::SelectedPid(selected.unwrap());
                                }
                                if toolbar_button(ui, text.button_find()).clicked() {
                                    self.refresh_processes_now();
                                }
                                ui.separator();
                                ui.add_enabled(
                                    !scanning,
                                    toolbar_button_widget(
                                        ui,
                                        RichText::new(text.button_scan()).strong(),
                                    ),
                                )
                                .clicked()
                                .then(|| self.start_scan());
                                ui.add_enabled(scanning, toolbar_button_widget(ui, text.button_stop()))
                                    .clicked()
                                    .then(|| self.stop_scan());
                            });
                        });
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(text.status_line(&status_line, self.results.len()))
                                .color(egui::Color32::GRAY),
                        );
                        if scanning {
                            ui.separator();
                            ui.label(text.label_searching());
                        }
                    });
                });
            });

        egui::TopBottomPanel::bottom("status_bar")
            .frame(toolbar_frame)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(text.progress_line(
                            self.progress.processed_regions,
                            self.progress.total_regions,
                            self.progress.matches,
                        ))
                        .color(egui::Color32::GRAY),
                    );
                    if scanning && toolbar_button(ui, text.button_cancel()).clicked() {
                        self.stop_scan();
                    }
                    if let Some(err) = &self.last_error {
                        ui.add_space(12.0);
                        ui.colored_label(egui::Color32::LIGHT_RED, text.error_with(err));
                        if toolbar_button(ui, text.button_hide()).clicked() {
                            self.last_error = None;
                        }
                    }
                });
            });

        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(420.0)
            .min_width(320.0)
            .show(ctx, |ui| {
                ui.add_space(6.0);
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(text.header_search_settings()).strong());
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(text.label_min_length());
                            if ui
                                .add(egui::Slider::new(
                                    &mut self.search_settings.min_length,
                                    2..=32,
                                ))
                                .changed()
                            {
                                self.config_dirty = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            if ui
                                .checkbox(
                                    &mut self.search_settings.detect_unicode,
                                    text.label_detect_unicode(),
                                )
                                .changed()
                            {
                                self.config_dirty = true;
                            }
                            if ui
                                .checkbox(
                                    &mut self.search_settings.extended_unicode,
                                    text.label_extended_unicode(),
                                )
                                .changed()
                            {
                                self.config_dirty = true;
                            }
                        });
                        ui.horizontal_wrapped(|ui| {
                            if ui
                                .checkbox(&mut self.search_settings.include_private, text.label_private())
                                .changed()
                            {
                                self.config_dirty = true;
                            }
                            if ui
                                .checkbox(&mut self.search_settings.include_image, text.label_image())
                                .changed()
                            {
                                self.config_dirty = true;
                            }
                            if ui
                                .checkbox(&mut self.search_settings.include_mapped, text.label_mapped())
                                .changed()
                            {
                                self.config_dirty = true;
                            }
                        });
                        let checkbox = ui.checkbox(
                            &mut self.enable_debug_privilege,
                            text.label_enable_debug_privilege(),
                        );
                        if checkbox.changed() {
                            match admin::enable_debug_privilege(self.enable_debug_privilege) {
                                Ok(_) => self.config_dirty = true,
                                Err(err) => {
                                    self.last_error = Some(err.to_string());
                                    self.enable_debug_privilege = false;
                                }
                            }
                        }
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(text.label_settings_saved())
                                .small()
                                .color(egui::Color32::GRAY),
                        );
                    });
                });

                ui.add_space(10.0);

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(text.header_strings_input()).strong());
                        let count = strings_input_ui(ui, &mut self.input_state, &text);
                        ui.horizontal(|ui| {
                            ui.label(text.label_lines_count(count, self.input_state.limit));
                            if count == 0 {
                                ui.colored_label(
                                    egui::Color32::LIGHT_RED,
                                    text.label_no_lines(),
                                );
                            } else if count >= self.input_state.limit {
                                ui.colored_label(
                                    egui::Color32::YELLOW,
                                    text.label_trimmed(self.input_state.limit),
                                );
                            }
                        });
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(6.0);
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(text.header_results()).strong());
                    ui.add_space(6.0);
                    if self.results.is_empty() && !scanning {
                        ui.label(
                            egui::RichText::new(text.label_no_results_prompt())
                                .color(egui::Color32::GRAY),
                        );
                        ui.add_space(6.0);
                    }
                    results_table_ui(ctx, ui, &mut self.results_state, &self.results, &text);
                });
            });
        });

        self.save_config();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(join) = self.watcher_join.take() {
            let _ = join.join();
        }
        self.save_config();
    }
}

fn toolbar_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(toolbar_button_widget(ui, label))
}

fn toolbar_button_widget<'a>(
    ui: &egui::Ui,
    label: impl Into<egui::WidgetText> + 'a,
) -> egui::Button<'a> {
    let size = toolbar_button_size(ui);
    egui::Button::new(label).min_size(size)
}

fn toolbar_button_size(ui: &egui::Ui) -> egui::Vec2 {
    egui::vec2(96.0, ui.spacing().interact_size.y)
}

fn options_menu(ui: &mut egui::Ui, language: &mut Language) -> bool {
    let mut changed = false;
    let style = ui.style();
    let menu_margin = style.spacing.menu_margin.sum().x;
    let stroke = style.visuals.window_stroke.width * 2.0;
    let menu_width = (toolbar_button_size(ui).x - menu_margin - stroke).max(0.0);
    let options_button = toolbar_button_widget(ui, OPTIONS_LABEL);
    let _ = MenuButton::from_button(options_button).ui(ui, |ui| {
        ui.set_min_width(menu_width);
        ui.label(LANGUAGES_LABEL);
        ui.separator();
        for lang in LANGUAGE_ORDER {
            if ui
                .radio_value(language, lang, lang.display_name())
                .changed()
            {
                changed = true;
                ui.close();
            }
        }
    });
    changed
}

fn spawn_process_watcher() -> (
    Receiver<Vec<ProcessInfo>>,
    Arc<AtomicBool>,
    thread::JoinHandle<()>,
) {
    let (tx, rx) = flume::bounded(2);
    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = stop.clone();
    let join = thread::spawn(move || {
        while !stop_clone.load(Ordering::Relaxed) {
            if let Ok(list) = enumerate_processes() {
                let _ = tx.send(list);
            }
            thread::sleep(Duration::from_millis(1500));
        }
    });
    (rx, stop, join)
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
struct AppConfig {
    search_settings: SearchSettings,
    enable_debug_privilege: bool,
    language: Option<Language>,
}

impl AppConfig {
    fn path() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|base| base.join("RSS-strings").join("config.json"))
    }

    fn load() -> Result<Self> {
        let path = Self::path().ok_or_else(|| anyhow::anyhow!("Нет пути для конфига"))?;
        if let Ok(data) = std::fs::read_to_string(&path) {
            let cfg: Self = serde_json::from_str(&data)?;
            return Ok(cfg);
        }
        Ok(Self::default())
    }

    fn save(&self) -> Result<()> {
        let Some(path) = Self::path() else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }
}

impl RssStringsApp {
    fn refresh_processes_now(&mut self) {
        if let Ok(list) = enumerate_processes() {
            self.processes = list;
        }
    }

    fn compose_status_line(&self, text: &UiText) -> String {
        if self.scanner.is_some() {
            return text.status_scanning().to_string();
        }
        let has_queries =
            !parse_input_lines(&self.input_state.value, self.input_state.limit).is_empty();
        if !has_queries {
            return text.status_add_strings().to_string();
        }
        if self.picker.selected_pid.is_none() {
            return text.status_select_process().to_string();
        }
        self.status.to_string(text)
    }
}
