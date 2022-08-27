use std::sync::{Arc, Mutex};
use eframe::{egui::*, App};

const RED: Color32 = Color32::from_rgb(255, 0, 0);
const GREEN: Color32 = Color32::from_rgb(0, 255, 0);

#[derive(Debug)]
pub struct Data {
    pub watch_path: String,
    pub target_path: String,
    pub is_watching: bool,
    pub switch_ip: String,
    pub ftp_port: u16,
    pub ftp_user: String,
    pub ftp_pass: String,
}

pub struct MainApp {
    pub data: Arc<Mutex<Data>>
}

impl Default for Data {
    fn default() -> Self {
        Self {
            // watch_path: "E:\\Modding\\Ultimate\\Auto-Refresh".to_owned(),
            watch_path: "".to_owned(),
            target_path: "".to_owned(),
            // target_path: "ftp:\\ultimate\\mods\\Auto-Refresh".to_owned(),
            is_watching: false,
            // switch_ip: "10.0.0.143".to_owned(),
            switch_ip: "".to_owned(),
            ftp_port: 5000,
            ftp_user: "".to_owned(),
            ftp_pass: "".to_owned(),
        }
    }
}

impl Default for MainApp {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(Data::default()))
        }
    }
}

impl MainApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl App for MainApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Auto-Refresh Client");
            });

            ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                Grid::new("main_data_grid").show(ui, |ui| {
                    ui.label("Watch Mod Path: ");
                    ui.text_edit_singleline(&mut self.data.lock().unwrap().watch_path);
                    ui.end_row();
    
                    ui.label("Target Path (if ftp, then use 'ftp:\\' as a prefix): ");
                    ui.text_edit_singleline(&mut self.data.lock().unwrap().target_path);
                    ui.end_row();
    
                    ui.label("Switch IP: ");
                    ui.text_edit_singleline(&mut self.data.lock().unwrap().switch_ip);
                    ui.end_row();
    
                    ui.label("FTP Username: ");
                    ui.text_edit_singleline(&mut self.data.lock().unwrap().ftp_user);
                    ui.end_row();
                    
                    ui.label("FTP Password: ");
                    ui.text_edit_singleline(&mut self.data.lock().unwrap().ftp_pass);
                    ui.end_row();
                    
                    ui.label("FTP Port: ");
                    ui.add(Slider::new(&mut self.data.lock().unwrap().ftp_port, 0..=u16::MAX));
                    ui.end_row();
                    
                    ui.label("");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("Toggle Watcher").clicked() {
                            let original_state = self.data.lock().unwrap().is_watching;
                            self.data.lock().unwrap().is_watching = !original_state;
                        }
                    });
                    ui.end_row();
                    
                    ui.label("Watcher Status:");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.colored_label(if self.data.lock().unwrap().is_watching {GREEN} else {RED}, format!("{}", if self.data.lock().unwrap().is_watching { "Watching" } else { "Not Watching" }));
                    });
                    ui.end_row();
                });
            });
        });
    }
}