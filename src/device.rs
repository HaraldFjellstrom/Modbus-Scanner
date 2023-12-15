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
        }
    }

    pub fn draw_device_frame(&mut self, ui: &mut egui::Ui) {
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

        if ui.button("Query").clicked() {
            self.querrys[0].execute(&self);
        }
    }

    pub fn query(&self) {
        let timeout = Duration::from_secs(1);

        // open TCP connection
        let mut stream = TcpStream::connect(format!("{}:{}", self.ip, self.port)).unwrap();
        stream.set_read_timeout(Some(timeout)).unwrap();
        stream.set_write_timeout(Some(timeout)).unwrap();

        // create request object
        let mut mreq = ModbusRequest::new(1, ModbusProto::TcpUdp);
        mreq.tr_id = 2; // just for test, default tr_id is 1

        // set 2 coils
        let mut request = Vec::new();
        mreq.generate_set_coils_bulk(0, &[true, true], &mut request)
            .unwrap();

        // write request to stream
        stream.write(&request).unwrap();

        // read first 6 bytes of response frame
        let mut buf = [0u8; 6];
        stream.read_exact(&mut buf).unwrap();
        let mut response = Vec::new();
        response.extend_from_slice(&buf);
        let len = guess_response_frame_len(&buf, ModbusProto::TcpUdp).unwrap();
        // read rest of response frame
        if len > 6 {
            let mut rest = vec![0u8; (len - 6) as usize];
            stream.read_exact(&mut rest).unwrap();
            response.extend(rest);
        }
        // check if frame has no Modbus error inside
        mreq.parse_ok(&response).unwrap();

        // get coil values back
        mreq.generate_get_coils(0, 2, &mut request).unwrap();
        stream.write(&request).unwrap();
        let mut buf = [0u8; 6];
        stream.read_exact(&mut buf).unwrap();
        let mut response = Vec::new();
        response.extend_from_slice(&buf);
        let len = guess_response_frame_len(&buf, ModbusProto::TcpUdp).unwrap();
        if len > 6 {
            let mut rest = vec![0u8; (len - 6) as usize];
            stream.read_exact(&mut rest).unwrap();
            response.extend(rest);
        }
        let mut data = Vec::new();
        // check if frame has no Modbus error inside and parse response bools into data vec
        mreq.parse_bool(&response, &mut data).unwrap();
        for i in 0..data.len() {
            println!("{} {}", i, data[i]);
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
                    if ui.toggle_value(&mut x.selected, &x.lable).clicked() {
                        ret = true;
                        final_index = quer_index;
                    }
                    if quer_index > 0 {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                            if ui.button("-").clicked() {
                                if index == quer_index {
                                    final_index = quer_index - 1;
                                    if final_index <= 0 {
                                        final_index = 0
                                    }
                                }
                                retain = false
                            }
                        });
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
                })
                .body(|ui| {});
            if (index == quer_index && this_device == current_device) {
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
