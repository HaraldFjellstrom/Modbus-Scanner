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

    pub fn execute(&mut self,ip: &String, port: &String) -> () {
        let mut mreq = ModbusRequest::new(self.tr_id, ModbusProto::TcpUdp);
        let mut request = Vec::new();
        match &mut self.function_code {
            FC::ReadCoils => mreq.generate_get_coils(self.reg, self.count, &mut request).unwrap(),
            FC::ReadDiscreteInput => mreq.generate_get_discretes(self.reg, self.count, &mut request).unwrap(),
            FC::ReadHoldingRegisters => mreq.generate_get_holdings(self.reg, self.count, &mut request).unwrap(),
            FC::ReadInputRegisters => mreq.generate_get_inputs(self.reg, self.count, &mut request).unwrap(),
            FC::WriteCoil => mreq.generate_set_coil(self.reg, if self.write_buffer[0] == 0 {false}else{true}, &mut request).unwrap(),
            FC::WriteHoldingRegister => mreq.generate_set_holding(self.reg, LittleEndian::read_u16(&self.write_buffer[0..1]), &mut request).unwrap(),
            FC::WriteCoils => mreq.generate_set_coils_bulk(self.reg, self.write_buffer.chunks_exact(2).into_iter().map(|a| bool::from(u16::from_ne_bytes([a[0], a[1]])!=0)).collect::<Vec<_>>().as_slice(), &mut request).unwrap(),
            FC::WriteHoldingRegisters => mreq.generate_set_holdings_bulk(self.reg, self.write_buffer.chunks_exact(2).into_iter().map(|a| u16::from_ne_bytes([a[0], a[1]])).collect::<Vec<_>>().as_slice(), &mut request).unwrap(),
        };
        match QuerryWrapper::connect(ip, port){
            Err(e) => (),
            Ok(mut con) => {
                con.write(&request).unwrap();
            }
        }
    }

    pub fn connect(ip: &String, port: &String) -> Result<TcpStream, std::io::Error> {
        let timeout = Duration::from_secs(1);

        match TcpStream::connect(format!("{}:{}", ip, port)) {
            Ok(tcp_stream) => {
                tcp_stream.set_read_timeout(Some(timeout))?;
                tcp_stream.set_write_timeout(Some(timeout))?;
                return Ok(tcp_stream)
            },
            Err(e) => return Err(e),
        };
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

            ui.spacing_mut().slider_width = -5.0;
            ui.add_sized([80.0, 10.0], egui::Label::new("Offset:"));
            ui.add(egui::Slider::new(&mut self.reg, 0..=u16::MAX).handle_shape(egui::style::HandleShape::Rect{ aspect_ratio: -1.0 }));
            ui.add_sized([80.0, 10.0], egui::Label::new("Count:"));
            ui.add(egui::Slider::new(&mut self.count, 0..=124).handle_shape(egui::style::HandleShape::Rect{ aspect_ratio: -1.0 }));
        });
        ui.separator();
    }
}
