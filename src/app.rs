use crate::device;
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ModbusApp {
    // Example stuff:
    label: String,
    devices: Vec<device::ModbusDevice>,
    sel_device_index: usize,
    sel_querry_index: usize,
}

impl Default for ModbusApp {
    fn default() -> Self {
        Self {
            label: "Modbus Scanner".to_owned(),
            devices: vec![device::ModbusDevice::new()],
            sel_device_index: 0,
            sel_querry_index: 0,
        }
    }
}

impl ModbusApp {
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

impl eframe::App for ModbusApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.set_min_width(200.0);
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut dev_index: usize = 0;
                self.devices.retain_mut(|x| {
                    let mut retain = true;
                    let id = ui.make_persistent_id(dev_index);
                    egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(),
                        id,
                        true,
                    )
                    .show_header(ui, |ui| {
                        let tv = ui
                            .toggle_value(&mut x.selected, &x.lable)
                            .context_menu(|ui| {
                                //if dev_index > 0 {                                        Use these to move elements position inside vector, as of now i think i might need chained iterators.
                                //    if ui.button("\u{2B06} Move Up").clicked(){

                                //    }
                                //}
                                //if dev_index < x.querrys.len() {
                                //    if ui.button("\u{2B07} Move Down").clicked(){

                                //    }
                                //}

                                if dev_index > 0 {
                                    if ui.button("\u{1F5D1} Delete").clicked() {
                                        if self.sel_device_index == dev_index {
                                            self.sel_device_index -= 1;
                                        }
                                        retain = false;
                                    }
                                }
                            });

                        if tv.clicked() {
                            self.sel_device_index = dev_index;
                            self.sel_querry_index = usize::MAX;
                        }
                    })
                    .body(|ui| {
                        (self.sel_querry_index, self.sel_device_index) = x.build_querry_tree(
                            ui,
                            self.sel_querry_index,
                            self.sel_device_index,
                            dev_index,
                        );
                    });
                    if self.sel_device_index == dev_index {
                        x.selected = true
                    } else {
                        x.selected = false
                    }
                    dev_index += 1;
                    return retain;
                });

                if ui.button("Add Device").clicked() {
                    self.devices.push(device::ModbusDevice::new());
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.devices[self.sel_device_index].draw_device_frame(ui, self.sel_querry_index);
        });

    }
}

