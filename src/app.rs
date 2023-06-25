#![forbid(unsafe_code)]
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
use reqwest::{Client};
use std::sync::mpsc::{Receiver, Sender};
use egui_extras::{TableBuilder, Column};
#[cfg(target_arch = "wasm32")]
use web_sys::{Window};
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state

pub struct TemplateApp {
    #[serde(skip)]
    tx: Sender<Vec<Lift>>,
    #[serde(skip)]
    rx: Receiver<Vec<Lift>>,
    label: String,
    lifts: Vec<Lift>,
    submitlift_open: bool,
    newLift: NewLift,
    liftType: LiftType,
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

#[derive(serde::Serialize,serde::Deserialize, Default, Debug)]
struct NewLift{
    pub lift: String,
    pub weight:i32,
    pub reps:i32,
    pub rpe:i32,
    pub time:chrono::DateTime<chrono::Utc>
}
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
enum LiftType {
    Bench,
    Squat,
    Deadlift,
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
            submitlift_open : false,
            newLift:NewLift { lift: ("Bench".to_string()), ..Default::default()},
            liftType: LiftType::Bench,
        }
    }
}

fn get_lifts(tx: Sender<Vec<Lift>>, ctx: egui::Context){
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move{
        let url = web_sys::window().unwrap().location().origin().unwrap() + "/api/workout/lifts";
        let body: Vec<Lift> = Client::default()
            .get(url)
            .send()
            .await
            .expect("Unable to send request")
            .json()
            .await
            .expect("Unable to parse response");

        let _ = tx.send(body);
        ctx.request_repaint();
    });

    #[cfg(not(target_arch = "wasm32"))]
    tokio::spawn(async move {
        let url = "http://192.168.1.38:8080/api/workout/lifts";
        let body: Vec<Lift> = Client::default()
            .get(url)
            .send()
            .await
            .expect("Unable to send request")
            .json()
            .await
            .expect("Unable to parse response");

        let _ = tx.send(body);
        ctx.request_repaint();
    });
}

fn write_lift(form_data: NewLift){
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move{
        let url = web_sys::window().unwrap().location().origin().unwrap() + "/api/workout/lifts";
        let json = serde_json::to_string(&form_data).unwrap();
        let _writtenlift:Lift = Client::default()
            .post(url)
            .header("Content-Type", "application/json")
            .body(json)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
    });
    #[cfg(not(target_arch = "wasm32"))]
    tokio::spawn(async move {
        let url = "http://192.168.1.38:8080/api/workout/lifts";
        let json = serde_json::to_string(&form_data).unwrap();
        let _writtenlift:Lift = Client::default()
            .post(url)
            .header("Content-Type", "application/json")
            .body(json)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
    });
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
        let Self {
            // Example stuff:
            tx,
            rx,
            lifts,
            label,
            submitlift_open,
            newLift,
            liftType
        } = self;
        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

         // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                    
                });
            });
            
        });
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");
            if ui.button("Submit Lift").clicked() {
                self.submitlift_open = !self.submitlift_open;
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button(format!("Refresh")).clicked() {
                        get_lifts(self.tx.clone(), ctx.clone());
                    }
                });
                if self.submitlift_open {
                    egui::Window::new("New Lift")
                        .auto_sized()
                        .show(ctx, |ui| {
                            
                            ui.horizontal(|ui| {
                                ui.label("Lift Type:");
                                ui.selectable_value(liftType, LiftType::Bench, "Bench");
                                ui.selectable_value(liftType, LiftType::Squat, "Squat");
                                ui.selectable_value(liftType, LiftType::Deadlift, "Deadlift");
                            });
                            ui.horizontal(|ui| {
                                ui.label("Weight:");
                                ui.add(egui::DragValue::new(&mut self.newLift.weight).speed(1.0));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Reps:");
                                ui.add(egui::DragValue::new(&mut self.newLift.reps).speed(1.0));
                            });
                            ui.horizontal(|ui| {
                                ui.label("RPE:");
                                ui.add(egui::Slider::new(&mut self.newLift.rpe, 0..=10).integer());
                            });
                        // Submit button in the pop-up
                            if ui.button("Submit").clicked() {
                                // Call your function here
                                match liftType{
                                    LiftType::Bench => self.newLift.lift = "Bench".to_owned(),
                                    LiftType::Squat => self.newLift.lift = "Squat".to_owned(),
                                    LiftType::Deadlift => self.newLift.lift = "Deadlift".to_owned(),
                                };
                                let submit_lift = NewLift{
                                    lift: self.newLift.lift.clone(),
                                    weight: self.newLift.weight,
                                    reps: self.newLift.reps,
                                    rpe: self.newLift.rpe,
                                    time: chrono::offset::Utc::now(),
                                };
                                write_lift(submit_lift);
                                self.submitlift_open = false; // Close the pop-up
                            }
                    });
                }
                TableBuilder::new(ui)
                    .columns(Column::auto(),5)
                    .auto_shrink([true, true])
                    .resizable(true)
                    .striped(true)
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
                    
        });
        
    }
}
