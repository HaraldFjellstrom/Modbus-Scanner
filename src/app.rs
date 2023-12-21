use crate::device;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ModbusApp {
    label: String,
    devices: Vec<device::ModbusDevice>,
    sel_device_index: usize,
    sel_query_index: usize,
    device_templates: Vec<std::path::PathBuf>,
    query_templates: Vec<std::path::PathBuf>,
    #[serde(skip)]
    timer : std::time::SystemTime,
}

impl Default for ModbusApp {
    fn default() -> Self {
        Self {
            label: "Modbus Scanner".to_owned(),
            devices: vec![device::ModbusDevice::new()],
            sel_device_index: 0,
            sel_query_index: usize::MAX,
            device_templates: vec![],
            query_templates: vec![],
            timer : std::time::SystemTime::now(),
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

    fn update_templates_lists(&mut self) {
        self.device_templates = vec![];
        for element in eframe::storage_dir("Modbus Scanner")
            .unwrap()
            .read_dir()
            .unwrap()
        {
            let path = element.unwrap().path();
            if let Some(extension) = path.extension() {
                if extension == "device" {
                    self.device_templates.push(path);
                }
            }
        }
        self.query_templates = vec![];
        for element in eframe::storage_dir("Modbus Scanner")
            .unwrap()
            .read_dir()
            .unwrap()
        {
            let path = element.unwrap().path();
            if let Some(extension) = path.extension() {
                if extension == "query" {
                    self.query_templates.push(path);
                }
            }
        }
    }
}

impl eframe::App for ModbusApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
        self.update_templates_lists();
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
                        // Denna funkar, för att slippa importera fler crates för att välja filer
                        if ui.button("Save Device").clicked() {
                            let data = serde_json::to_string(&self.devices[self.sel_device_index])
                                .unwrap();
                            let mut path = eframe::storage_dir("Modbus Scanner").unwrap();
                            path.push("data");
                            path.set_file_name(self.devices[self.sel_device_index].lable.as_str());
                            path.set_extension("device");
                            std::fs::write(path, data).expect("Save Failed");
                            self.update_templates_lists();
                            ui.close_menu();
                        }
                        ui.menu_button("Import Device", |ui| {
                            self.device_templates.iter().for_each(|z| {
                                if ui
                                    .button(z.file_stem().unwrap().to_string_lossy())
                                    .clicked()
                                {
                                    self.devices.push(
                                        serde_json::from_str(
                                            std::fs::read_to_string(z).unwrap().as_str(),
                                        )
                                        .unwrap(),
                                    );
                                    self.sel_device_index = self.devices.len() - 1;
                                    ui.close_menu()
                                }
                            });
                        });
                        ui.separator();
                        if ui.button("Save Query").clicked() {
                            // Error handling needed, especially if no query is selected!!
                            let data = serde_json::to_string(
                                &self.devices[self.sel_device_index].querys[self.sel_query_index],
                            )
                            .unwrap();
                            let mut path = eframe::storage_dir("Modbus Scanner").unwrap();
                            path.push("data");
                            path.set_file_name(
                                self.devices[self.sel_device_index].querys[self.sel_query_index]
                                    .lable
                                    .as_str(),
                            );
                            path.set_extension("query");
                            std::fs::write(path, data).expect("Save Failed");
                            self.update_templates_lists();
                            ui.close_menu();
                        }
                        ui.menu_button("Import Query", |ui| {
                            self.query_templates.iter().for_each(|z| {
                                if ui
                                    .button(z.file_stem().unwrap().to_string_lossy())
                                    .clicked()
                                {
                                    self.devices[self.sel_device_index].querys.push(
                                        serde_json::from_str(
                                            std::fs::read_to_string(z).unwrap().as_str(),
                                        )
                                        .unwrap(),
                                    );
                                    self.sel_query_index =
                                        self.devices[self.sel_device_index].querys.len() - 1;
                                    ui.close_menu();
                                }
                            });
                        });
                        ui.separator();
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
                    egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(),
                        ui.make_persistent_id(dev_index),
                        true,
                    )
                    .show_header(ui, |ui| {
                        if ui
                            .toggle_value(&mut x.selected, &x.lable)
                            .context_menu(|ui| {
                                //if ui.button("\u{2B06} Move Up").clicked(){}
                                //if ui.button("\u{2B07} Move Down").clicked(){}
                                if dev_index > 0 && ui.button("\u{1F5D1} Delete").clicked() {
                                    if self.sel_device_index == dev_index {
                                        self.sel_device_index -= 1;
                                    }
                                    ui.close_menu();
                                    retain = false;
                                }
                            })
                            .clicked()
                        {
                            self.sel_device_index = dev_index;
                            self.sel_query_index = usize::MAX;
                        }
                    })
                    .body(|ui| {
                        (self.sel_query_index, self.sel_device_index) = x.build_query_tree(
                            ui,
                            self.sel_query_index,
                            self.sel_device_index,
                            dev_index,
                        );
                    });
                    x.selected = self.sel_device_index == dev_index;
                    dev_index += 1;
                    retain
                });

                if ui.button("Add Device").clicked() {
                    self.devices.push(device::ModbusDevice::new());
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.devices[self.sel_device_index].draw_device_frame(ui, self.sel_query_index);
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {}

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(30)
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // NOTE: a bright gray makes the shadows of the windows look weird.
        // We use a bit of transparency so that if the user switches on the
        // `transparent()` option they get immediate results.
        egui::Color32::from_rgba_unmultiplied(12, 12, 12, 180).to_normalized_gamma_f32()

        // _visuals.window_fill() would also be a natural choice
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }
}
