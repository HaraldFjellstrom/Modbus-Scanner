use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use byteorder::{ByteOrder, LittleEndian};
use rmodbus::{client::ModbusRequest, guess_response_frame_len, ModbusProto};

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
#[repr(u8)]
pub enum FC {
    ReadCoils = 1,
    ReadDiscreteInput = 2,
    ReadHoldingRegisters = 3,
    ReadInputRegisters = 4,
    WriteCoil = 5,
    WriteHoldingRegister = 6,
    WriteCoils = 15,
    WriteHoldingRegisters = 16,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Clone, Copy)]
pub enum DataView {
    Unsigned16bit,
    Signed16bit,
    Unsigned32bit,
    Signed32bit,
    Float32bit,
    Hexadecimal,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct QuerryWrapper {
    pub lable: String,
    pub reg: u16,
    pub count: u16,
    pub tr_id: u8,
    pub unit_id: u8,
    pub function_code: FC,
    pub read_buffer: Vec<u8>,
    pub write_buffer: Vec<u8>,
    pub selected: bool,
    pub response: String,
    pub data_veiw1: DataView,
    pub factor: f32,
    pub value_offsett: f32,
    pub watched_list: Vec<crate::watched::WatchedReg>,
}

impl Default for QuerryWrapper {
    fn default() -> Self {
        Self {
            // Example stuff:
            lable: "New Querry".to_owned(),
            reg: 0,
            count: 0,
            tr_id: 0,
            unit_id: 0,
            function_code: FC::ReadCoils,
            read_buffer: vec![0, 247],
            write_buffer: vec![0, 247],
            selected: false,
            response: "Not Executed".to_owned(),
            data_veiw1: DataView::Unsigned16bit,
            factor: 1.,
            value_offsett: 0.,
            watched_list: vec![],
        }
    }
}

impl QuerryWrapper {
    /// Called once before the first frame.
    pub fn new() -> Self {
        Self {
            lable: "New Querry".to_owned(),
            reg: 0,
            count: 1,
            tr_id: 1,
            unit_id: 1,
            function_code: FC::ReadCoils,
            read_buffer: vec![0, 247],
            write_buffer: vec![0, 247],
            selected: false,
            response: "Not Executed".to_owned(),
            data_veiw1: DataView::Unsigned16bit,
            factor: 1.,
            value_offsett: 0.,
            watched_list: vec![],
        }
    }

    pub fn execute(&mut self, ip: &String, port: &String) {
        let mut mreq = ModbusRequest::new(self.tr_id, ModbusProto::TcpUdp);
        let mut request = Vec::new();
        match &mut self.function_code {
            FC::ReadCoils => match mreq.generate_get_coils(self.reg, self.count * 16, &mut request)
            {
                Ok(_) => (),
                Err(e) => self.response = e.to_string(),
            },
            FC::ReadDiscreteInput => {
                match mreq.generate_get_discretes(self.reg, self.count * 16, &mut request) {
                    Ok(_) => (),
                    Err(e) => self.response = e.to_string(),
                }
            }
            FC::ReadHoldingRegisters => {
                match mreq.generate_get_holdings(self.reg, self.count, &mut request) {
                    Ok(_) => (),
                    Err(e) => self.response = e.to_string(),
                }
            }
            FC::ReadInputRegisters => {
                match mreq.generate_get_inputs(self.reg, self.count, &mut request) {
                    Ok(_) => (),
                    Err(e) => self.response = e.to_string(),
                }
            }
            FC::WriteCoil => {
                match mreq.generate_set_coil(self.reg, self.write_buffer[0] != 0, &mut request) {
                    Ok(_) => (),
                    Err(e) => self.response = e.to_string(),
                }
            }
            FC::WriteHoldingRegister => {
                match mreq.generate_set_holding(
                    self.reg,
                    LittleEndian::read_u16(&self.write_buffer),
                    &mut request,
                ) {
                    Ok(_) => (),
                    Err(e) => {
                        self.response = e.to_string();
                        return;
                    }
                }
            }
            FC::WriteCoils => match mreq.generate_set_coils_bulk(
                self.reg,
                self.write_buffer
                    .chunks_exact(2)
                    .map(|a| u16::from_ne_bytes([a[0], a[1]]) != 0)
                    .collect::<Vec<_>>()
                    .as_slice(),
                &mut request,
            ) {
                Ok(_) => (),
                Err(e) => self.response = e.to_string(),
            },
            FC::WriteHoldingRegisters => {
                match mreq.generate_set_holdings_bulk(
                    self.reg,
                    self.write_buffer
                        .chunks_exact(2)
                        .map(|a| u16::from_ne_bytes([a[0], a[1]]))
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &mut request,
                ) {
                    Ok(_) => (),
                    Err(e) => {
                        self.response = e.to_string();
                        return;
                    }
                }
            }
        }

        match QuerryWrapper::connect(ip, port) {
            Err(e) => self.response = e.to_string(),
            Ok(mut con) => {
                match con.write(&request) {
                    Ok(_) => (),
                    Err(e) => self.response = e.to_string(),
                }

                // read first 6 bytes of response frame
                let mut buf = [0u8; 6];
                match con.read_exact(&mut buf) {
                    Ok(_) => (),
                    Err(e) => self.response = e.to_string(),
                }

                let mut response = Vec::new();
                response.extend_from_slice(&buf);
                match guess_response_frame_len(&buf, ModbusProto::TcpUdp) {
                    Ok(len) => {
                        if len > 6 {
                            let mut rest = vec![0u8; (len - 6) as usize];
                            con.read_exact(&mut rest).unwrap();
                            response.extend(rest);
                        }
                    }
                    Err(e) => self.response = e.to_string(),
                }

                // check if frame has no Modbus error inside
                match mreq.parse_ok(&response) {
                    Err(e) => self.response = e.to_string(),
                    Ok(_ok) => match self.function_code {
                        FC::ReadCoils | FC::ReadDiscreteInput => {
                            let mut temphold: Vec<bool> = vec![];
                            match mreq.parse_bool(&response, &mut temphold) {
                                Ok(_) => {
                                    self.read_buffer = unsafe { std::mem::transmute(temphold) }
                                }
                                Err(e) => self.response = e.to_string(),
                            }
                        }
                        FC::ReadHoldingRegisters | FC::ReadInputRegisters => {
                            response.drain(0..9);
                            self.read_buffer = response;
                            self.response = "Read successful".to_owned()
                        }
                        FC::WriteCoil
                        | FC::WriteCoils
                        | FC::WriteHoldingRegister
                        | FC::WriteHoldingRegisters => {
                            self.response = "Write successful".to_owned()
                        }
                    },
                }
            }
        }
    }

    pub fn connect(ip: &String, port: &String) -> Result<TcpStream, std::io::Error> {
        let timeout = Duration::from_secs(1);

        match TcpStream::connect(format!("{}:{}", ip, port)) {
            Ok(tcp_stream) => {
                tcp_stream.set_read_timeout(Some(timeout))?;
                tcp_stream.set_write_timeout(Some(timeout))?;
                Ok(tcp_stream)
            }
            Err(e) => Err(e),
        }
    }

    pub fn draw_query_frame(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_sized([80.0, 10.0], egui::Label::new("Query Lable:"));
            ui.add_sized(
                [100.0, 10.0],
                egui::TextEdit::singleline(&mut self.lable).hint_text("Input Lable"),
            );

            ui.add_sized([100.0, 10.0], egui::Label::new("Function Code:"));
            egui::ComboBox::from_id_source(self.lable.to_owned())
                .selected_text(format!("{:?}", self.function_code))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.function_code, FC::ReadCoils, "FC1 Read Coils");
                    ui.selectable_value(
                        &mut self.function_code,
                        FC::ReadDiscreteInput,
                        "FC2 Read Discrete Input",
                    );
                    ui.selectable_value(
                        &mut self.function_code,
                        FC::ReadHoldingRegisters,
                        "FC3 Read Holding Registers",
                    );
                    ui.selectable_value(
                        &mut self.function_code,
                        FC::ReadInputRegisters,
                        "FC4 Read Input Registers",
                    );
                    ui.selectable_value(&mut self.function_code, FC::WriteCoil, "FC5 Write Coil");
                    ui.selectable_value(
                        &mut self.function_code,
                        FC::WriteHoldingRegister,
                        "FC6 Write Holding Register",
                    );
                    ui.selectable_value(
                        &mut self.function_code,
                        FC::WriteCoils,
                        "FC15 Write Coils",
                    );
                    ui.selectable_value(
                        &mut self.function_code,
                        FC::WriteHoldingRegisters,
                        "FC16 Write Holding Registers",
                    );
                });
        });
        ui.horizontal(|ui| {
            ui.add_sized([80.0, 10.0], egui::Label::new("Offset:"));
            ui.add(
                egui::DragValue::new(&mut self.reg)
                    .clamp_range(0..=u16::MAX)
                    .speed(0.0),
            )
            .on_hover_cursor(egui::CursorIcon::Text);
            ui.add_sized([80.0, 10.0], egui::Label::new("Count:"));
            if ui
                .add(
                    egui::DragValue::new(&mut self.count)
                        .clamp_range(1..=122)
                        .speed(0.0),
                )
                .on_hover_cursor(egui::CursorIcon::Text)
                .lost_focus()
            {
                self.write_buffer = vec![0; (self.count * 2) as usize]
            }
            ui.label(self.response.as_str());
        });
        ui.separator();

        ui.horizontal(|ui| {
            // Radio button to set big or little endian ??
            ui.label("Data as ");
            egui::ComboBox::from_id_source("Dataview 1")
                .selected_text(format!("{:?}", self.data_veiw1))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.data_veiw1,
                        DataView::Unsigned16bit,
                        "16-bit Unsigned Integer",
                    );
                    ui.selectable_value(
                        &mut self.data_veiw1,
                        DataView::Signed16bit,
                        "16-bit Signed Integer",
                    );
                    ui.selectable_value(
                        &mut self.data_veiw1,
                        DataView::Unsigned32bit,
                        "32-bit Unsigned Integer",
                    );
                    ui.selectable_value(
                        &mut self.data_veiw1,
                        DataView::Signed32bit,
                        "32-bit Signed Integer",
                    );
                    ui.selectable_value(&mut self.data_veiw1, DataView::Float32bit, "32-bit Float");
                    ui.selectable_value(&mut self.data_veiw1, DataView::Hexadecimal, "Hexadeciaml");
                });

            match self.function_code {
                FC::ReadCoils
                | FC::ReadDiscreteInput
                | FC::ReadHoldingRegisters
                | FC::ReadInputRegisters => {
                    if self.data_veiw1 != DataView::Hexadecimal {
                        ui.label("Factor:");
                        ui.add(
                            egui::DragValue::new(&mut self.factor)
                                .clamp_range(0.0..=f32::MAX)
                                .speed(0.0)
                                .custom_formatter(|n, _| format!("{:}", n)),
                        )
                        .on_hover_cursor(egui::CursorIcon::Text);
                        ui.label("Value Offsett:");
                        ui.add(
                            egui::DragValue::new(&mut self.value_offsett)
                                .clamp_range(f32::MIN..=f32::MAX)
                                .speed(0.0)
                                .custom_formatter(|n, _| format!("{:}", n)),
                        )
                        .on_hover_cursor(egui::CursorIcon::Text);
                    }
                }
                FC::WriteCoil
                | FC::WriteCoils
                | FC::WriteHoldingRegister
                | FC::WriteHoldingRegisters => (),
            }
        });

        ui.horizontal(|ui| match self.data_veiw1 {
            DataView::Unsigned16bit => match self.function_code {
                FC::ReadCoils
                | FC::ReadDiscreteInput
                | FC::ReadHoldingRegisters
                | FC::ReadInputRegisters => {
                    self.draw_read_data_grid_u16(ui);
                }
                FC::WriteCoil
                | FC::WriteCoils
                | FC::WriteHoldingRegister
                | FC::WriteHoldingRegisters => {
                    self.draw_write_data_grid_u16(ui);
                }
            },
            DataView::Signed16bit => match self.function_code {
                FC::ReadCoils
                | FC::ReadDiscreteInput
                | FC::ReadHoldingRegisters
                | FC::ReadInputRegisters => {
                    self.draw_read_data_grid_i16(ui);
                }
                FC::WriteCoil
                | FC::WriteCoils
                | FC::WriteHoldingRegister
                | FC::WriteHoldingRegisters => {
                    self.draw_write_data_grid_i16(ui);
                }
            },
            DataView::Unsigned32bit => match self.function_code {
                FC::ReadCoils
                | FC::ReadDiscreteInput
                | FC::ReadHoldingRegisters
                | FC::ReadInputRegisters => {
                    self.draw_read_data_grid_u32(ui);
                }
                FC::WriteCoil
                | FC::WriteCoils
                | FC::WriteHoldingRegister
                | FC::WriteHoldingRegisters => {
                    self.draw_write_data_grid_u32(ui);
                }
            },
            DataView::Signed32bit => match self.function_code {
                FC::ReadCoils
                | FC::ReadDiscreteInput
                | FC::ReadHoldingRegisters
                | FC::ReadInputRegisters => {
                    self.draw_read_data_grid_i32(ui);
                }
                FC::WriteCoil
                | FC::WriteCoils
                | FC::WriteHoldingRegister
                | FC::WriteHoldingRegisters => {
                    self.draw_write_data_grid_i32(ui);
                }
            },
            DataView::Float32bit => match self.function_code {
                FC::ReadCoils
                | FC::ReadDiscreteInput
                | FC::ReadHoldingRegisters
                | FC::ReadInputRegisters => {
                    self.draw_read_data_grid_f32(ui);
                }
                FC::WriteCoil
                | FC::WriteCoils
                | FC::WriteHoldingRegister
                | FC::WriteHoldingRegisters => {
                    self.draw_write_data_grid_f32(ui);
                }
            },
            DataView::Hexadecimal => match self.function_code {
                FC::ReadCoils
                | FC::ReadDiscreteInput
                | FC::ReadHoldingRegisters
                | FC::ReadInputRegisters => {
                    self.draw_read_data_grid_hex(ui);
                }
                FC::WriteCoil
                | FC::WriteCoils
                | FC::WriteHoldingRegister
                | FC::WriteHoldingRegisters => {
                    self.draw_write_data_grid_hex(ui);
                }
            },
        });

        ui.separator();
    }
    ///Start to implementing generic function for drawing different datatypes, second guessed myself halfway
    ///because it might bee a  good idea to have the controle over each type for specific styling and parcing etc...
    //
    //pub fn draw_data_grid(&mut self, ui: &mut egui::Ui) {
    //    egui::Grid::new("some_unique_id")
    //        .striped(true)
    //        .show(ui, |ui| match self.function_code {
    //            FC::ReadCoils
    //            | FC::ReadDiscreteInput
    //            | FC::ReadHoldingRegisters
    //            | FC::ReadInputRegisters => {
    //                for i in (0..self.read_buffer.len()).step_by(1) {
    //                    if (i % 16) == 0 {
    //                        ui.end_row();
    //                    }

    //                    ui.add_sized(
    //                        [40.0, 10.0],
    //                        egui::Label::new(format!("{:X}", self.read_buffer[i]))
    //                            .sense(egui::Sense::click()),
    //                    )
    //                    .on_hover_text(format!("Byte {}", i));
    //                }
    //            }
    //            FC::WriteCoil
    //            | FC::WriteCoils
    //            | FC::WriteHoldingRegister
    //            | FC::WriteHoldingRegisters => {
    //                for i in (0..self.write_buffer.len()).step_by(1) {
    //                    if (i % 16) == 0 {
    //                        ui.end_row();
    //                    }

    //                    ui.add_sized(
    //                        [40.0, 10.0],
    //                        egui::Label::new(format!("{:X}", self.read_buffer[i]))
    //                            .sense(egui::Sense::click()),
    //                    )
    //                    .on_hover_text(format!("Byte {}", i));
    //                }
    //            }
    //        });
    //}

    //pub fn draw_read_gridcell() {}
    //pub fn draw_write_gridcell() {}

    pub fn draw_read_data_grid_hex(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("some_unique_id")
            .striped(true)
            .show(ui, |ui| {
                let mut reg_nr: u16 = self.reg + 1;
                for i in (0..self.read_buffer.len()).step_by(2) {
                    if (i % 16) == 0 {
                        ui.end_row();
                    }

                    ui.add_sized(
                        [60.0, 20.0],
                        egui::Label::new(format!(
                            "{:04X}",
                            u16::from_be_bytes([self.read_buffer[i], self.read_buffer[i + 1]])
                        ))
                        .sense(egui::Sense::click()),
                    )
                    .on_hover_text(format!("Reg {}", reg_nr))
                    .context_menu(|ui| {
                        if ui.button("\u{1F441} Add to watched").clicked() {
                            self.watched_list.push(crate::watched::WatchedReg::new(
                                format!(
                                    "Reg {} as U16 * {} + {}",
                                    reg_nr, self.factor, self.value_offsett
                                )
                                .to_owned(),
                                "".to_owned(),
                                i,
                                self.factor,
                                self.value_offsett,
                                self.data_veiw1,
                            ));
                        }
                    });
                    reg_nr += 1;
                }
            });
    }

    pub fn draw_read_data_grid_u16(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("u16_grid_r")
            .striped(true)
            .max_col_width(60.)
            .min_col_width(60.)
            .show(ui, |ui| {
                let mut reg_nr: u16 = self.reg + 1;
                for i in (0..self.read_buffer.len()).step_by(2) {
                    if (i % 16) == 0 {
                        ui.end_row();
                    }
                    ui.add_sized(
                        [60.0, 20.0],
                        egui::Label::new(
                            (u16::from_be_bytes([self.read_buffer[i], self.read_buffer[i + 1]])
                                as f32
                                * self.factor
                                + self.value_offsett)
                                .to_string(),
                        )
                        .sense(egui::Sense::click()),
                    )
                    .on_hover_text(format!("Reg {}", reg_nr))
                    .context_menu(|ui| {
                        if ui.button("\u{1F441} Add to watched").clicked() {
                            self.watched_list.push(crate::watched::WatchedReg::new(
                                format!(
                                    "Reg {} as U16 * {} + {}",
                                    reg_nr, self.factor, self.value_offsett
                                )
                                .to_owned(),
                                "".to_owned(),
                                i,
                                self.factor,
                                self.value_offsett,
                                self.data_veiw1,
                            ));
                        }
                    });
                    reg_nr += 1;
                }
            });
    }

    pub fn draw_read_data_grid_i16(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("i16_grid_r")
            .striped(true)
            .max_col_width(60.)
            .min_col_width(60.)
            .show(ui, |ui| {
                let mut reg_nr: u16 = self.reg + 1;
                for i in (0..self.read_buffer.len()).step_by(2) {
                    if (i % 16) == 0 {
                        ui.end_row();
                    }
                    ui.add_sized(
                        [60.0, 20.0],
                        egui::Label::new(
                            (i16::from_be_bytes([self.read_buffer[i], self.read_buffer[i + 1]])
                                as f32
                                * self.factor
                                + self.value_offsett)
                                .to_string(),
                        )
                        .sense(egui::Sense::click()),
                    )
                    .on_hover_text(format!("Reg {}", reg_nr))
                    .context_menu(|ui| {
                        if ui.button("Watch").clicked() {
                            self.watched_list.push(crate::watched::WatchedReg::new(
                                format!(
                                    "Reg {} as U16 * {} + {}",
                                    reg_nr, self.factor, self.value_offsett
                                )
                                .to_owned(),
                                "".to_owned(),
                                i,
                                self.factor,
                                self.value_offsett,
                                self.data_veiw1,
                            ));
                        }
                    });
                    reg_nr += 1;
                }
            });
    }

    pub fn draw_read_data_grid_u32(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("u32_grid_r")
            .striped(true)
            .max_col_width(60.)
            .min_col_width(60.)
            .show(ui, |ui| {
                let mut reg_nr: u16 = self.reg + 1;
                for i in (0..self.read_buffer.len() - 2).step_by(4) {
                    if (i % 32) == 0 {
                        ui.end_row();
                    }
                    ui.add_sized(
                        [60.0, 20.0],
                        egui::Label::new(
                            (u32::from_be_bytes([
                                self.read_buffer[i + 2],
                                self.read_buffer[i + 3],
                                self.read_buffer[i],
                                self.read_buffer[i + 1],
                            ]) as f32
                                * self.factor
                                + self.value_offsett)
                                .to_string(),
                        )
                        .sense(egui::Sense::click()),
                    )
                    .on_hover_text(format!("Reg {}", reg_nr))
                    .context_menu(|ui| {
                        if ui.button("Watch").clicked() {
                            self.watched_list.push(crate::watched::WatchedReg::new(
                                format!(
                                    "Reg {} as U16 * {} + {}",
                                    reg_nr, self.factor, self.value_offsett
                                )
                                .to_owned(),
                                "".to_owned(),
                                i,
                                self.factor,
                                self.value_offsett,
                                self.data_veiw1,
                            ));
                        }
                    });
                    reg_nr += 2;
                }
            });
    }

    pub fn draw_read_data_grid_i32(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("i32_grid_r")
            .striped(true)
            .max_col_width(60.)
            .min_col_width(60.)
            .show(ui, |ui| {
                let mut reg_nr: u16 = self.reg + 1;
                for i in (0..self.read_buffer.len() - 2).step_by(4) {
                    if (i % 32) == 0 {
                        ui.end_row();
                    }
                    ui.add_sized(
                        [60.0, 20.0],
                        egui::Label::new(
                            (i32::from_be_bytes([
                                self.read_buffer[i + 2],
                                self.read_buffer[i + 3],
                                self.read_buffer[i],
                                self.read_buffer[i + 1],
                            ]) as f32
                                * self.factor
                                + self.value_offsett)
                                .to_string(),
                        )
                        .sense(egui::Sense::click()),
                    )
                    .on_hover_text(format!("Reg {}", reg_nr))
                    .context_menu(|ui| {
                        if ui.button("Watch").clicked() {
                            self.watched_list.push(crate::watched::WatchedReg::new(
                                format!(
                                    "Reg {} as U16 * {} + {}",
                                    reg_nr, self.factor, self.value_offsett
                                )
                                .to_owned(),
                                "".to_owned(),
                                i,
                                self.factor,
                                self.value_offsett,
                                self.data_veiw1,
                            ));
                        }
                    });
                    reg_nr += 2;
                }
            });
    }

    pub fn draw_read_data_grid_f32(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("f32_grid_r")
            .striped(true)
            .max_col_width(60.)
            .min_col_width(60.)
            .show(ui, |ui| {
                let mut reg_nr: u16 = self.reg + 1;
                for i in (0..self.read_buffer.len() - 2).step_by(4) {
                    if (i % 32) == 0 {
                        ui.end_row();
                    }
                    ui.add_sized(
                        [60.0, 20.0],
                        egui::Label::new(
                            (f32::from_be_bytes([
                                self.read_buffer[i + 2],
                                self.read_buffer[i + 3],
                                self.read_buffer[i],
                                self.read_buffer[i + 1],
                            ]) * self.factor
                                + self.value_offsett)
                                .to_string(),
                        )
                        .sense(egui::Sense::click()),
                    )
                    .on_hover_text(format!("Reg {}", reg_nr))
                    .context_menu(|ui| {
                        if ui.button("Watch").clicked() {
                            self.watched_list.push(crate::watched::WatchedReg::new(
                                format!(
                                    "Reg {} as U16 * {} + {}",
                                    reg_nr, self.factor, self.value_offsett
                                )
                                .to_owned(),
                                "".to_owned(),
                                i,
                                self.factor,
                                self.value_offsett,
                                self.data_veiw1,
                            ));
                        }
                    });
                    reg_nr += 2;
                }
            });
    }

    pub fn draw_write_data_grid_u16(&self, ui: &mut egui::Ui) {
        egui::Grid::new("u16_grid_w")
            .striped(true)
            .max_col_width(60.)
            .min_col_width(60.)
            .show(ui, |ui| {
                let mut reg_nr: u16 = self.reg + 1;
                for i in (0..self.write_buffer.len()).step_by(2) {
                    if (i % 16) == 0 {
                        ui.end_row();
                    }
                    ui.add_sized(
                        [60., 20.],
                        egui::DragValue::new(unsafe {
                            &mut std::slice::from_raw_parts_mut(
                                self.write_buffer[i..i + 1].as_ptr() as *mut u16,
                                2,
                            )[0]
                        })
                        .clamp_range(u16::MIN..=u16::MAX)
                        .speed(0.0),
                    )
                    .on_hover_cursor(egui::CursorIcon::Text)
                    .on_hover_cursor(egui::CursorIcon::Text)
                    .on_hover_text(format!("Reg {}", reg_nr))
                    .context_menu(|ui| if ui.button("Watch").clicked() {});
                    reg_nr += 1;
                }
            });
    }

    pub fn draw_write_data_grid_i16(&self, ui: &mut egui::Ui) {
        egui::Grid::new("i16_grid_w")
            .striped(true)
            .max_col_width(60.)
            .min_col_width(60.)
            .show(ui, |ui| {
                let mut reg_nr: u16 = self.reg + 1;
                for i in (0..self.write_buffer.len()).step_by(2) {
                    if (i % 16) == 0 {
                        ui.end_row();
                    }
                    ui.add_sized(
                        [60., 20.],
                        egui::DragValue::new(unsafe {
                            &mut std::slice::from_raw_parts_mut(
                                self.write_buffer[i..i + 1].as_ptr() as *mut i16,
                                2,
                            )[0]
                        })
                        .clamp_range(i16::MIN..=i16::MAX)
                        .speed(0.0),
                    )
                    .on_hover_cursor(egui::CursorIcon::Text)
                    .on_hover_text(format!("Reg {}", reg_nr))
                    .context_menu(|ui| if ui.button("Watch").clicked() {});
                    reg_nr += 1;
                }
            });
    }

    pub fn draw_write_data_grid_u32(&self, ui: &mut egui::Ui) {
        egui::Grid::new("u32_grid_w")
            .striped(true)
            .max_col_width(60.)
            .min_col_width(60.)
            .show(ui, |ui| {
                let mut reg_nr: u16 = self.reg + 1;
                for i in (0..self.write_buffer.len()).step_by(4) {
                    if (i % 32) == 0 {
                        ui.end_row();
                    }
                    ui.add_sized(
                        [60., 20.],
                        egui::DragValue::new(unsafe {
                            &mut std::slice::from_raw_parts_mut(
                                self.write_buffer[i..i + 3].as_ptr() as *mut u32,
                                2,
                            )[0]
                        })
                        .clamp_range(u32::MIN..=u32::MAX)
                        .speed(0.0),
                    )
                    .on_hover_cursor(egui::CursorIcon::Text)
                    .on_hover_text(format!("Reg {}", reg_nr))
                    .context_menu(|ui| if ui.button("Watch").clicked() {});
                    reg_nr += 2;
                }
            });
    }

    pub fn draw_write_data_grid_i32(&self, ui: &mut egui::Ui) {
        egui::Grid::new("i32_grid_w")
            .striped(true)
            .max_col_width(60.)
            .min_col_width(60.)
            .show(ui, |ui| {
                let mut reg_nr: u16 = self.reg + 1;
                for i in (0..self.write_buffer.len()).step_by(4) {
                    if (i % 32) == 0 {
                        ui.end_row();
                    }
                    ui.add_sized(
                        [60., 20.],
                        egui::DragValue::new(unsafe {
                            &mut std::slice::from_raw_parts_mut(
                                self.write_buffer[i..i + 3].as_ptr() as *mut i32,
                                2,
                            )[0]
                        })
                        .clamp_range(i32::MIN..=i32::MAX)
                        .speed(0.0),
                    )
                    .on_hover_cursor(egui::CursorIcon::Text)
                    .on_hover_text(format!("Reg {}", reg_nr))
                    .context_menu(|ui| if ui.button("Watch").clicked() {});
                    reg_nr += 2;
                }
            });
    }

    pub fn draw_write_data_grid_f32(&self, ui: &mut egui::Ui) {
        egui::Grid::new("f32_grid_w")
            .striped(true)
            .max_col_width(60.)
            .min_col_width(60.)
            .show(ui, |ui| {
                let mut reg_nr: u16 = self.reg + 1;
                for i in (0..self.write_buffer.len()).step_by(4) {
                    if (i % 32) == 0 {
                        ui.end_row();
                    }
                    ui.add_sized(
                        [60., 20.],
                        egui::DragValue::new(unsafe {
                            &mut std::slice::from_raw_parts_mut(
                                self.write_buffer[i..i + 3].as_ptr() as *mut f32,
                                2,
                            )[0]
                        })
                        .clamp_range(f32::MIN..=f32::MAX)
                        .speed(0.0)
                        .custom_formatter(|n, _| format!("{}", n)),
                    )
                    .on_hover_cursor(egui::CursorIcon::Text)
                    .on_hover_text(format!("Reg {}", reg_nr))
                    .context_menu(|ui| if ui.button("Watch").clicked() {});
                    reg_nr += 2;
                }
            });
    }

    pub fn draw_write_data_grid_hex(&self, ui: &mut egui::Ui) {
        egui::Grid::new("hex_grid_w")
            .striped(true)
            .max_col_width(60.)
            .min_col_width(60.)
            .show(ui, |ui| {
                let mut reg_nr: u16 = self.reg + 1;
                for i in (0..self.write_buffer.len()).step_by(2) {
                    if (i % 16) == 0 {
                        ui.end_row();
                    }
                    ui.add_sized(
                        [60., 20.],
                        egui::DragValue::new(unsafe {
                            &mut std::slice::from_raw_parts_mut(
                                self.write_buffer[i..i + 1].as_ptr() as *mut u16,
                                2,
                            )[0]
                        })
                        .clamp_range(u16::MIN..=u16::MAX)
                        .hexadecimal(4, false, true)
                        .speed(0.0),
                    )
                    .on_hover_cursor(egui::CursorIcon::Text)
                    .on_hover_text(format!("Reg {}", reg_nr))
                    .context_menu(|ui| if ui.button("Watch").clicked() {});
                    reg_nr += 1;
                }
            });
    }
}
