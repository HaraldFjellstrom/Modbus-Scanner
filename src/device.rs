use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

//use egui;
use rmodbus::{client::ModbusRequest, guess_response_frame_len, ModbusProto};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ModbusDevice {
    // Example stuff:
    pub lable: String,
    pub unit_id: u8,
    pub ip: String,
    pub port: String,
    pub selected: bool,
    pub querrys: Vec<crate::query::QuerryWrapper>,
    pub notes: String,
}

impl Default for ModbusDevice {
    fn default() -> Self {
        Self {
            // Example stuff:
            lable: "New Device".to_owned(),
            unit_id: 1,
            ip: Default::default(),
            port: Default::default(),
            selected: false,
            querrys: vec![crate::query::QuerryWrapper::new()],
            notes: "Add device notes here".to_string(),
        }
    }
}

impl ModbusDevice {
    /// Called once before the first frame.
    pub fn new() -> Self {
        Self {
            lable: "New Device".to_owned(),
            unit_id: 1,
            ip: Default::default(),
            port: Default::default(),
            selected: false,
            querrys: Default::default(),
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

            //if ui.button("Try Connection").clicked() {
            //    match self.connect(){
            //        Ok(r) => self.connection_status = "Connection succesfully established".to_string(),
            //        Err(e) => self.connection_status = e.to_string(),

            //    }
            //}
            //ui.label(self.connection_status.to_owned());
            ui.add_sized(ui.available_size(), egui::TextEdit::multiline(&mut self.notes));
            
        } else {
            self.querrys
                .iter_mut()
                .nth(query_id)
                .unwrap()
                .draw_query_frame(ui);
        }
    }

    pub fn build_querry_tree(
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
        self.querrys.retain_mut(|x| {
            let id = ui.make_persistent_id(quer_index);
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
                .show_header(ui, |ui| {
                    let tv = ui
                        .toggle_value(&mut x.selected, &x.lable)
                        .context_menu(|ui| {
                            if quer_index > 0 {
                                if ui.button("Delete").clicked() {
                                    if index == quer_index {
                                        final_index = quer_index - 1;
                                        if final_index <= 0 {
                                            final_index = 0
                                        }
                                    }
                                    retain = false
                                }
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

                    if ret {
                        return (final_index, this_device);
                    } else {
                        if retain {
                            return (index, current_device);
                        } else {
                            return (final_index, current_device);
                        }
                    }
                })
                .body(|ui| {});

            if index == quer_index && this_device == current_device {
                x.selected = true
            } else {
                x.selected = false
            }
            quer_index += 1;
            return retain;
        });
        if ui.button("Add Querry").clicked() {
            self.querrys.push(crate::query::QuerryWrapper::new());
        }
        if ret {
            return (final_index, this_device);
        } else {
            if retain {
                return (index, current_device);
            } else {
                return (final_index, current_device);
            }
        }
    }
}
