use chrono::{DateTime, Local, TimeDelta};
use cubesim::{Cube, Move, MoveVariant};
#[cfg(not(target_arch = "wasm32"))]
use discord_rich_presence::{activity::{self, Assets}, DiscordIpc};
use rand::Rng;
use std::collections::HashMap;

use self::scramble::{Cubes, Scrambler};
mod scramble;

// Wasm detection

#[cfg(target_arch = "wasm32")]
fn is_wasm() -> bool {
    true
}

#[cfg(not(target_arch = "wasm32"))]
fn is_wasm() -> bool {
    false
}

fn move_string(movement: cubesim::Move) -> String {
    match movement {
        Move::U(variant) => format_move("U", variant),
        Move::F(variant) => format_move("F", variant),
        Move::L(variant) => format_move("L", variant),
        Move::R(variant) => format_move("R", variant),
        Move::D(variant) => format_move("D", variant),
        Move::B(variant) => format_move("B", variant),
        Move::X(variant) => format_move("X", variant),
        Move::Y(variant) => format_move("Y", variant),
        Move::Z(variant) => format_move("Z", variant),
        Move::Rw(val, variant) => format_move_with_val("Rw", val, variant),
        Move::Fw(val, variant) => format_move_with_val("Fw", val, variant),
        Move::Lw(val, variant) => format_move_with_val("Lw", val, variant),
        Move::Dw(val, variant) => format_move_with_val("Dw", val, variant),
        Move::Uw(val, variant) => format_move_with_val("Uw", val, variant),
        Move::Bw(val, variant) => format_move_with_val("Bw", val, variant),
    }
}

fn format_move(move_str: &str, variant: cubesim::MoveVariant) -> String {
    match variant {
        MoveVariant::Double => format!("{}2", move_str),
        MoveVariant::Standard => move_str.to_string(),
        MoveVariant::Inverse => format!("{}'", move_str),
    }
}

fn format_move_with_val(move_str: &str, val: i32, variant: cubesim::MoveVariant) -> String {
    if val == 1 {
        format_move(move_str, variant)
    } else {
        format_move(format!("{}{}w", val, move_str).as_str(), variant)
    }
}


#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug)]
pub struct SolveStats {
    time: String,
    scramble: String,
    timestamp: String,
    comment: String,
    plus2: bool,
    dnf: bool,
    cube_type: Cubes,
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
            cube_type: Cubes::ThreeByThree,
        }
    }
}

fn solve(scramble: String) -> String {
    let moves = cubesim::parse_scramble(scramble);
    let cube = cubesim::FaceletCube::new(3);
    let cube = cube.apply_moves(&moves);
    let solution = match cubesim::solve(&cube) {
        Some(data) => data,
        None => return "Bad Scramble".to_string()
    };
    let mut result = String::new();
    for movement in solution {
        result.push_str(format!("{} ", move_string(movement).to_string()).as_str());
    }
    result

}

// State

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Debug)]
pub struct State {
    // Time-related fields
    time: String,     // Time as seen in timer
    starttime: DateTime<Local>, // Start time of the timer
    timeron: bool,    // Indicates whether the timer is currently running
    debounce: DateTime<Local>, // Used for debouncing timer inputs

    // Scramble-related fields
    scramble: String, // Scramble for the puzzle
    scramble_text: String, // Text representation of the scramble
    c_scramble: String, // Compressed scramble

    // Averages and statistics
    ao5: String,      // Average of 5 solves
    ao12: String,     // Average of 12 solves
    ao25: String,     // Average of 25 solves
    ao50: String,     // Average of 50 solves
    ao100: String,    // Average of 100 solves
    ao500: String,    // Average of 500 solves
    ao1000: String,   // Average of 1000 solves
    ao2000: String,   // Average of 2000 solves
    ao5000: String,   // Average of 5000 solves
    mo3: String,      // Mean of 3 solves
    mean: String,     // Mean of all solves
    
    // Best Averages
    best_ao5: String,
    best_ao12: String,
    best_ao25: String,
    best_ao50: String,
    best_ao100: String,
    best_ao500: String,
    best_ao1000: String,
    best_ao2000: String,
    best_ao5000: String,
    best_mo3: String,

    // Solve-related fields
    solves: Vec<SolveStats>, // Vector containing statistics of individual solves
    solve_index: usize,      // Index of the current solve
    show_solve: bool,        // Indicates whether solve details are being displayed
    solve_info: bool,        // Indicates whether solve information is shown
    solve_info_copy: String, // Copy of solve information

    // Settings and UI-related fields
    settings_open: bool,      // Indicates whether settings are open
    importing: bool,          // Indicates whether data is being imported
    imported_data: String,    // Imported data
    imported_fail: String,    // Message for failed imports
    prec: usize,              // Precision for numeric display
    ao5_prec: usize,          // Precision for ao5 display
    solves_prec: usize,       // Precision for solves display
    old_ao5_prec: usize,      // Old precision for ao5
    old_solves_prec: usize,   // Old precision for solves
    old_prec: usize,          // Old precision
    fmt_solves: Vec<String>,  // Formatted solves for display
    background: [u8; 3],      // Background color
    window: [u8; 3],          // Window color
    button: [u8; 3],          // Button color
    outline: [u8; 3],         // Outline color
    titlebar: [u8; 3],        // Title bar color
    text: [u8; 3],            // Text color
    outline_w: f32,           // Outline width
    widget: [u8; 3],          // Widget color
    show_solve_info: bool,    // Indicates whether solve info is being shown
    show_tools: bool,         // Indicates whether tools are being shown
    current_tool: String,     // Current selected tool
    plottable: Vec<[f64; 2]>, // Vector of plottable data
    plot_aspect_ratio: f32,   // Aspect ratio for plots
    stats_open: bool,         // Whether to have Stats menu open
    show_left_bar: bool,      // Whether to show the left (main) bar of the screen
    footer: String,           // Custom footer text (for youtubers etc)
    show_footer: bool,

    // Puzzle-related fields
    name: String,             // Name of the puzzle
    cube_type: Cubes,         // Type of the cube
    cube_type_old: Cubes,     // Old type of the cube
    show_scramble: bool,      // Indicates whether scramble is being shown
    solution: String,         // Solution for the puzzle
    c_solution: String,       // Compressed solution

    // Initiation fields
    download: bool,           // Whether the Download the Desktop App prompt has been closed
    used: bool,               // Whether the About Cubism menu has been shown
}

impl Default for State {
    fn default() -> Self {
        Self {
            // Customizable Footer
            footer: "Cubism by Aityz (This text is customizable)".to_string(),

            // Cubes Configuration
            show_scramble: true,
            cube_type_old: Cubes::ThreeByThree,
            cube_type: Cubes::ThreeByThree,
            scramble_text: "".to_string(),

            // Scramble & Solution
            c_scramble: "".to_string(),
            c_solution: "".to_string(),
            solution: "".to_string(),

            // Widget & Tools
            widget: [150, 150, 150],
            show_tools: true,

            // Plotting
            plottable: vec![],
            plot_aspect_ratio: 2.0,

            // First Load 
            download: true,
            used: false,

            // Solve Info
            solve_index: 0,
            solve_info: false,
            solve_info_copy: "".to_string(),

            // Timing
            time: "0.00".to_string(),
            starttime: Local::now(),
            timeron: false,
            debounce: Local::now(),

            // Averages
            ao5: "".to_string(),
            ao12: "".to_string(),
            ao25: "".to_string(),
            ao50: "".to_string(),
            ao100: "".to_string(),
            ao500: "".to_string(),
            ao1000: "".to_string(),
            ao2000: "".to_string(),
            ao5000: "".to_string(),
            mo3: "".to_string(),
            mean: "".to_string(),

            // Best Averages
            best_ao5: "".to_string(),
            best_ao12: "".to_string(),
            best_ao25: "".to_string(),
            best_ao50: "".to_string(),
            best_ao100: "".to_string(),
            best_ao500: "".to_string(),
            best_ao1000: "".to_string(),
            best_ao2000: "".to_string(),
            best_ao5000: "".to_string(),
            best_mo3: "".to_string(),

            // Solves & Importing
            solves: vec![],
            importing: false,
            imported_data: "".to_string(),
            imported_fail: "".to_string(),

            // Precision Settings
            prec: 2,
            ao5_prec: 3,
            solves_prec: 2,
            old_prec: 2,
            old_ao5_prec: 3,
            old_solves_prec: 2,

            // Formatting
            fmt_solves: vec![],

            // UI Colors
            background: [255, 255, 255],
            window: [255, 255, 255],
            button: [255, 255, 255],
            titlebar: [255, 255, 255],
            outline: [0, 0, 0],
            text: [0, 0, 0],

            // Visual Options
            outline_w: 0.5,
            show_solve: false,
            show_footer: true,
            show_solve_info: true,
            current_tool: "Select Tool".to_string(),

            // Application Settings
            name: "Default".to_string(),
            stats_open: false,
            show_left_bar: true,
            settings_open: false,

            // Scramble
            scramble: "".to_string(),
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

fn mean(solves: &Vec<SolveStats>, number: usize, prec: usize) -> String {
    let latest: Vec<SolveStats> = solves.get(0..=number).unwrap().to_vec();
    let mut total: f64 = 0.0;
    for solve in &latest {
        total += solve.time.parse::<f64>().unwrap();
    }
    round(total / latest.len() as f64, prec).to_string()
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Cubism {
    state: State,
    sessions: HashMap<String, State>,
    #[serde(skip)]
    set_font: bool,
    #[serde(skip)]
    started: bool,
}

impl Default for Cubism {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
            state: State::default(),
            set_font: false,
            started: false,
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
        let scrambler = Scrambler::from(self.state.cube_type.clone());
        scrambler.scramble()
    }
    pub fn refresh_averages(&mut self) {
        let len = self.state.solves.len();
        let solves = &self.state.solves;
        if len > 2 {
            let mo3 = average(&solves, 2, self.state.ao5_prec);
            if self.state.mo3 > mo3 {
                self.state.best_mo3 = mo3.clone();
            }
            self.state.mo3 = mo3;
        }
        if len > 4 {
            let ao5 = average(&solves, 4, self.state.ao5_prec);
            if self.state.ao5 > ao5 {
                self.state.best_ao5 = ao5.clone();
            }
            self.state.ao5 = ao5;
        }
        if len > 11 {
            let ao12 = average(&solves, 11, self.state.ao5_prec);
            if self.state.ao12 > ao12 {
                self.state.best_ao12 = ao12.clone();
            }
            self.state.ao12 = ao12;
        }
        if len > 24 {
            let ao25 = average(&solves, 24, self.state.ao5_prec);
            if self.state.ao25 > ao25 {
                self.state.best_ao25 = ao25.clone();
            }
            self.state.ao25 = ao25;
        }
        if len > 49 {
            let ao50 = average(&solves, 49, self.state.ao5_prec);
            if self.state.ao50 > ao50 {
                self.state.best_ao50 = ao50.clone();
            }
            self.state.ao50 = ao50;
        }
        if len > 99 {
            let ao100 = average(&solves, 99, self.state.ao5_prec);
            if self.state.ao100 > ao100 {
                self.state.best_ao100 = ao100.clone();
            }
            self.state.ao100 = ao100;
        }
        if len > 499 {
            let ao500 = average(&solves, 499, self.state.ao5_prec);
            if self.state.ao500 > ao500 {
                self.state.best_ao500 = ao500.clone();
            }
            self.state.ao500 = ao500;
        }
        if len > 999 {
            let ao1000 = average(&solves, 999, self.state.ao5_prec);
            if self.state.ao1000 > ao1000 {
                self.state.best_ao1000 = ao1000.clone();
            }
            self.state.ao1000 = ao1000;
        }
        if len > 1999 {
            let ao2000 = average(&solves, 1999, self.state.ao5_prec);
            if self.state.ao2000 > ao2000 {
                self.state.best_ao2000 = ao2000.clone();
            }
            self.state.ao2000 = ao2000;
        }
        if len > 4999 {
            let ao5000 = average(&solves, 4999, self.state.ao5_prec);
            if self.state.ao5000 > ao5000 {
                self.state.best_ao5000 = ao5000.clone();
            }
            self.state.ao5000 = ao5000;
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

#[cfg(not(target_arch = "wasm32"))]
fn rpc() {
    let mut client = discord_rich_presence::DiscordIpcClient::new("1217391447695818824").unwrap();
    let connection = client.connect();
    match connection {
        Err(e) => println!("{}", e),
        Ok(_) => {
            loop {
                client.set_activity(activity::Activity::new()
                    .details("A speedcubing timer built in Rust")
                    .state("Try it out at cubetimer.github.io")
                    .assets(Assets::new().
                        large_image("logo")
                        .large_text("Cubism Timer")) 
                ).unwrap();
                std::thread::sleep(std::time::Duration::from_secs(5));
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
        if self.started == false {
            if is_wasm() == false {
                #[cfg(not(target_arch = "wasm32"))]
                rpc();
            }
            self.started = true;
        }
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
            if self.state.show_scramble == true {
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
        }

        if self.state.timeron == false {
            if self.state.show_footer == true {
                egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| {
                            ui.label(format!("{}", self.state.footer));
                        },
                    );
                });
            }
        }

        if self.state.timeron == false {
            if self.state.show_left_bar == true {
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

                    ui.horizontal(|ui| {
                        ui.label("Session Name");
                        ui.text_edit_singleline(&mut self.state.name);
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
                        ui.label("Open Statistics: ");
                        if ui.radio(self.state.stats_open, "").clicked() {
                            if self.state.stats_open == true {
                                self.state.stats_open = false;
                            } else {
                                self.state.stats_open = true;
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Open Solve Stats: ");
                        if ui.radio(self.state.show_solve_info, "").clicked() {
                            if self.state.show_solve_info == true {
                                self.state.show_solve_info = false;
                            } else {
                                self.state.show_solve_info = true;
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Open Tools: ");
                        if ui.radio(self.state.show_tools, "").clicked() {
                            if self.state.show_tools == true {
                                self.state.show_tools = false;
                            } else {
                                self.state.show_tools = true;
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Show Scramble: ");
                        if ui.radio(self.state.show_scramble, "").clicked() {
                            if self.state.show_scramble == true {
                                self.state.show_scramble = false;
                            } else {
                                self.state.show_scramble = true;
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Show Footer: ");
                        if ui.radio(self.state.show_footer, "").clicked() {
                            if self.state.show_footer == true {
                                self.state.show_footer = false;
                            } else {
                                self.state.show_footer = true;
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Show Left Bar: ");
                        if ui.radio(self.state.show_left_bar, "").clicked() {
                            if self.state.show_left_bar == true {
                                self.state.show_left_bar = false;
                            } else {
                                self.state.show_left_bar = true;
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
                                if self.state.solves[i].comment.as_str() != "" {
                                    text = format!(
                                        "*Solve: {}",
                                        round(
                                            self.state.solves[i].time.parse().unwrap(),
                                            self.state.solves_prec
                                        )
                                    )
                                } else {
                                    text = format!(
                                        "Solve: {}",
                                        round(
                                            self.state.solves[i].time.parse().unwrap(),
                                            self.state.solves_prec
                                        )
                                    );
                                }
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
                         } // TODO Fix CSTimer Import
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
                            ui.label("Cube Type: ");
                            egui::ComboBox::from_label("").selected_text(format!("{}", self.state.cube_type)).show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.state.cube_type, Cubes::TwoByTwo, "2x2");
                                ui.selectable_value(&mut self.state.cube_type, Cubes::ThreeByThree, "3x3");
                                ui.selectable_value(&mut self.state.cube_type, Cubes::FourByFour, "4x4");
                                ui.selectable_value(&mut self.state.cube_type, Cubes::FiveByFive, "5x5");
                            });
                            if self.state.cube_type_old != self.state.cube_type {
                                self.state.scramble = self.make_scramble();
                                self.state.cube_type_old = self.state.cube_type;
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
                        ui.horizontal(|ui| {
                            ui.label("Footer Text");
                            ui.text_edit_singleline(&mut self.state.footer);
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
                                        if self.state.solves.len() > 2 {
                                            self.state.mo3 = mean(&self.state.solves.get(0..=2).unwrap().to_vec(), 2, self.state.ao5_prec);
                                        }
                                        if self.state.solves.len() != 0 {
                                            self.state.mean = mean(&self.state.solves, self.state.solves.len() - 1, self.state.ao5_prec);
                                        }
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
                        let portion = self.state.solves.get(self.state.solve_index..self.state.solves.len());
                        match portion {
                            Some(data) => {
                                if data.len() > 2 {
                                    ui.label(format!("Mo3: {}", mean(&data.to_vec(), 2, self.state.ao5_prec)));
                                }
                            },
                            None => {
                            }
                        };
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
                        let mut dont_redraw = false;
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
                                dont_redraw = true;
                                // self.state.time = round(self.state.solves[self.state.solve_index].time.parse().unwrap(), self.state.prec).to_string();
                                self.state.solve_info = false;
                                if self.state.solves.len() == 0 {
                                    self.state.show_solve = false;
                                }
                                self.refresh_averages();
                                self.redraw_solves();
                            }
                        });
                        if dont_redraw == false {
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
                                    ui.selectable_value(&mut self.state.current_tool, "Solve".to_string(), "Solve");
                                    ui.selectable_value(&mut self.state.current_tool, "Custom Solve".to_string(), "Custom Solve");
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
                        if self.state.current_tool == "Solve".to_string() {
                            ui.separator();
                            ui.heading("Solve");
                            ui.label("Solve the current Rubik's cube scramble in a low amount of moves");
                            if ui.button("Solve").clicked() {
                                self.state.solution = solve(self.state.scramble.clone());
                            }
                            ui.label(format!("Solution: {}", self.state.solution));
                        }
                        if self.state.current_tool == "Custom Solve".to_string() {
                            ui.separator();
                            ui.heading("Custom Solve");
                            ui.label("Input a scramble and the solver will propose a solution!");
                            ui.text_edit_singleline(&mut self.state.c_scramble);
                            if ui.button("Solve").clicked() {
                                self.state.c_solution = solve(self.state.c_scramble.clone());
                            }
                            ui.label(format!("Solution: {}", self.state.c_solution));
                        }
                    });
                }
                if self.state.stats_open == true {
                    egui::Window::new("Statistics").show(ctx, |ui| {
                    ui.label(format!("Solves: {}", self.state.solves.len()));
                    if self.state.mean.as_str() != "" {
                        ui.label(format!("Mean: {}", self.state.mean));
                    }
                    ui.separator();
                    if self.state.mo3.as_str() != "" {
                        ui.label(format!("Mo3: {}", self.state.mo3))
                            .on_hover_text(format!("Best Mo3: {}", self.state.best_mo3));
                    }
                    if self.state.ao5.as_str() != "" {
                        ui.label(format!("Ao5: {}", self.state.ao5))
                            .on_hover_text(format!("Best Ao5: {}", self.state.best_ao5));
                    }
                    if self.state.ao12.as_str() != "" {
                        ui.label(format!("Ao12: {}", self.state.ao12))
                            .on_hover_text(format!("Best Ao12: {}", self.state.best_ao12));
                    }
                    if self.state.ao25.as_str() != "" {
                        ui.label(format!("Ao25: {}", self.state.ao25))
                            .on_hover_text(format!("Best Ao25: {}", self.state.best_ao25));
                    }
                    if self.state.ao50.as_str() != "" {
                        ui.label(format!("Ao50: {}", self.state.ao50))
                            .on_hover_text(format!("Best Ao50: {}", self.state.best_ao50));
                    }
                    if self.state.ao100.as_str() != "" {
                        ui.label(format!("Ao100: {}", self.state.ao100))
                            .on_hover_text(format!("Best Ao100: {}", self.state.best_ao100));
                    }
                    if self.state.ao500.as_str() != "" {
                        ui.label(format!("Ao500: {}", self.state.ao500))
                            .on_hover_text(format!("Best Ao500: {}", self.state.best_ao500));
                    }
                    if self.state.ao1000.as_str() != "" {
                        ui.label(format!("Ao1000: {}", self.state.ao1000))
                            .on_hover_text(format!("Best Ao1000: {}", self.state.best_ao1000));
                    }
                    if self.state.ao2000.as_str() != "" {
                        ui.label(format!("Ao2000: {}", self.state.ao2000))
                            .on_hover_text(format!("Best Ao2000: {}", self.state.best_ao2000));
                    }
                    if self.state.ao5000.as_str() != "" {
                        ui.label(format!("Ao5000: {}", self.state.ao2000))
                            .on_hover_text(format!("Best Ao5000: {}", self.state.best_ao5000));
                    }
                    });
                }
                if self.state.show_left_bar == false {
                    egui::Window::new("Show Left Bar").show(ctx, |ui| {
                        if ui.button("Show Left Bar").clicked() {
                            self.state.show_left_bar = true;
                        }
                    });
                }
            }
            if self.state.timeron == true {
                ctx.request_repaint();
            }
            if ctx.wants_keyboard_input() == false {
                ctx.input(|i| {
                    for event in i.clone().events {
                        match event {
                            egui::Event::Key { key, .. } => {
                                let delta = Local::now().signed_duration_since(self.state.debounce);
                                if delta > TimeDelta::try_milliseconds(250).unwrap() {
                                    self.state.debounce = Local::now();
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
                                            let solve_: SolveStats;
                                                solve_ = SolveStats {
                                                    time: rawtime.to_string(),
                                                    scramble: self.state.scramble.clone(),
                                                    timestamp: timestamp(),
                                                    cube_type: self.state.cube_type,
                                                    ..SolveStats::default()
                                                };
                                            self.state.solves.insert(0, solve_);

                                            self.state.time = timertime.to_string();
                                            self.state.fmt_solves.insert(0, solvetime.to_string());
                                            if self.state.solves.len() > 2 {
                                                self.state.mo3 = mean(&self.state.solves.get(0..=2).unwrap().to_vec(), 2, self.state.ao5_prec);
                                            }
                                            if self.state.solves.len() != 0 {
                                                self.state.mean = mean(&self.state.solves, self.state.solves.len() - 1, self.state.ao5_prec);
                                            }
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
                            _ => {}
                        }
                    }
                });
            }
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
