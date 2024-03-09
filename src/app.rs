use chrono::{DateTime, Local, TimeDelta};
use rand::Rng;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
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
    time: String,
    scramble: String,
    ao5: String,
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
}

impl Default for Cubism {
    fn default() -> Self {
        Self {
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
        }
    }
}

impl Cubism {
    pub fn reset_time(&mut self) {
        let mut time: String = "0.".to_string();
        for _ in 0..self.prec {
            time.push_str("0");
        }
        self.time = time;
    }
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
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
        for _i in 1..23 {
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
        let len = self.solves.len();
        let solves = &self.solves;
        if len > 4 {
            self.ao5 = average(&solves, 4, self.ao5_prec);
        }
        if len > 11 {
            self.ao12 = average(&solves, 11, self.ao5_prec);
        }
        if len > 24 {
            self.ao25 = average(&solves, 24, self.ao5_prec);
        }
        if len > 49 {
            self.ao50 = average(&solves, 49, self.ao5_prec);
        }
        if len > 99 {
            self.ao100 = average(&solves, 99, self.ao5_prec);
        }
        if len > 499 {
            self.ao500 = average(&solves, 499, self.ao5_prec);
        }
        if len > 999 {
            self.ao1000 = average(&solves, 999, self.ao5_prec);
        }
        if len > 1999 {
            self.ao2000 = average(&solves, 1999, self.ao5_prec);
        }
        if len > 4999 {
            self.ao5000 = average(&solves, 4999, self.ao5_prec);
        }
    }
    fn redraw_solves(&mut self) {
        self.fmt_solves = vec![];
        for solve in &self.solves {
            if solve.dnf == true {
                self.fmt_solves.push("DNF".to_string());
            } else {
            self.fmt_solves
                .push(round(solve.time.parse::<f64>().unwrap(), self.solves_prec).to_string());
            }
        }
    }
}

impl eframe::App for Cubism {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals {
            override_text_color: Some(egui::Color32::from_rgb(
                self.text[0],
                self.text[1],
                self.text[2],
            )),
            panel_fill: egui::Color32::from_rgb(
                self.background[0],
                self.background[1],
                self.background[2],
            ),
            window_fill: egui::Color32::from_rgb(self.window[0], self.window[1], self.window[2]),
            extreme_bg_color: egui::Color32::from_rgb(
                self.window[0],
                self.window[1],
                self.window[2],
            ),
            widgets: egui::style::Widgets {
                inactive: egui::style::WidgetVisuals {
                    weak_bg_fill: egui::Color32::from_rgb(
                        self.button[0],
                        self.button[1],
                        self.button[2],
                    ),
                    bg_stroke: egui::Stroke {
                        width: self.outline_w,
                        color: egui::Color32::from_rgb(
                            self.outline[0],
                            self.outline[1],
                            self.outline[2],
                        ),
                    },
                    bg_fill: egui::Color32::from_rgb(self.text[0], self.text[1], self.text[2]),
                    ..egui::Visuals::light().widgets.inactive
                },
                open: egui::style::WidgetVisuals {
                    weak_bg_fill: egui::Color32::from_rgb(
                        self.titlebar[0],
                        self.titlebar[1],
                        self.titlebar[2],
                    ),
                    ..egui::Visuals::light().widgets.open
                },
                hovered: egui::style::WidgetVisuals {
                    weak_bg_fill: egui::Color32::from_rgb(
                        self.button[0],
                        self.button[1],
                        self.button[2],
                    ),
                    bg_stroke: egui::Stroke {
                        width: self.outline_w,
                        color: egui::Color32::from_rgb(
                            self.outline[0],
                            self.outline[1],
                            self.outline[2],
                        ),
                    },
                    ..egui::Visuals::light().widgets.hovered
                },
                ..egui::Visuals::light().widgets
            },
            ..egui::Visuals::light()
        });
        if self.timeron == false {
            if self.scramble.as_str() == "" {
                self.scramble = self.make_scramble();
            }
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| {
                        ui.heading(format!("{}", self.scramble));
                    },
                );
            });
        }

        if self.timeron == false {
            egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| {
                        ui.label("Cubism V1.0");
                    },
                );
            });
        }

        if self.timeron == false {
            egui::SidePanel::left("left_panel").show(ctx, |ui| {
                ui.heading("Statistics");
                ui.label(format!("Solves: {}", self.solves.len()));
                if self.ao5.as_str() != "" {
                    ui.label(format!("Ao5: {}", self.ao5));
                }
                if self.ao12.as_str() != "" {
                    ui.label(format!("Ao12: {}", self.ao12));
                }
                if self.ao25.as_str() != "" {
                    ui.label(format!("Ao25: {}", self.ao25));
                }
                if self.ao50.as_str() != "" {
                    ui.label(format!("Ao50: {}", self.ao50));
                }
                if self.ao100.as_str() != "" {
                    ui.label(format!("Ao100: {}", self.ao100));
                }
                if self.ao500.as_str() != "" {
                    ui.label(format!("Ao500: {}", self.ao500));
                }
                if self.ao1000.as_str() != "" {
                    ui.label(format!("Ao1000: {}", self.ao1000));
                }
                if self.ao2000.as_str() != "" {
                    ui.label(format!("Ao2000: {}", self.ao2000));
                }
                if self.ao5000.as_str() != "" {
                    ui.label(format!("Ao5000: {}", self.ao2000));
                }
                ui.separator();
                ui.heading("Settings");
                if ui.button("Toggle Settings").clicked() {
                    if self.settings_open == true {
                        self.settings_open = false;
                        self.importing = false;
                        self.imported_fail = "".to_string();
                        self.imported_data = "".to_string();
                    } else {
                        self.settings_open = true;
                    }
                }

                ui.separator();
                ui.heading("Solves");
                egui::scroll_area::ScrollArea::vertical().show(ui, |ui| {
                    for i in 0..self.solves.len() {
                        let text: String;
                        if self.solves[i].dnf == true {
                            text = "Solve: DNF    ".to_string();
                        } else {
                            text = format!(
                                "Solve: {}    ",
                                round(self.solves[i].time.parse().unwrap(), self.solves_prec)
                            )
                        }
                        if ui
                            .button(text).clicked() == true
                        {
                            self.solve_info = true;
                            self.solve_index = i;
                            self.solve_info_copy = format!(
                                "{} @ {} {}",
                                round(
                                    self.solves[i].time.parse().unwrap(),
                                    self.solves_prec
                                ),
                                self.solves[i].scramble,
                                self.solves[i].comment
                            );
                        }
                    }
                });
            });
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.timeron == false {
                if self.settings_open == true {
                    egui::Window::new("Settings").show(ctx, |ui| {
                        ui.heading("Help");
                        if ui.button("About Cubism").clicked() == true {
                            self.used = false;
                        }
                        ui.separator();
                        ui.heading("Stats");
                        if ui.button("Reset Progress").clicked() == true {
                            self.ao5 = "".to_string();
                            self.ao12 = "".to_string();
                            self.ao25 = "".to_string();
                            self.ao50 = "".to_string();
                            self.ao100 = "".to_string();
                            self.ao500 = "".to_string();
                            self.ao1000 = "".to_string();
                            self.ao2000 = "".to_string();
                            self.ao5000 = "".to_string();
                            self.reset_time();
                            self.timeron = false;
                            self.solves = vec![];
                            self.fmt_solves = vec![];
                            self.show_solve = false;
                        }
                        if ui.button("Import from CSTimer").clicked() == true {
                            if self.importing == true {
                                self.importing = false;
                                self.imported_fail = "".to_string();
                                self.imported_data = "".to_string();
                            } else {
                                self.importing = true;
                            }
                        }
                        ui.horizontal(|ui| {
                            ui.label("Average Precision: ");
                            ui.add(egui::widgets::Slider::new(&mut self.ao5_prec, 0..=6));
                            if self.ao5_prec != self.old_ao5_prec {
                                self.refresh_averages();
                                self.old_ao5_prec = self.ao5_prec;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Timer Precision: ");
                            ui.add(egui::widgets::Slider::new(&mut self.prec, 0..=3));
                            if self.prec != self.old_prec {
                                if self.solves.len() != 0 {
                                    self.time = round(self.solves[0].time.parse::<f64>().unwrap(), self.prec).to_string();
                                }
                                self.old_prec = self.prec;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Solves Precision: ");
                            ui.add(egui::widgets::Slider::new(&mut self.solves_prec, 0..=3));
                            if self.solves_prec != self.old_solves_prec {
                                self.redraw_solves();
                                self.old_solves_prec = self.solves_prec;
                            }
                        });
                        ui.separator();
                        ui.heading("Style");
                        ui.horizontal(|ui| {
                            ui.label("Background Colour");
                            ui.color_edit_button_srgb(&mut self.background);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Window Colour");
                            ui.color_edit_button_srgb(&mut self.window);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Button Colour");
                            ui.color_edit_button_srgb(&mut self.button);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Text Colour");
                            ui.color_edit_button_srgb(&mut self.text);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Button Outline");
                            ui.color_edit_button_srgb(&mut self.outline);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Outline Width");
                            ui.add(egui::widgets::Slider::new(&mut self.outline_w, 0.0..=1.5));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Title Bar Colour");
                            ui.color_edit_button_srgb(&mut self.titlebar);
                        })

                    });
                }
                if self.importing == true {
                    egui::Window::new("Import from CSTimer").show(ctx, |ui| {
                        ui.label("Open CSTimer, click export, and click export to file. Copy the contents of that file and paste it here.");
                        ui.text_edit_singleline(&mut self.imported_data);
                        ui.horizontal(|ui| {
                            if ui.button("Import").clicked() == true {
                                let data: serde_json::Value;
                                let result = serde_json::from_str(&self.imported_data);
                                if let Err(e) = result {
                                    println!("{}", e);
                                    self.imported_fail = "Invalid JSON!".to_string();
                                } else {
                                    data = result.unwrap();
                                    if let Some(session) = data["session1"].as_array() {
                                        self.imported_fail = "Imported data!".to_string();
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
                                            self.solves.insert(0, solve);
                                        }
                                        self.redraw_solves();
                                        self.refresh_averages();
                                        self.scramble_text = format!("{} @ {}", round(self.solves[0].time.parse().unwrap(), self.solves_prec), self.solves[0].scramble);
                                    } else {
                                        self.imported_fail = "Failed to import data!".to_string();
                                    }
                                }
                            }
                            if ui.button("Close").clicked() == true{
                                self.importing = false;
                                self.imported_fail = "".to_string();
                                self.imported_data = "".to_string();
                            }
                        });
                        ui.label(format!("{}", self.imported_fail));
                    });
                }
            }
            if self.timeron == false {
                if self.used == false {
                    egui::Window::new("About Cubism").show(ctx, |ui| {
                        ui.label("Cubism is like CSTimer, but the way you start the timer is different. Press the space bar to start and stop, no need to hold. You cannot record times under 0.25 seconds!");
                        ui.hyperlink_to(
                            "Click here for logo credits.",
                            "https://www.flaticon.com/free-icons/3d-cube",
                        );
                        if ui.button("I Understand").clicked() == true {
                            self.used = true;
                        }
                    });
                }
                if self.show_solve == true {
                    egui::Window::new("Solve Stats").show(ctx, |ui| {
                        let displayed: String;
                        if self.solves[0].dnf == true {
                            displayed = "DNF".to_string();
                        } else {
                            displayed = round(self.solves[0].time.parse().unwrap(), self.solves_prec).to_string();
                        }
                        ui.heading(format!("{}", displayed));
                        ui.label(format!("{}", self.solves[0].scramble));
                        ui.horizontal(|ui| {
                            if ui.button("+2").clicked() {
                                if !self.solves[0].plus2 {
                                    self.solves[0].plus2 = true;
                                    self.solves[0].dnf = false;
                                    self.solves[0].time = (self.solves[0].time.parse::<f64>().unwrap() + 2.0).to_string();
                                    self.scramble_text = format!("{}+2 @ {}", round(self.solves[0].clone().time.parse::<f64>().unwrap() - 2.0, self.prec), self.solves[0].scramble);
                                    self.refresh_averages();
                                }
                            } 
                            if ui.button("DNF").clicked() {
                                if !self.solves[0].dnf {
                                    if self.solves[0].plus2 {
                                        self.solves[0].time = (self.solves[0].time.parse::<f64>().unwrap() - 2.0).to_string();
                                    }
                                    self.solves[0].plus2 = false;
                                    self.solves[0].dnf = true;
                                    self.scramble_text = format!("DNF [{}] @ {}", round(self.solves[0].time.parse().unwrap(), self.prec), self.solves[0].scramble);
                                    self.refresh_averages();
                                }
                            }
                            if ui.button("OK").clicked() {
                                if self.solves[0].plus2 {
                                    self.solves[0].time = (self.solves[0].time.parse::<f64>().unwrap() - 2.0).to_string();
                                }
                                self.solves[0].plus2 = false;
                                self.solves[0].dnf = false;
                                self.scramble_text = format!("{} @ {}", round(self.solves[0].time.parse().unwrap(), self.prec), self.solves[0].scramble);
                                self.refresh_averages();
                            }
                        });
                        // Time
                        ui.horizontal(|ui| {
                            ui.label("Copyable:  ");
                            ui.text_edit_singleline(&mut self.scramble_text.as_str());
                        });
                    });
                }
                if self.solve_info == true {
                    egui::Window::new("Solve Info").show(ctx, |ui| {
                        ui.heading(format!("{}", round(self.solves[self.solve_index].time.parse().unwrap(), self.solves_prec)));
                        if self.solves[self.solve_index].dnf == true {
                            ui.label("Did not finish");
                        }
                        if self.solves[self.solve_index].plus2 == true {
                            ui.label("+2 Penalty");
                        }
                        ui.label(format!("Scramble: {}", self.solves[self.solve_index].scramble));
                        ui.horizontal(|ui| {
                            ui.label("Comment: ");
                            ui.text_edit_singleline(&mut self.solves[self.solve_index].comment);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Copyable:  ");
                            ui.text_edit_singleline(&mut self.solve_info_copy.as_str());
                        });
                        if ui.button("Close").clicked() == true {
                            self.solve_info_copy = "".to_string();
                            self.solve_info = false;
                        }
                    });
                }
            }
            if self.timeron == true {
                ctx.request_repaint();
            }
            ctx.input(|i| {
                for event in i.clone().events {
                    match event {
                        egui::Event::Key { key, .. } => {
                            let delta = Local::now().signed_duration_since(self.debounce);
                            if delta > TimeDelta::try_milliseconds(250).unwrap() {
                                self.debounce = Local::now();
                                if self.importing == false && self.solve_info == false {
                                    if key == egui::Key::Space {
                                        if self.timeron == false {
                                            self.timeron = true;
                                            self.starttime = Local::now();
                                        } else {
                                            self.timeron = false;
                                            let rawtime: f64 = Local::now().signed_duration_since(self.starttime).num_microseconds().unwrap() as f64 / 1000000 as f64;
                                            let timertime = round(rawtime,
                                                self.prec
                                            );
                                            let solvetime = round(rawtime,
                                                self.solves_prec
                                            );

                                            let solve = SolveStats {
                                                time: rawtime.to_string(),
                                                scramble: self.scramble.clone(),
                                                timestamp: timestamp(),
                                                ..SolveStats::default()
                                            };

                                            self.solves.insert(0, solve);

                                            self.time = timertime.to_string();
                                            self.fmt_solves.insert(0, solvetime.to_string());
                                            self.refresh_averages();
                                            self.scramble = self.make_scramble();
                                            self.scramble_text = format!("{} @ {}", round(self.solves[0].time.parse().unwrap(), self.prec), self.solves[0].scramble);
                                            self.show_solve = true;
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
                    if self.timeron == true {
                        let secs = {
                            let duration: f64 = Local::now().signed_duration_since(self.starttime).num_microseconds().unwrap() as f64 / 1000000 as f64;

                            let duration = round(duration, self.prec);
                            let duration = duration.to_string();
                            let parts: Vec<&str> = duration.split(".").collect();
                            if parts.len() == 1 {
                                "0".to_string()
                            } else {
                                let main = parts[0].to_string();
                                let mut dec = parts[1].to_string();
                                if dec.len() < self.prec {
                                    loop {
                                        if dec.len() != self.prec {
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
                        ui.heading(format!("{}", self.time));
                    }
                    ui.style_mut().text_styles.get_mut(&egui::TextStyle::Heading).unwrap().size = 30.0;
                },
            )
        });
    }
}
