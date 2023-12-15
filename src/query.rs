use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

//use egui;
use crate::device;
use rmodbus::{client::ModbusRequest, guess_response_frame_len, ModbusProto};

#[derive(serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum FC {
    ReadCoils = 1,
    ReadDiscreteInput = 2,
    ReadMultipleHoldingRegisters = 3,
    ReadInputRegisters = 4,
    WriteSingleCoil = 5,
    WriteSingleHoldingRegister = 6,
    WriteMultipleCoils = 15,
    WriteMultipleHoldingRegisters = 16,
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
    pub read_buffer: Vec<u16>,
    pub write_buffer: Vec<u16>,
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

    pub fn execute(&self, device: &device::ModbusDevice) {
        let timeout = Duration::from_secs(1);

        // open TCP connection
        let mut stream = TcpStream::connect(format!("{}:{}", device.ip, device.port)).unwrap();
        stream.set_read_timeout(Some(timeout)).unwrap();
        stream.set_write_timeout(Some(timeout)).unwrap();

        // create request object
        let mut mreq = ModbusRequest::new(self.tr_id, ModbusProto::TcpUdp);
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
}
