use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

//use egui;
use rmodbus::{client::ModbusRequest, guess_response_frame_len, ModbusProto};

use egui::CollapsingHeader;

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

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
}

//static NEXT_ID: AtomicU64 = AtomicU64::new(1);

impl Default for ModbusApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
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
            //if self.sel_querry_index == usize::MAX {
            self.devices[self.sel_device_index].draw_device_frame(ui, self.sel_querry_index);
            //} else {
            //    self.devices[self.sel_device_index].querrys[self.sel_device_index].draw_query_frame(ui);
            //}
        });

        //egui::CentralPanel::default().show(ctx, |ui| {
        //    // The central panel the region left after adding TopPanel's and SidePanel's
        //    ui.heading("eframe template");

        //    ui.horizontal(|ui| {
        //        ui.label("Write something: ");
        //        ui.text_edit_singleline(&mut self.label);
        //    });

        //    ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
        //    if ui.button("Increment").clicked() {
        //        self.value += 1.0;
        //    }

        //    ui.separator();
        //    ui.horizontal(|ui|{
        //        ui.label("Device Label: ");
        //        ui.add(egui::TextEdit::singleline(&mut self.devices[0].lable).hint_text("Input Lable"));
        //    });

        //    ui.horizontal(|ui|{
        //        ui.label("IP: ");
        //        ui.add(egui::TextEdit::singleline(&mut self.devices[0].ip).hint_text("Input IP"));

        //    });

        //    ui.horizontal(|ui|{
        //        ui.label("Port: ");
        //        ui.add(egui::TextEdit::singleline(&mut self.devices[0].port).hint_text("Input Port"));
        //    });

        //    if ui.button("Query").clicked() {
        //       self.devices[0].query();
        //    }

        //    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        //        powered_by_egui_and_eframe(ui);
        //        egui::warn_if_debug_build(ui);
        //    });
        //});
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}

pub fn query(device: &crate::device::ModbusDevice, query: &mut crate::query::QuerryWrapper) {
    let timeout = Duration::from_secs(1);

    // open TCP connection
    let mut stream = TcpStream::connect(format!("{}:{}", device.ip, device.port)).unwrap();
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
