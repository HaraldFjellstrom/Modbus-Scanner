use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

//use egui;
use crate::device;
use byteorder::{LittleEndian, ByteOrder};
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

#[repr(C)]
union reg {
    U8 : [u8; 248],
    U16 : [u16; 124],
    U32 : [u32; 62],
    U64 : [u64; 31],
    I8 : [i8; 248],
    I16 : [i16; 124],
    I32 : [i32; 62],
    I64 : [i64; 31],
    F32 : [f32; 62],
    F64 : [f64; 31],
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
            read_buffer: vec![],
            write_buffer: vec![],
            selected: false,
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
            read_buffer: vec![],
            write_buffer: vec![],
            selected: false,
        }
    }

    //pub fn get_request(&mut self) -> &[u8] {
    //    let mut mreq = ModbusRequest::new(self.tr_id, ModbusProto::TcpUdp);
    //    let mut request = Vec::new();
    //    match &mut self.function_code {
    //        FC::ReadCoils => mreq.generate_get_coils(self.reg, self.count, &mut self.read_buffer).unwrap(),
    //        FC::ReadDiscreteInput => mreq.generate_get_discretes(self.reg, self.count, &mut self.read_buffer).unwrap(),
    //        FC::ReadHoldingRegisters => mreq.generate_get_holdings(self.reg, self.count, &mut self.read_buffer).unwrap(),
    //        FC::ReadInputRegisters => mreq.generate_get_inputs(self.reg, self.count, &mut self.read_buffer).unwrap(),
    //        FC::WriteCoil => mreq.generate_get_inputs(self.reg, let w: [u8; self.count] = self.write_buffert[0..self.count].try_into().unwrap(), &mut self.write_buffer).unwrap(),
    //        FC::WriteHoldingRegister => mreq.generate_get_inputs(self.reg, self.count, &mut self.write_buffer).unwrap(),
    //        FC::WriteCoils => mreq.generate_get_inputs(self.reg, self.count, &mut self.write_buffer).unwrap(),
    //        FC::WriteHoldingRegisters => mreq.generate_get_inputs(self.reg, self.count, &mut self.write_buffer).unwrap(),
    //    };
    //    return request
    //}

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

            ui.add_sized([80.0, 10.0], egui::Label::new("Offset:"));
            ui.add_sized(
                [20.0, 10.0],
                egui::TextEdit::singleline(&mut self.reg.to_string()).hint_text("Input Port"),
            );

            ui.add_sized([80.0, 10.0], egui::Label::new("Count:"));
            ui.add_sized(
                [20.0, 10.0],
                egui::TextEdit::singleline(&mut self.count.to_string()).hint_text("Input Port"),
            );
        });
        ui.separator();
    }
}
