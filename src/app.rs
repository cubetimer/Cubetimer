use chrono::{DateTime, Local, TimeDelta};
use rand::Rng;
use std::collections::HashMap;
mod scramble;

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug)]
pub struct SolveStats {
    time: String,
    scramble: String,
    timestamp: String,
    comment: String,
    plus2: bool,
    dnf: bool,
}

impl Default for SolveStats {
    fn default() -> Self {
        Self {
            time: "".to_string(),
            scramble: "".to_string(),
            timestamp: "".to_string(),
            comment: "".to_string(),
            plus2: false,
            dnf: false,
        }
    }
}

// State

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Debug)]
pub struct State {
    time: String,     // Time as seen in timer
    scramble: String, // Scramble
    ao5: String,      // Averages
    ao12: String,
    ao25: String,
    ao50: String,
    ao100: String,
    ao500: String,
    ao1000: String,
    ao2000: String,
    ao5000: String,
    starttime: DateTime<Local>,
    timeron: bool,
    debounce: DateTime<Local>,
    solves: Vec<SolveStats>,
    settings_open: bool,
    importing: bool,
    imported_data: String,
    imported_fail: String,
    prec: usize,
    ao5_prec: usize,
    used: bool,
    solves_prec: usize,
    old_ao5_prec: usize,
    old_solves_prec: usize,
    old_prec: usize,
    fmt_solves: Vec<String>,
    background: [u8; 3],
    window: [u8; 3],
    button: [u8; 3],
    text: [u8; 3],
    outline: [u8; 3],
    titlebar: [u8; 3],
    outline_w: f32,
    show_solve: bool,
    scramble_text: String,
    solve_info: bool,
    solve_info_copy: String,
    solve_index: usize,
    download: bool,
    show_solve_info: bool,
    show_tools: bool,
    current_tool: String,
    plottable: Vec<[f64; 2]>,
    plot_aspect_ratio: f32,
    name: String,
    widget: [u8; 3],
    scramble_len: i32,
    scramble_len_old: i32,
    stats_open: bool,
    script: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            script: "".to_string(),
            scramble_len: 23,
            scramble_len_old: 23,
            widget: [150, 150, 150],
            show_tools: true,
            plottable: vec![],
            download: true,
            solve_info_copy: "".to_string(),
            solve_info: false,
            scramble_text: "".to_string(),
            show_solve: false,
            time: "0.00".to_string(),
            scramble: "".to_string(),
            ao5: "".to_string(),
            ao12: "".to_string(),
            ao25: "".to_string(),
            ao50: "".to_string(),
            ao100: "".to_string(),
            ao500: "".to_string(),
            ao1000: "".to_string(),
            ao2000: "".to_string(),
            ao5000: "".to_string(),
            starttime: Local::now(),
            timeron: false,
            debounce: Local::now(),
            solves: vec![],
            settings_open: false,
            importing: false,
            imported_data: "".to_string(),
            imported_fail: "".to_string(),
            prec: 2,
            ao5_prec: 3,
            used: false,
            solves_prec: 2,
            old_ao5_prec: 3,
            old_solves_prec: 2,
            old_prec: 2,
            fmt_solves: vec![],
            background: [255, 255, 255],
            window: [255, 255, 255],
            button: [255, 255, 255],
            titlebar: [255, 255, 255],
            outline: [0, 0, 0],
            text: [0, 0, 0],
            outline_w: 0.5,
            solve_index: 0,
            show_solve_info: true,
            current_tool: "Select Tool".to_string(),
            plot_aspect_ratio: 2.0,
            name: "Default".to_string(),
            stats_open: false,
        }
    }
}

fn timestamp() -> String {
    chrono::Local::now().timestamp().to_string()
}

fn round(num: f64, decimals: usize) -> f64 {
    let factor = 10.0_f64.powi(decimals as i32);
    (num * factor).round() / factor
}

fn average(solves: &Vec<SolveStats>, number: usize, prec: usize) -> String {
    let latest: Vec<SolveStats> = solves.get(0..=number).unwrap().to_vec();
    let mut max_val: f64 = latest[0].time.parse::<f64>().unwrap();
    let mut min_val: f64 = latest[0].time.parse::<f64>().unwrap();
    let mut dnfs = 0;

    for solve in &latest {
        if solve.dnf {
            dnfs += 1;
        } else {
            let parsed = solve.time.parse::<f64>().unwrap();
            if parsed > max_val {
                max_val = parsed;
            }
            if parsed < min_val {
                min_val = parsed;
            }
        }
    }

    if dnfs >= 2 {
        return "DNF".to_string();
    }

    let mut sum: f64 = 0.0;
    let mut count = 0;
    for solve in &latest {
        if !solve.dnf {
            let parsed = solve.time.parse::<f64>().unwrap();
            if parsed != max_val && parsed != min_val {
                sum += parsed;
                count += 1;
            }
        }
    }

    if dnfs == 1 {
        sum = sum + max_val;
        count += 1;
    }

    round(sum / count as f64, prec).to_string()
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Cubism {
    state: State,
    sessions: HashMap<String, State>,
    #[serde(skip)]
    set_font: bool,
}

impl Default for Cubism {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
            state: State::default(),
            set_font: false
        }
    }
}

impl Cubism {
    #[cfg(target_arch = "wasm32")]
    fn downloader(&mut self, ctx: &egui::Context) {
        egui::Window::new("Download").show(ctx, |ui| {
            ui.label("Installing the desktop app allows you to use Cubism anywhere!");
            ui.hyperlink_to(
                "Download here",
                "https://github.com/cubetimer/Cubetimer/releases",
            );
            if ui.button("Close").clicked() {
                self.state.download = false;
            }
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn downloader(&mut self, _ctx: &egui::Context) {
        self.state.download = false;
    }

    pub fn reset_time(&mut self) {
        let mut time: String = "0.".to_string();
        for _ in 0..self.state.prec {
            time.push_str("0");
        }
        self.state.time = time;
    }
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
    pub fn calculate_plottable(&mut self) {
        let mut x: f64 = 0.0;
        let mut times: Vec<[f64; 2]> = vec![];
        let mut solves = self.state.solves.clone();
        solves.reverse();
        for solve in solves {
            times.push([x, solve.time.parse().unwrap()]);
            x += 1.0;
        }
        self.state.plottable = times;
    }
    pub fn make_scramble(&self) -> String {
        let options: Vec<String> = vec![
            "R".to_string(),
            "R2".to_string(),
            "R'".to_string(),
            "R".to_string(),
            "U".to_string(),
            "U'".to_string(),
            "U2".to_string(),
            "F".to_string(),
            "F'".to_string(),
            "F2".to_string(),
            "D".to_string(),
            "D'".to_string(),
            "D2".to_string(),
            "L".to_string(),
            "L2".to_string(),
            "L'".to_string(),
        ];
        let mut back = "".to_string();
        let mut scramble: Vec<String> = vec![];
        let mut rng = rand::thread_rng();
        for _i in 1..=self.state.scramble_len {
            loop {
                let option = &options[rng.gen_range(1..options.len())];
                let character = match option.as_str() {
                    "R" => "R",
                    "R'" => "R",
                    "R2" => "R",
                    "U" => "U",
                    "U'" => "U",
                    "U2" => "U",
                    "F" => "F",
                    "F'" => "F",
                    "F2" => "F",
                    "D" => "D",
                    "D'" => "D",
                    "D2" => "D",
                    "L" => "L",
                    "L'" => "L",
                    "L2" => "L",
                    _ => continue,
                };
                if back == character.to_string() {
                    continue;
                }
                scramble.push(option.to_string());
                back = character.to_string();
                break;
            }
        }
        scramble.join(" ").to_string()
    }
    pub fn refresh_averages(&mut self) {
        let len = self.state.solves.len();
        let solves = &self.state.solves;
        if len > 4 {
            self.state.ao5 = average(&solves, 4, self.state.ao5_prec);
        }
        if len > 11 {
            self.state.ao12 = average(&solves, 11, self.state.ao5_prec);
        }
        if len > 24 {
            self.state.ao25 = average(&solves, 24, self.state.ao5_prec);
        }
        if len > 49 {
            self.state.ao50 = average(&solves, 49, self.state.ao5_prec);
        }
        if len > 99 {
            self.state.ao100 = average(&solves, 99, self.state.ao5_prec);
        }
        if len > 499 {
            self.state.ao500 = average(&solves, 499, self.state.ao5_prec);
        }
        if len > 999 {
            self.state.ao1000 = average(&solves, 999, self.state.ao5_prec);
        }
        if len > 1999 {
            self.state.ao2000 = average(&solves, 1999, self.state.ao5_prec);
        }
        if len > 4999 {
            self.state.ao5000 = average(&solves, 4999, self.state.ao5_prec);
        }
    }
    fn redraw_solves(&mut self) {
        self.state.fmt_solves = vec![];
        for solve in &self.state.solves {
            if solve.dnf == true {
                self.state.fmt_solves.push("DNF".to_string());
            } else {
                self.state.fmt_solves.push(
                    round(solve.time.parse::<f64>().unwrap(), self.state.solves_prec).to_string(),
                );
            }
        }
    }
}

impl eframe::App for Cubism {
    fn persist_egui_memory(&self) -> bool {
        true
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.set_font == false {
            let mut definitions = egui::FontDefinitions::default();
            definitions.font_data.insert("font".to_owned(), egui::FontData::from_static(include_bytes!("../assets/font.ttf")));
            definitions.families.get_mut(&egui::FontFamily::Proportional).unwrap()
                .insert(0, "font".to_owned());
            ctx.set_fonts(definitions);
            self.set_font = true;
        }
        ctx.set_visuals(egui::Visuals {
            override_text_color: Some(egui::Color32::from_rgb(
                self.state.text[0],
                self.state.text[1],
                self.state.text[2],
            )),
            panel_fill: egui::Color32::from_rgb(
                self.state.background[0],
                self.state.background[1],
                self.state.background[2],
            ),
            window_fill: egui::Color32::from_rgb(
                self.state.window[0],
                self.state.window[1],
                self.state.window[2],
            ),
            extreme_bg_color: egui::Color32::from_rgb(
                self.state.window[0],
                self.state.window[1],
                self.state.window[2],
            ),
            widgets: egui::style::Widgets {
                inactive: egui::style::WidgetVisuals {
                    weak_bg_fill: egui::Color32::from_rgb(
                        self.state.button[0],
                        self.state.button[1],
                        self.state.button[2],
                    ),
                    bg_stroke: egui::Stroke {
                        width: self.state.outline_w,
                        color: egui::Color32::from_rgb(
                            self.state.outline[0],
                            self.state.outline[1],
                            self.state.outline[2],
                        ),
                    },
                    bg_fill: egui::Color32::from_rgb(
                        self.state.widget[0],
                        self.state.widget[1],
                        self.state.widget[2],
                    ),
                    ..egui::Visuals::light().widgets.inactive
                },
                open: egui::style::WidgetVisuals {
                    weak_bg_fill: egui::Color32::from_rgb(
                        self.state.titlebar[0],
                        self.state.titlebar[1],
                        self.state.titlebar[2],
                    ),
                    ..egui::Visuals::light().widgets.open
                },
                hovered: egui::style::WidgetVisuals {
                    weak_bg_fill: egui::Color32::from_rgb(
                        self.state.button[0],
                        self.state.button[1],
                        self.state.button[2],
                    ),
                    bg_stroke: egui::Stroke {
                        width: self.state.outline_w,
                        color: egui::Color32::from_rgb(
                            self.state.outline[0],
                            self.state.outline[1],
                            self.state.outline[2],
                        ),
                    },
                    ..egui::Visuals::light().widgets.hovered
                },
                ..egui::Visuals::light().widgets
            },
            ..egui::Visuals::light()
        });
        if self.state.timeron == false {
            if self.state.scramble.as_str() == "" {
                self.state.scramble = self.make_scramble();
            }
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| {
                        ui.heading(format!("{}", self.state.scramble));
                    },
                );
            });
        }

        if self.state.timeron == false {
            egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| {
                        ui.label("Cubism V1.0");
                    },
                );
            });
        }

        if self.state.timeron == false {
            egui::SidePanel::left("left_panel").show(ctx, |ui| {
                ui.heading("Sesssions");
                ui.horizontal(|ui| {
                    ui.label("Session");
                    egui::ComboBox::from_label("")
                        .selected_text(self.state.name.clone())
                        .show_ui(ui, |ui| {
                            for (_key, value) in self.sessions.clone() {
                                if value.name == self.state.name {
                                } else {
                                    if ui.selectable_label(false, value.name.clone()).clicked() {
                                        // Transfering state
                                        self.sessions.remove(&self.state.name.clone());
                                        self.sessions.insert(self.state.name.clone(), self.state.clone());
                                        self.state = value.clone();
                                    }
                                }
                            }
                        });
                });

                if ui.button("New Session").clicked() {
                    let new_session_id = rand::thread_rng().gen_range(1..100000).to_string();
                    let new_state = State {
                        name: new_session_id,
                        ..State::default()
                    };
                    if let Some(_data) = self.sessions.get(&self.state.name) {
                        self.sessions.remove(&self.state.name);
                        self.sessions.insert(self.state.name.clone(), self.state.clone());
                    } else {
                        self.sessions.insert(self.state.name.clone(), self.state.clone());
                    }
                    // Making a new session ig
                    self.state = new_state;
                }
                ui.separator();
                ui.heading("View");
                ui.horizontal(|ui| {
                    ui.label("Open Settings");
                    if ui.radio(self.state.settings_open, "").clicked() {
                        if self.state.settings_open == true {
                            self.state.settings_open = false;
                            self.state.importing = false;
                            self.state.imported_fail = "".into();
                            self.state.imported_data = "".into();
                        } else {
                            self.state.settings_open = true;
                        }
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Open Statistics");
                    if ui.radio(self.state.stats_open, "").clicked() {
                        if self.state.stats_open == true {
                            self.state.stats_open = false;
                        } else {
                            self.state.stats_open = true;
                        }
                    }
                });
                ui.separator();
                ui.heading("Solves");
                egui::scroll_area::ScrollArea::vertical().show(ui, |ui| {
                    for i in 0..self.state.solves.len() {
                        let text: String;
                        if self.state.solves[i].dnf == true {
                            text = "Solve: DNF".to_string();
                        } else {
                            text = format!(
                                "Solve: {}",
                                round(
                                    self.state.solves[i].time.parse().unwrap(),
                                    self.state.solves_prec
                                )
                            )
                        }
                        ui.horizontal(|ui| {
                            if ui.button(text).clicked() == true {
                                self.state.solve_info = true;
                                self.state.solve_index = i;
                                if self.state.solves[i].plus2 == true {
                                    self.state.solve_info_copy = format!(
                                        "{}+2 @ {} {}",
                                        round(
                                            self.state.solves[i].time.parse().unwrap(),
                                            self.state.solves_prec
                                        ),
                                        self.state.solves[i].scramble,
                                        self.state.solves[i].comment,
                                    );
                                } else {
                                    if self.state.solves[i].dnf == true {
                                        self.state.solve_info_copy = format!(
                                            "DNF [{}] @ {} {}",
                                            round(
                                                self.state.solves[i].time.parse().unwrap(),
                                                self.state.solves_prec
                                            ),
                                            self.state.solves[i].scramble,
                                            self.state.solves[i].comment
                                        );
                                    } else {
                                        self.state.solve_info_copy = format!(
                                            "{}+2 @ {} {}",
                                            round(
                                                self.state.solves[i].time.parse().unwrap(),
                                                self.state.solves_prec
                                            ),
                                            self.state.solves[i].scramble,
                                            self.state.solves[i].comment,
                                        );
                                    }
                                }
                            }
                            ui.label("    ");
                        });
                    }
                });
            });
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.state.timeron == false {
                if self.state.download == true {
                    self.downloader(ctx);
                }
                if self.state.settings_open == true {
                    egui::Window::new("Settings").show(ctx, |ui| {
                        ui.heading("Help");
                        if ui.button("About Cubism").clicked() == true {
                            self.state.used = false;
                        }
                        ui.separator();
                        ui.heading("Stats");
                        ui.horizontal(|ui| {
                            if ui.button("Reset Session").clicked() == true {
                                self.state = State::default();
                            }
                            if ui.button("Reset App").clicked() == true {
                                self.state = State::default();
                                self.sessions = HashMap::new(); 
                            }
                        });
                        if ui.button("Import from CSTimer").clicked() == true {
                            if self.state.importing == true {
                                self.state.importing = false;
                                self.state.imported_fail = "".to_string();
                                self.state.imported_data = "".to_string();
                            } else {
                                self.state.importing = true;
                            }
                        }
                        ui.horizontal(|ui| {
                            ui.label("Average Precision: ");
                            ui.add(egui::widgets::Slider::new(&mut self.state.ao5_prec, 0..=6));
                            if self.state.ao5_prec != self.state.old_ao5_prec {
                                self.refresh_averages();
                                self.state.old_ao5_prec = self.state.ao5_prec;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Timer Precision: ");
                            ui.add(egui::widgets::Slider::new(&mut self.state.prec, 0..=3));
                            if self.state.prec != self.state.old_prec {
                                if self.state.solves.len() != 0 {
                                    self.state.time = round(self.state.solves[0].time.parse::<f64>().unwrap(), self.state.prec).to_string();
                                }
                                self.state.old_prec = self.state.prec;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Solves Precision: ");
                            ui.add(egui::widgets::Slider::new(&mut self.state.solves_prec, 0..=3));
                            if self.state.solves_prec != self.state.old_solves_prec {
                                self.redraw_solves();
                                self.state.old_solves_prec = self.state.solves_prec;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Show Solve Stats: ");
                            if ui.radio(self.state.show_solve_info, "").clicked() {
                                if self.state.show_solve_info == true {
                                    self.state.show_solve_info = false;
                                } else {
                                    self.state.show_solve_info = true;
                                }
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Show Tools: ");
                            if ui.radio(self.state.show_tools, "").clicked() {
                                if self.state.show_tools == true {
                                    self.state.show_tools = false;
                                } else {
                                    self.state.show_tools = true;
                                }
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Scramble Length: ");
                            ui.add(egui::widgets::Slider::new(&mut self.state.scramble_len, 1i32..=40i32));
                            if self.state.scramble_len_old != self.state.scramble_len {
                                self.state.scramble = self.make_scramble();
                                self.state.scramble_len_old = self.state.scramble_len;
                            }
                        });
                        ui.separator();
                        ui.heading("Style");
                        ui.horizontal(|ui| {
                            ui.label("Background Colour");
                            ui.color_edit_button_srgb(&mut self.state.background);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Window Colour");
                            ui.color_edit_button_srgb(&mut self.state.window);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Button Colour");
                            ui.color_edit_button_srgb(&mut self.state.button);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Text Colour");
                            ui.color_edit_button_srgb(&mut self.state.text);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Button Outline");
                            ui.color_edit_button_srgb(&mut self.state.outline);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Outline Width");
                            ui.add(egui::widgets::Slider::new(&mut self.state.outline_w, 0.0..=1.5));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Title Bar Colour");
                            ui.color_edit_button_srgb(&mut self.state.titlebar);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Widget Colour");
                            ui.color_edit_button_srgb(&mut self.state.widget);
                        });

                    });
                }
                if self.state.importing == true {
                    egui::Window::new("Import from CSTimer").show(ctx, |ui| {
                        ui.label("Open CSTimer, click export, and click export to file. Copy the contents of that file and paste it here.");
                        ui.text_edit_singleline(&mut self.state.imported_data);
                        ui.horizontal(|ui| {
                            if ui.button("Import").clicked() == true {
                                let data: serde_json::Value;
                                let result = serde_json::from_str(&self.state.imported_data);
                                if let Err(e) = result {
                                    println!("{}", e);
                                    self.state.imported_fail = "Invalid JSON!".to_string();
                                } else {
                                    data = result.unwrap();
                                    if let Some(session) = data["session1"].as_array() {
                                        self.state.imported_fail = "Imported data!".to_string();
                                        for value in session {
                                            let mut total: f64 = 0.0;
                                            for value in value[0].as_array().unwrap() {
                                                total += value.as_f64().unwrap() / 1000.0;
                                            }
                                            let solve = SolveStats {
                                                time: total.to_string(),
                                                timestamp: timestamp(),
                                                scramble: value[1].as_str().unwrap().to_string(),
                                                comment: value[2].as_str().unwrap().to_string(),
                                                ..SolveStats::default()
                                            };
                                            self.state.solves.insert(0, solve);
                                        }
                                        self.redraw_solves();
                                        self.refresh_averages();
                                        self.state.scramble_text = format!("{} @ {}", round(self.state.solves[0].time.parse().unwrap(), self.state.solves_prec), self.state.solves[0].scramble);
                                        self.calculate_plottable();
                                    } else {
                                        self.state.imported_fail = "Failed to import data!".to_string();
                                    }
                                }
                            }
                            if ui.button("Close").clicked() == true{
                                self.state.importing = false;
                                self.state.imported_fail = "".to_string();
                                self.state.imported_data = "".to_string();
                            }
                        });
                        ui.label(format!("{}", self.state.imported_fail));
                    });
                }
            }
            if self.state.timeron == false {
                if self.state.used == false {
                    egui::Window::new("About Cubism").show(ctx, |ui| {
                        ui.label("Cubism is like CSTimer, but the way you start the timer is different. Press the space bar to start and stop, no need to hold. You cannot record times under 0.25 seconds!");
                        ui.hyperlink_to(
                            "Click here for logo credits.",
                            "https://www.flaticon.com/free-icons/3d-cube",
                        );
                        if ui.button("I Understand").clicked() == true {
                            self.state.used = true;
                        }
                    });
                }
                if self.state.show_solve == true {
                    if self.state.show_solve_info == true {
                        egui::Window::new("Solve Stats").show(ctx, |ui| {
                            let displayed: String;
                            if self.state.solves[0].dnf == true {
                                displayed = "DNF".to_string();
                            } else {
                                displayed = round(self.state.solves[0].time.parse().unwrap(), self.state.solves_prec).to_string();
                            }
                            ui.heading(format!("{}", displayed));
                            ui.label(format!("{}", self.state.solves[0].scramble));
                            ui.horizontal(|ui| {
                                if ui.button("+2").clicked() {
                                    if !self.state.solves[0].plus2 {
                                        self.state.solves[0].plus2 = true;
                                        self.state.solves[0].dnf = false;
                                        self.state.solves[0].time = (self.state.solves[0].time.parse::<f64>().unwrap() + 2.0).to_string();
                                        self.state.scramble_text = format!("{}+2 @ {}", round(self.state.solves[0].clone().time.parse::<f64>().unwrap() - 2.0, self.state.solves_prec), self.state.solves[0].scramble);
                                        self.refresh_averages();
                                    }
                                }
                                if ui.button("DNF").clicked() {
                                    if !self.state.solves[0].dnf {
                                        if self.state.solves[0].plus2 {
                                            self.state.solves[0].time = (self.state.solves[0].time.parse::<f64>().unwrap() - 2.0).to_string();
                                        }
                                        self.state.solves[0].plus2 = false;
                                        self.state.solves[0].dnf = true;
                                        self.state.scramble_text = format!("DNF [{}] @ {}", round(self.state.solves[0].time.parse().unwrap(), self.state.solves_prec), self.state.solves[0].scramble);
                                        self.refresh_averages();
                                    }
                                }
                                if ui.button("OK").clicked() {
                                    if self.state.solves[0].plus2 {
                                        self.state.solves[0].time = (self.state.solves[0].time.parse::<f64>().unwrap() - 2.0).to_string();
                                    }
                                    self.state.solves[0].plus2 = false;
                                    self.state.solves[0].dnf = false;
                                    self.state.scramble_text = format!("{} @ {}", round(self.state.solves[0].time.parse().unwrap(), self.state.solves_prec), self.state.solves[0].scramble);
                                    self.refresh_averages();
                                }
                                if ui.button("DEL").clicked() {
                                    self.state.solves.remove(0);
                                    self.state.time = round(self.state.solves[0].time.parse().unwrap(), self.state.prec).to_string();
                                    self.state.show_solve = false;
                                    self.refresh_averages();
                                    self.redraw_solves();
                                }
                            });
                            // Time
                            ui.horizontal(|ui| {
                                ui.label("Copyable:  ");
                                ui.text_edit_singleline(&mut self.state.scramble_text.as_str());
                            });
                        });
                    }
                }
                if self.state.solve_info == true {
                    egui::Window::new("Solve Info").show(ctx, |ui| {
                        ui.heading(format!("{}", round(self.state.solves[self.state.solve_index].time.parse().unwrap(), self.state.solves_prec)));
                        if self.state.solves[self.state.solve_index].dnf == true {
                            ui.label("Did Not Finish");
                        }
                        if self.state.solves[self.state.solve_index].plus2 == true {
                            ui.label("+2 Penalty");
                        }
                        for gap in [4, 11, 24, 49, 99, 499, 999, 1999, 4999] {
                            let portion = self.state.solves.get(self.state.solve_index..self.state.solves.len());
                            match portion {
                                Some(data) => {
                                    let portion = data.to_vec();
                                    if portion.len() > gap {
                                        let average = average(&portion, gap, self.state.ao5_prec);
                                        ui.label(format!("Ao{}: {}", gap+1, average));
                                    }
                                },
                                None => {},
                            }
                        }
                        ui.horizontal(|ui| {
                            if ui.button("+2").clicked() {
                                if !self.state.solves[self.state.solve_index].plus2 {
                                    self.state.solves[self.state.solve_index].plus2 = true;
                                    self.state.solves[self.state.solve_index].dnf = false;
                                    self.state.solves[self.state.solve_index].time = (self.state.solves[self.state.solve_index].time.parse::<f64>().unwrap() + 2.0).to_string();
                                    self.state.solve_info_copy = format!("{}+2 @ {}", round(self.state.solves[self.state.solve_index].clone().time.parse::<f64>().unwrap() - 2.0, self.state.solves_prec), self.state.solves[self.state.solve_index].scramble);
                                    self.refresh_averages();
                                }
                            }
                            if ui.button("DNF").clicked() {
                                if !self.state.solves[self.state.solve_index].dnf {
                                    if self.state.solves[self.state.solve_index].plus2 {
                                        self.state.solves[self.state.solve_index].time = (self.state.solves[self.state.solve_index].time.parse::<f64>().unwrap() - 2.0).to_string();
                                    }
                                    self.state.solves[self.state.solve_index].plus2 = false;
                                    self.state.solves[self.state.solve_index].dnf = true;
                                    self.state.solve_info_copy = format!("DNF [{}] @ {}", round(self.state.solves[self.state.solve_index].time.parse().unwrap(), self.state.solves_prec), self.state.solves[self.state.solve_index].scramble);
                                    self.refresh_averages();
                                }
                            }
                            if ui.button("OK").clicked() {
                                if self.state.solves[self.state.solve_index].plus2 {
                                    self.state.solves[self.state.solve_index].time = (self.state.solves[self.state.solve_index].time.parse::<f64>().unwrap() - 2.0).to_string();
                                }
                                self.state.solves[self.state.solve_index].plus2 = false;
                                self.state.solves[self.state.solve_index].dnf = false;
                                self.state.solve_info_copy = format!("{} @ {}", round(self.state.solves[self.state.solve_index].time.parse().unwrap(), self.state.solves_prec), self.state.solves[self.state.solve_index].scramble);
                                self.refresh_averages();
                            }
                            if ui.button("DEL").clicked() {
                                self.state.solves.remove(self.state.solve_index);
                                self.state.time = round(self.state.solves[self.state.solve_index].time.parse().unwrap(), self.state.prec).to_string();
                                self.state.solve_info = false;
                                self.refresh_averages();
                                self.redraw_solves();
                            }
                        });
                        ui.label(format!("Scramble: {}", self.state.solves[self.state.solve_index].scramble));
                        ui.horizontal(|ui| {
                            ui.label("Comment: ");
                            ui.text_edit_singleline(&mut self.state.solves[self.state.solve_index].comment);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Copyable:  ");
                            ui.text_edit_singleline(&mut self.state.solve_info_copy.as_str());
                        });
                        if ui.button("Close").clicked() == true {
                            self.state.solve_info_copy = "".to_string();
                            self.state.solve_info = false;
                        }
                    });
                }
                if self.state.show_tools == true {
                    egui::Window::new("Tools").show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            egui::containers::ComboBox::from_label("")
                                .selected_text(&self.state.current_tool)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.state.current_tool, "Plot Times".to_string(), "Plot Times");
                                    ui.selectable_value(&mut self.state.current_tool, "Script".to_string(), "Script")
                                });
                        });
                        if self.state.current_tool == "Plot Times".to_string() {
                            ui.separator();
                            ui.heading("Plot Times");
                            let line = egui_plot::Line::new(self.state.plottable.clone());
                            egui_plot::Plot::new("Time Distribution").view_aspect(self.state.plot_aspect_ratio).show(ui, |plot_ui| plot_ui.line(line));
                            ui.horizontal(|ui| {
                                ui.label("Width: ");
                                ui.add(egui::Slider::new(&mut self.state.plot_aspect_ratio, 1.0..=6.0));
                            });
                        }
                        if self.state.current_tool == "Script".to_string() {
                            ui.separator();
                            ui.heading("Script");
                            ui.label("You can access solves by accessing the DATA dictionary");
                            ui.text_edit_multiline(&mut self.state.script);
                            if ui.button("Run").clicked() {
                            }
                        }
                    });
                }
                if self.state.stats_open == true {
                    egui::Window::new("Statistics").show(ctx, |ui| {
                    ui.label(format!("Solves: {}", self.state.solves.len()));
                    if self.state.ao5.as_str() != "" {
                        ui.label(format!("Ao5: {}", self.state.ao5));
                    }
                    if self.state.ao12.as_str() != "" {
                        ui.label(format!("Ao12: {}", self.state.ao12));
                    }
                    if self.state.ao25.as_str() != "" {
                        ui.label(format!("Ao25: {}", self.state.ao25));
                    }
                    if self.state.ao50.as_str() != "" {
                        ui.label(format!("Ao50: {}", self.state.ao50));
                    }
                    if self.state.ao100.as_str() != "" {
                        ui.label(format!("Ao100: {}", self.state.ao100));
                    }
                    if self.state.ao500.as_str() != "" {
                        ui.label(format!("Ao500: {}", self.state.ao500));
                    }
                    if self.state.ao1000.as_str() != "" {
                        ui.label(format!("Ao1000: {}", self.state.ao1000));
                    }
                    if self.state.ao2000.as_str() != "" {
                        ui.label(format!("Ao2000: {}", self.state.ao2000));
                    }
                    if self.state.ao5000.as_str() != "" {
                        ui.label(format!("Ao5000: {}", self.state.ao2000));
                    }
                    });
                }
            }
            if self.state.timeron == true {
                ctx.request_repaint();
            }
            ctx.input(|i| {
                for event in i.clone().events {
                    match event {
                        egui::Event::Key { key, .. } => {
                            let delta = Local::now().signed_duration_since(self.state.debounce);
                            if delta > TimeDelta::try_milliseconds(250).unwrap() {
                                self.state.debounce = Local::now();
                                if self.state.importing == false && self.state.solve_info == false {
                                    if key == egui::Key::Space {
                                        if self.state.timeron == false {
                                            self.state.timeron = true;
                                            self.state.starttime = Local::now();
                                        } else {
                                            self.state.timeron = false;
                                            let rawtime: f64 = Local::now().signed_duration_since(self.state.starttime).num_microseconds().unwrap() as f64 / 1000000 as f64;
                                            let timertime = round(rawtime,
                                                self.state.prec
                                            );
                                            let solvetime = round(rawtime,
                                                self.state.solves_prec
                                            );

                                            let solve = SolveStats {
                                                time: rawtime.to_string(),
                                                scramble: self.state.scramble.clone(),
                                                timestamp: timestamp(),
                                                ..SolveStats::default()
                                            };

                                            self.state.solves.insert(0, solve);

                                            self.state.time = timertime.to_string();
                                            self.state.fmt_solves.insert(0, solvetime.to_string());
                                            self.refresh_averages();
                                            self.state.scramble = self.make_scramble();
                                            self.state.scramble_text = format!("{} @ {}", round(self.state.solves[0].time.parse().unwrap(), self.state.prec), self.state.solves[0].scramble);
                                            self.state.show_solve = true;
                                            self.calculate_plottable();
                                        }
                                    } else if key == egui::Key::Escape {
                                        self.reset_time();
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            });
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    ui.style_mut().text_styles.get_mut(&egui::TextStyle::Heading).unwrap().size = 64.0;
                    if self.state.timeron == true {
                        let secs = {
                            let duration: f64 = Local::now().signed_duration_since(self.state.starttime).num_microseconds().unwrap() as f64 / 1000000 as f64;

                            let duration = round(duration, self.state.prec);
                            let duration = duration.to_string();
                            let parts: Vec<&str> = duration.split(".").collect();
                            if parts.len() == 1 {
                                if self.state.prec > 0 {
                                    if parts[0] == "0" {
                                        "0".to_string()
                                    } else {
                                        let mut zeros = "".to_string();
                                        for _ in 0..self.state.prec {
                                            zeros.push_str("0");
                                        }
                                        format!("{}.{}", parts[0], zeros)
                                    }
                                } else {
                                duration.to_string()
                                }
                            } else {
                                let main = parts[0].to_string();
                                let mut dec = parts[1].to_string();
                                if dec.len() < self.state.prec {
                                    loop {
                                        if dec.len() != self.state.prec {
                                            dec.push_str("0");
                                        } else {
                                            break;
                                        }
                                    }
                                }
                                format!("{}.{}", main, dec)
                            }
                        };

                        ui.heading(format!(
                            "{}", secs
                        ));
                    } else {
                        ui.heading(format!("{}", self.state.time));
                    }
                    ui.style_mut().text_styles.get_mut(&egui::TextStyle::Heading).unwrap().size = 30.0;
                },
            )
        });
    }
}
