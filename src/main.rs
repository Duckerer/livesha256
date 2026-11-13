// LiveSHA256 by Duckerer
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod gpu;
mod hashing;
mod search;

use eframe::egui::{self, Color32, RichText};
use hashing::*;
use search::{Method, Search};
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::sync::{mpsc, Arc};
use std::time::Duration;
use std::{fs, thread};

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Live,
    Create,
    Check,
}
#[derive(PartialEq, Clone, Copy)]
enum Theme {
    Dark,
    Light,
    Gray,
}
#[derive(PartialEq, Clone, Copy)]
enum TabPos {
    Top,
    Bottom,
    Left,
}

struct CheckJob {
    rx: mpsc::Receiver<Result<String, String>>,
    prog: Arc<AtomicU64>,
    total: u64,
}

struct App {
    tab: Tab,
    theme: Theme,
    pos: TabPos,
    algo: Algo,
    settings_open: bool,
    // live
    live_text: String,
    live_hash: String,
    live_algo: Algo,
    // create
    base: String,
    target: String,
    result: Option<String>,
    status: String,
    search: Option<Search>,
    method: Method,
    threads: usize,
    max_threads: usize,
    // check
    check_path: Option<PathBuf>,
    check_hash: String,
    check_algo: Algo,
    check_job: Option<CheckJob>,
    check_total: u64,
}

fn save_as(data: &[u8]) {
    if let Some(p) = rfd::FileDialog::new().save_file() {
        let _ = fs::write(p, data);
    }
}

fn readonly(ui: &mut egui::Ui, s: &str) {
    ui.add(egui::TextEdit::singleline(&mut s.to_owned().as_str()).font(egui::TextStyle::Monospace).desired_width(f32::INFINITY));
}

fn hash_line(ui: &mut egui::Ui, label: &str, hex: &str, matched: usize) {
    let m = matched.min(hex.len());
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label(label);
        ui.label(RichText::new(&hex[..m]).monospace().color(Color32::from_rgb(80, 220, 100)));
        ui.label(RichText::new(&hex[m..]).monospace());
    });
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let get = |k: &str| cc.storage.and_then(|s| s.get_string(k));
        let theme = match get("theme").as_deref() {
            Some("light") => Theme::Light,
            Some("gray") => Theme::Gray,
            _ => Theme::Dark,
        };
        let pos = match get("pos").as_deref() {
            Some("bottom") => TabPos::Bottom,
            Some("left") => TabPos::Left,
            _ => TabPos::Top,
        };
        let algo = get("algo").and_then(|k| Algo::from_key(&k)).unwrap_or(Algo::Sha256);
        let method = match get("method").as_deref() {
            Some("gpu") => Method::Gpu,
            Some("hybrid") => Method::Hybrid,
            Some("generic") => Method::CpuGeneric,
            _ => Method::CpuFast,
        };
        let max_threads = thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
        Self {
            tab: Tab::Live,
            theme,
            pos,
            algo,
            settings_open: false,
            live_text: String::new(),
            live_hash: hash_hex(algo, b""),
            live_algo: algo,
            base: String::new(),
            target: String::new(),
            result: None,
            status: String::new(),
            search: None,
            method,
            threads: max_threads,
            max_threads,
            check_path: None,
            check_hash: String::new(),
            check_algo: algo,
            check_job: None,
            check_total: 0,
        }
    }

    fn running(&self) -> bool {
        self.search.as_ref().map_or(false, |s| !s.done)
    }

    fn start(&mut self) {
        match Search::spawn(self.algo, self.base.clone(), &self.target, self.threads, self.method) {
            Ok(s) => {
                self.status = s.note.clone();
                self.result = None;
                self.search = Some(s);
            }
            Err(e) => self.status = e,
        }
    }

    fn stop(&mut self) {
        if let Some(s) = self.search.as_mut() {
            s.finish();
        }
        self.status = "Остановлено".into();
    }

    fn load_check(&mut self, p: PathBuf) {
        self.check_path = Some(p);
        self.start_check();
    }

    fn start_check(&mut self) {
        let Some(p) = self.check_path.clone() else { return };
        let algo = self.algo;
        self.check_algo = algo;
        self.check_hash.clear();
        let total = fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
        self.check_total = total;
        let prog = Arc::new(AtomicU64::new(0));
        let pr = prog.clone();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let r = (|| -> std::io::Result<String> {
                let mut f = fs::File::open(&p)?;
                let mut hs = Hs::new(algo);
                let mut buf = vec![0u8; 1 << 20];
                loop {
                    let n = f.read(&mut buf)?;
                    if n == 0 {
                        break;
                    }
                    hs.update(&buf[..n]);
                    pr.fetch_add(n as u64, Relaxed);
                }
                Ok(hex(&hs.finish()))
            })();
            let _ = tx.send(r.map_err(|e| e.to_string()));
        });
        self.check_job = Some(CheckJob { rx, prog, total });
    }

    fn tabs_ui(&mut self, ui: &mut egui::Ui) {
        ui.selectable_value(&mut self.tab, Tab::Live, "Live");
        ui.selectable_value(&mut self.tab, Tab::Create, "Create");
        ui.selectable_value(&mut self.tab, Tab::Check, "Check");
        egui::ComboBox::from_id_salt("algo").selected_text(self.algo.name()).show_ui(ui, |ui| {
            for a in ALGOS {
                ui.selectable_value(&mut self.algo, a, a.name());
            }
        });
        if ui.button("⚙").clicked() {
            self.settings_open = true;
        }
    }

    fn create_info(&self, ui: &mut egui::Ui) {
        let Some(s) = &self.search else { return };
        let tries = s.ctl.tries.load(Relaxed);
        let speed = if s.done { tries as f64 / s.elapsed.max(0.001) } else { s.speed };
        let mut line = format!("Попыток: {}   Скорость: {}/с   Прошло: {}", fmt_big(tries as f64), fmt_big(speed), fmt_dur(s.elapsed));
        if !s.dev.is_empty() {
            line += &format!("   [{}]", s.dev);
        }
        ui.label(line);
        if !s.done && speed > 0.0 {
            let exp = 16f64.powi(s.tlen as i32);
            ui.label(format!("В среднем нужно ≈ {} попыток, это ≈ {}", fmt_big(exp), fmt_dur(exp / speed)));
        }
        if let Some(e) = s.ctl.err.lock().unwrap().as_ref() {
            ui.colored_label(Color32::LIGHT_RED, e);
        }
        let cur = s.ctl.cur.load(Relaxed);
        if cur != u64::MAX && !s.done {
            let h = s.hash_for(cur);
            let m = common(&h, &s.target);
            hash_line(ui, &format!("попытка #{}: ", cur), &h, m);
        }
        let (bl, bn) = *s.ctl.best.lock().unwrap();
        if bl > 0 {
            let h = s.hash_for(bn);
            let m = common(&h, &s.target);
            hash_line(ui, &format!("лучшая #{} ({} симв.): ", bn, m), &h, m);
        }
    }
}

impl eframe::App for App {
    fn save(&mut self, s: &mut dyn eframe::Storage) {
        s.set_string("theme", match self.theme { Theme::Dark => "dark", Theme::Light => "light", Theme::Gray => "gray" }.into());
        s.set_string("pos", match self.pos { TabPos::Top => "top", TabPos::Bottom => "bottom", TabPos::Left => "left" }.into());
        s.set_string("algo", self.algo.key().into());
        s.set_string("method", match self.method { Method::Gpu => "gpu", Method::Hybrid => "hybrid", Method::CpuFast => "fast", Method::CpuGeneric => "generic" }.into());
    }

    fn update(&mut self, ctx: &egui::Context, _f: &mut eframe::Frame) {
        ctx.set_visuals(match self.theme {
            Theme::Dark => egui::Visuals::dark(),
            Theme::Light => egui::Visuals::light(),
            Theme::Gray => {
                let mut v = egui::Visuals::dark();
                v.panel_fill = Color32::from_gray(72);
                v.window_fill = Color32::from_gray(64);
                v.extreme_bg_color = Color32::from_gray(52);
                v.faint_bg_color = Color32::from_gray(80);
                v
            }
        });

        // drag & drop: любой файл -> вкладка Check
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if let Some(p) = dropped.into_iter().find_map(|f| f.path) {
            self.tab = Tab::Check;
            self.load_check(p);
        }
        if ctx.input(|i| !i.raw.hovered_files.is_empty()) {
            let p = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("drop")));
            let r = ctx.screen_rect();
            p.rect_filled(r, 0.0, Color32::from_black_alpha(180));
            p.text(r.center(), egui::Align2::CENTER_CENTER, "Бросай файл сюда", egui::FontId::proportional(28.0), Color32::WHITE);
        }

        // смена алгоритма -> пересчёт
        if self.live_algo != self.algo {
            self.live_hash = hash_hex(self.algo, self.live_text.as_bytes());
            self.live_algo = self.algo;
        }
        if self.check_algo != self.algo && self.check_path.is_some() && self.check_job.is_none() {
            self.start_check();
        }

        // фоновые задачи
        let mut found = None;
        if let Some(s) = self.search.as_mut() {
            s.tick();
            if !s.done {
                if let Ok(r) = s.rx.try_recv() {
                    found = Some(r);
                } else if s.ctl.stop.load(Relaxed) {
                    s.finish(); // GPU/воркер остановился с ошибкой
                }
            }
        }
        if let Some(r) = found {
            self.result = Some(r);
            self.status = "Нашёл!".into();
            if let Some(s) = self.search.as_mut() {
                s.finish();
            }
        }
        let mut chk = None;
        if let Some(j) = &self.check_job {
            chk = j.rx.try_recv().ok();
        }
        if let Some(r) = chk {
            self.check_hash = r.unwrap_or_else(|e| format!("Ошибка: {}", e));
            self.check_job = None;
        }
        if self.running() || self.check_job.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }

        match self.pos {
            TabPos::Top => {
                egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
                    ui.horizontal(|ui| self.tabs_ui(ui));
                });
            }
            TabPos::Bottom => {
                egui::TopBottomPanel::bottom("tabs").show(ctx, |ui| {
                    ui.horizontal(|ui| self.tabs_ui(ui));
                });
            }
            TabPos::Left => {
                egui::SidePanel::left("tabs").resizable(false).show(ctx, |ui| {
                    ui.vertical(|ui| self.tabs_ui(ui));
                });
            }
        }

        let mut open = self.settings_open;
        egui::Window::new("Settings").open(&mut open).collapsible(false).show(ctx, |ui| {
            ui.label("Тема");
            ui.radio_value(&mut self.theme, Theme::Dark, "Чёрная");
            ui.radio_value(&mut self.theme, Theme::Gray, "Серая");
            ui.radio_value(&mut self.theme, Theme::Light, "Белая");
            ui.separator();
            ui.label("Расположение вкладок");
            ui.radio_value(&mut self.pos, TabPos::Top, "Сверху");
            ui.radio_value(&mut self.pos, TabPos::Bottom, "Снизу");
            ui.radio_value(&mut self.pos, TabPos::Left, "Слева");
        });
        self.settings_open = open;

        match self.tab {
            Tab::Live => {
                egui::TopBottomPanel::bottom("live_bottom").show(ctx, |ui| {
                    ui.label(self.algo.name());
                    readonly(ui, &self.live_hash);
                    if ui.button("Save as").clicked() {
                        save_as(self.live_text.as_bytes());
                    }
                });
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let r = ui.add_sized(ui.available_size(), egui::TextEdit::multiline(&mut self.live_text).font(egui::TextStyle::Monospace).lock_focus(true));
                        if r.changed() {
                            self.live_hash = hash_hex(self.algo, self.live_text.as_bytes());
                        }
                    });
                });
            }
            Tab::Create => {
                egui::TopBottomPanel::bottom("create_bottom").show(ctx, |ui| {
                    self.create_info(ui);
                    if let Some(r) = &self.result {
                        hash_line(ui, "Итог: ", &hash_hex(self.algo, r.as_bytes()), self.target.trim().len());
                    }
                    let running = self.running();
                    ui.horizontal_wrapped(|ui| {
                        if ui.button(if running { "Stop" } else { "Start" }).clicked() {
                            if running { self.stop() } else { self.start() }
                        }
                        ui.add_enabled_ui(!running, |ui| {
                            egui::ComboBox::from_id_salt("method")
                                .selected_text(match self.method {
                                    Method::Gpu => "GPU (SHA-256)",
                                    Method::Hybrid => "GPU + CPU (SHA-256)",
                                    Method::CpuFast => "CPU быстрый (SHA-256)",
                                    Method::CpuGeneric => "CPU универсальный",
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.method, Method::Gpu, "GPU (SHA-256)");
                                    ui.selectable_value(&mut self.method, Method::Hybrid, "GPU + CPU гибрид (SHA-256)");
                                    ui.selectable_value(&mut self.method, Method::CpuFast, "CPU быстрый (SHA-256)");
                                    ui.selectable_value(&mut self.method, Method::CpuGeneric, "CPU универсальный (все алгоритмы)");
                                });
                            if self.method != Method::Gpu {
                                ui.add(egui::Slider::new(&mut self.threads, 1..=self.max_threads).text("потоков"));
                            }
                        });
                    });
                    if !self.status.is_empty() {
                        ui.label(&self.status);
                    }
                    ui.add(egui::TextEdit::singleline(&mut self.target).hint_text(format!("{} (или его начало, hex)", self.algo.name())).font(egui::TextStyle::Monospace).desired_width(f32::INFINITY));
                    ui.add_enabled_ui(self.result.is_some(), |ui| {
                        if ui.button("Save as").clicked() {
                            if let Some(r) = &self.result {
                                save_as(r.as_bytes());
                            }
                        }
                    });
                });
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label("Начальный текст (в конец допишется \\n<число>). Результат появится здесь:");
                    egui::ScrollArea::vertical().id_salt("c").show(ui, |ui| {
                        if let Some(r) = &self.result {
                            ui.add(egui::TextEdit::multiline(&mut r.as_str()).font(egui::TextStyle::Monospace).desired_width(f32::INFINITY));
                        } else {
                            ui.add_sized(ui.available_size(), egui::TextEdit::multiline(&mut self.base).font(egui::TextStyle::Monospace));
                        }
                    });
                });
            }
            Tab::Check => {
                egui::TopBottomPanel::bottom("check_bottom").show(ctx, |ui| {
                    ui.label(self.check_algo.name());
                    readonly(ui, &self.check_hash);
                    if ui.button("Load").clicked() {
                        if let Some(p) = rfd::FileDialog::new().pick_file() {
                            self.load_check(p);
                        }
                    }
                });
                egui::CentralPanel::default().show(ctx, |ui| match &self.check_path {
                    None => {
                        ui.label("Перетащи файл в окно или нажми Load");
                    }
                    Some(p) => {
                        ui.label(format!("Файл: {}", p.display()));
                        ui.label(format!("Размер: {} байт", self.check_total));
                        if let Some(j) = &self.check_job {
                            let done = j.prog.load(Relaxed) as f32;
                            ui.add(egui::ProgressBar::new(if j.total > 0 { done / j.total as f32 } else { 0.0 }).show_percentage());
                        }
                    }
                });
            }
        }
    }
}

/// Консольный режим: livesha256 --find <algo> <hex> <text> [gpu|hybrid|fast|generic] [threads]
fn cli(a: &[String]) {
    let algo = a.get(2).and_then(|k| Algo::from_key(k)).unwrap_or(Algo::Sha256);
    let target = a.get(3).cloned().unwrap_or_default();
    let text = a.get(4).cloned().unwrap_or_default();
    let method = match a.get(5).map(String::as_str) {
        Some("gpu") => Method::Gpu,
        Some("hybrid") => Method::Hybrid,
        Some("generic") => Method::CpuGeneric,
        _ => Method::CpuFast,
    };
    let threads = a.get(6).and_then(|t| t.parse().ok()).unwrap_or_else(|| thread::available_parallelism().map(|n| n.get()).unwrap_or(4));
    let mut s = match Search::spawn(algo, text, &target, threads, method) {
        Ok(s) => s,
        Err(e) => return eprintln!("{}", e),
    };
    if !s.note.is_empty() {
        eprintln!("{}", s.note);
    }
    eprintln!("метод: {:?} {}", s.used, s.dev);
    loop {
        thread::sleep(Duration::from_millis(250));
        s.tick();
        if let Ok(r) = s.rx.try_recv() {
            s.finish();
            let tries = s.ctl.tries.load(Relaxed);
            eprintln!("нашёл за {:.2} с, попыток ~{}, {}/с", s.elapsed, fmt_big(tries as f64), fmt_big(s.speed));
            println!("{}", r.escape_debug());
            println!("{}", hash_hex(algo, r.as_bytes()));
            return;
        }
        if s.ctl.stop.load(Relaxed) {
            eprintln!("ошибка: {:?}", s.ctl.err.lock().unwrap());
            return;
        }
    }
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--find") {
        cli(&args);
        return Ok(());
    }
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([760.0, 520.0]).with_title("LiveSHA256").with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native("LiveSHA256", opts, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
