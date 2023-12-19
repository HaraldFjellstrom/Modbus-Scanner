#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct WatchedReg {
    pub lable: String,
    pub suffix: String,
    pub pos: usize,
    pub factor: f32,
    pub value_offsett: f32,
    pub resulting_value: f32,
    pub locked: bool,
    pub data_type: crate::query::DataView,
}

impl Default for WatchedReg {
    fn default() -> Self {
        Self {
            lable: "New Watched".to_owned(),
            suffix: "".to_owned(),
            pos: 0,
            factor: 1.,
            value_offsett: 0.,
            resulting_value: 0.,
            locked: false,
            data_type: crate::query::DataView::Unsigned16bit,
        }
    }
}

impl WatchedReg {
    pub fn new(
        label: String,
        suffix: String,
        pos: usize,
        factor: f32,
        value_offsett: f32,
        typ: crate::query::DataView,
    ) -> Self {
        Self {
            lable: label,
            suffix: suffix,
            pos: pos,
            factor: factor,
            value_offsett: value_offsett,
            resulting_value: 0.,
            locked: false,
            data_type: typ,
        }
    }

    pub fn update(&mut self, read_bytes: &Vec<u8>) -> () {
        match self.data_type {
            crate::query::DataView::Unsigned16bit => {
                self.resulting_value =
                    u16::from_be_bytes([read_bytes[self.pos], read_bytes[self.pos + 1]]) as f32
                        * self.factor
                        + self.value_offsett
            }
            crate::query::DataView::Signed16bit => {
                self.resulting_value =
                    i16::from_be_bytes([read_bytes[self.pos], read_bytes[self.pos + 1]]) as f32
                        * self.factor
                        + self.value_offsett
            }
            crate::query::DataView::Unsigned32bit => {
                self.resulting_value = u32::from_be_bytes([
                    read_bytes[self.pos + 2],
                    read_bytes[self.pos + 3],
                    read_bytes[self.pos],
                    read_bytes[self.pos + 1],
                ]) as f32
                    * self.factor
                    + self.value_offsett
            }
            crate::query::DataView::Signed32bit => {
                self.resulting_value = i32::from_be_bytes([
                    read_bytes[self.pos + 2],
                    read_bytes[self.pos + 3],
                    read_bytes[self.pos],
                    read_bytes[self.pos + 1],
                ]) as f32
                    * self.factor
                    + self.value_offsett
            }
            crate::query::DataView::Float32bit => {
                self.resulting_value = f32::from_be_bytes([
                    read_bytes[self.pos + 2],
                    read_bytes[self.pos + 3],
                    read_bytes[self.pos],
                    read_bytes[self.pos + 1],
                ]) * self.factor
                    + self.value_offsett
            }
        }
    }
}
