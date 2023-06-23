#![forbid(unsafe_code)]
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
use reqwest::{Client};
use std::sync::mpsc::{Receiver, Sender};
use egui_extras::{TableBuilder, Column};
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    #[serde(skip)]
    tx: Sender<Vec<Lift>>,
    #[serde(skip)]
    rx: Receiver<Vec<Lift>>,
    label: String,
    lifts: Vec<Lift>,
}

#[derive(serde::Deserialize,serde::Serialize, Debug, Clone, PartialEq)]
struct Lift{
    id:i32,
    lift: String,
    weight:i32,
    reps:i32,
    rpe:i32,
    time:chrono::DateTime<chrono::Utc>
}


impl Default for TemplateApp {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            // Example stuff:
            tx,
            rx,
            lifts: Vec::new(),
            label: "Hello World!".to_owned(),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

fn get_lifts(tx: Sender<Vec<Lift>>, ctx: egui::Context){
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move{
        let body: Vec<Lift> = Client::default()
            .get("http://192.168.1.38:8080/api/workout/lifts")
            .send()
            .await
            .expect("Unable to send request")
            .json()
            .await
            .expect("Unable to parse response");

        // After parsing the response, notify the GUI thread of the increment value.
        let _ = tx.send(body);
        ctx.request_repaint();
    });

    #[cfg(not(target_arch = "wasm32"))]
    tokio::spawn(async move {
        let body: Vec<Lift> = Client::default()
            .get("http://192.168.1.38:8080/api/workout/lifts")
            .send()
            .await
            .expect("Unable to send request")
            .json()
            .await
            .expect("Unable to parse response");

        // After parsing the response, notify the GUI thread of the increment value.
        let _ = tx.send(body);
        ctx.request_repaint();
    });
}


impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(lifts) = self.rx.try_recv() {
            self.lifts = lifts;
        }
        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
           
                TableBuilder::new(ui)
                    .columns(Column::remainder(),5)
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("Reps");
                        });
                        header.col(|ui| {
                            ui.heading("Weight");
                        });
                        header.col(|ui| {
                            ui.heading("Exercise Type");
                        });
                        header.col(|ui| {
                            ui.heading("Date");
                        });
                        header.col(|ui| {
                            ui.heading("RPE");
                        });
                    })
                    .body(|mut body| {
                        for lift in &self.lifts {
                            body.row(30.0, |mut row| {
                                
                                    row.col(|ui| {
                                        ui.label(lift.reps.to_string());
                                    });
                                    row.col(|ui| {
                                        ui.label(lift.weight.to_string());
                                    });
                                    row.col(|ui| {
                                        ui.label(&lift.lift);
                                    });
                                    row.col(|ui| {
                                        ui.label(&lift.time.to_string());
                                    });
                                    row.col(|ui| {
                                        ui.label(lift.rpe.to_string());
                                    });

                                
                            });
                        }
                    });
            if ui.button(format!("Refresh")).clicked() {
                get_lifts(self.tx.clone(), ctx.clone());
            }
            egui::warn_if_debug_build(ui);
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }
    }
}
