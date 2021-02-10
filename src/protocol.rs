use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientConfiguration {
    pub stylus_support: bool,
    pub faster_capture: bool,
    pub capturable_id: usize,
    pub capture_cursor: bool,
    pub max_width: usize,
    pub max_height: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageInbound {
    PointerEvent(PointerEvent),
    // request a video frame from the server
    // like this the client can partially control the framerate by sending requests at some given
    // rate. However, the server may drop a request if encoding is too slow.
    TryGetFrame,
    GetCapturableList,
    Config(ClientConfiguration),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageOutbound {
    CapturableList(Vec<String>),
    NewVideo,
    ConfigOk,
    ConfigError(String),
    Error(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PointerType {
    #[serde(rename = "")]
    Unknown,
    #[serde(rename = "mouse")]
    Mouse,
    #[serde(rename = "pen")]
    Pen,
    #[serde(rename = "touch")]
    Touch,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PointerEventType {
    #[serde(rename = "pointerdown")]
    DOWN,
    #[serde(rename = "pointerup")]
    UP,
    #[serde(rename = "pointercancel")]
    CANCEL,
    #[serde(rename = "pointermove")]
    MOVE,
}

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct Button: u8 {
        const NONE = 0b0000_0000;
        const PRIMARY = 0b0000_0001;
        const SECONDARY = 0b0000_0010;
        const AUXILARY = 0b0000_0100;
        const FOURTH = 0b0000_1000;
        const FIFTH = 0b0001_0000;
    }
}

fn from_str<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Button, D::Error> {
    let bits: u8 = Deserialize::deserialize(deserializer)?;
    Ok(Button::from_bits_truncate(bits))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PointerEvent {
    pub event_type: PointerEventType,
    pub pointer_id: i64,
    pub timestamp: u64,
    pub is_primary: bool,
    pub pointer_type: PointerType,
    #[serde(deserialize_with = "from_str")]
    pub button: Button,
    #[serde(deserialize_with = "from_str")]
    pub buttons: Button,
    pub x: f64,
    pub y: f64,
    pub movement_x: i64,
    pub movement_y: i64,
    pub pressure: f64,
    pub tilt_x: i32,
    pub tilt_y: i32,
    pub twist: i32,
    pub width: f64,
    pub height: f64,
}
