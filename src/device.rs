#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Clone, Copy)]
pub enum ListItemAction {
    Nothing,
    Delete,
    MoveUp,
    MoveDown,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ModbusDevice {
    pub lable: String,
    pub unit_id: u8,
    pub ip: String,
    pub port: String,
    pub selected: bool,
    pub querys: Vec<crate::query::QueryWrapper>,
    pub notes: String,
}

impl Default for ModbusDevice {
    fn default() -> Self {
        Self {
            lable: "New Device".to_owned(),
            unit_id: 1,
            ip: Default::default(),
            port: Default::default(),
            selected: false,
            querys: vec![crate::query::QueryWrapper::new()],
            notes: "Add device notes here".to_string(),
        }
    }
}

impl ModbusDevice {
    pub fn new() -> Self {
        Self {
            lable: "New Device".to_owned(),
            unit_id: 1,
            ip: Default::default(),
            port: Default::default(),
            selected: false,
            querys: Default::default(),
            notes: "Add device notes here".to_string(),
        }
    }

    pub fn draw_device_frame(&mut self, ui: &mut egui::Ui, query_id: usize) {
        if query_id == usize::MAX {
            ui.separator();
            ui.horizontal(|ui| {
                ui.add_sized([100.0, 10.0], egui::Label::new("Device Lable:"));
                ui.add_sized(
                    [200.0, 10.0],
                    egui::TextEdit::singleline(&mut self.lable).hint_text("Input Lable"),
                );
            });

            ui.horizontal(|ui| {
                ui.add_sized([100.0, 10.0], egui::Label::new("Ip Adress:"));
                ui.add_sized(
                    [200.0, 10.0],
                    egui::TextEdit::singleline(&mut self.ip).hint_text("Input IP"),
                );
            });

            ui.horizontal(|ui| {
                ui.add_sized([100.0, 10.0], egui::Label::new("TCP/UDP Port:"));
                ui.add_sized(
                    [200.0, 10.0],
                    egui::TextEdit::singleline(&mut self.port).hint_text("Input Port"),
                );
            });

            ui.horizontal(|ui| {
                ui.add_sized([100.0, 10.0], egui::Label::new("Device ID:"));
                ui.add(
                    egui::DragValue::new(&mut self.unit_id)
                        .clamp_range(u8::MIN..=u8::MAX)
                        .speed(0.0),
                )
                .on_hover_cursor(egui::CursorIcon::Text);
            });

            ui.add_sized(
                ui.available_size(),
                egui::TextEdit::multiline(&mut self.notes),
            );
        } else {
            self.querys.get_mut(query_id).unwrap().draw_query_frame(ui);
        }
    }

    pub fn build_query_tree(
        &mut self,
        ui: &mut egui::Ui,
        index: usize,
        current_device: usize,
        this_device: usize,
    ) -> (usize, usize) {
        let mut quer_index: usize = 0;
        let mut final_index = 0;
        let mut ret: bool = false;
        let mut retain = true;
        self.querys.retain_mut(|x| {
            egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                ui.make_persistent_id(quer_index),
                true,
            )
            .show_header(ui, |ui| {
                let tv = ui
                    .toggle_value(&mut x.selected, &x.lable)
                    .context_menu(|ui| {
                        if quer_index > 0 && ui.button("\u{1F5D1} Delete").clicked() {
                            if index == quer_index {
                                final_index = quer_index - 1;
                            }
                            ui.close_menu();
                            retain = false
                        }
                    });

                if tv.clicked() {
                    ret = true;
                    final_index = quer_index;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    if ui.button("\u{23F5}").clicked() {
                        x.execute(&self.ip, &self.port)
                    }
                });

                for n in 0..x.watched_list.len() {
                    x.watched_list[n].update(&x.read_buffer);
                }

                if ret {
                    (final_index, this_device)
                } else if retain {
                    (index, current_device)
                } else {
                    (final_index, current_device)
                }
            })
            .body(|ui| {
                x.watched_list.retain_mut(|x| {
                    let mut retain = true;
                    ui.horizontal(|ui| {
                        if !x.locked {
                            ui.add_sized([150., 10.], egui::TextEdit::singleline(&mut x.label))
                                .context_menu(|ui| {
                                    if ui.button("\u{1F5D1} Delete").clicked() {
                                        ui.close_menu();
                                        retain = false;
                                    }
                                });

                            ui.add_sized(
                                [60., 10.],
                                egui::Label::new(x.resulting_value.to_string()),
                            )
                            .context_menu(|ui| {
                                if ui.button("\u{1F5D1} Delete").clicked() {
                                    ui.close_menu();
                                    retain = false;
                                }
                            });

                            ui.add_sized([50., 10.], egui::TextEdit::singleline(&mut x.suffix))
                                .context_menu(|ui| {
                                    if ui.button("\u{1F5D1} Delete").clicked() {
                                        ui.close_menu();
                                        retain = false;
                                    }
                                });
                            if ui.button("\u{1F512}").clicked() {
                                x.locked = true
                            }
                        } else {
                            ui.label(format!(
                                "{}:     {} {}",
                                x.label, x.resulting_value, x.suffix
                            ))
                            .on_hover_text(format!(
                                "Factor {}   Offsett {}",
                                x.factor, x.value_offsett
                            ))
                            .context_menu(|ui| {
                                if ui.button("\u{1F511} Unlock").clicked() {
                                    ui.close_menu();
                                    x.locked = false;
                                }
                            });
                        }
                    });

                    retain
                })
            });

            x.selected = index == quer_index && this_device == current_device;

            quer_index += 1;
            retain
        });
        if ui.button("Add Query").clicked() {
            self.querys.push(crate::query::QueryWrapper::new());
        }
        if ret {
            (final_index, this_device)
        } else if retain {
            (index, current_device)
        } else {
            (final_index, current_device)
        }
    }
}
