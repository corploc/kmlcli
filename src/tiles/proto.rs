use prost::Message;

#[derive(Clone, Message)]
pub struct Tile {
    #[prost(message, repeated, tag = "3")]
    pub layers: Vec<Layer>,
}

#[derive(Clone, Message)]
pub struct Layer {
    #[prost(string, required, tag = "1")]
    pub name: String,
    #[prost(message, repeated, tag = "2")]
    pub features: Vec<Feature>,
    #[prost(string, repeated, tag = "3")]
    pub keys: Vec<String>,
    #[prost(message, repeated, tag = "4")]
    pub values: Vec<Value>,
    #[prost(uint32, optional, tag = "5")]
    pub extent: Option<u32>,
    #[prost(uint32, required, tag = "15")]
    pub version: u32,
}

#[derive(Clone, Message)]
pub struct Feature {
    #[prost(uint64, optional, tag = "1")]
    pub id: Option<u64>,
    #[prost(uint32, repeated, packed = "true", tag = "2")]
    pub tags: Vec<u32>,
    #[prost(enumeration = "GeomType", optional, tag = "3")]
    pub r#type: Option<i32>,
    #[prost(uint32, repeated, packed = "true", tag = "4")]
    pub geometry: Vec<u32>,
}

#[derive(Clone, Message)]
pub struct Value {
    #[prost(string, optional, tag = "1")]
    pub string_value: Option<String>,
    #[prost(float, optional, tag = "2")]
    pub float_value: Option<f32>,
    #[prost(double, optional, tag = "3")]
    pub double_value: Option<f64>,
    #[prost(int64, optional, tag = "4")]
    pub int_value: Option<i64>,
    #[prost(uint64, optional, tag = "5")]
    pub uint_value: Option<u64>,
    #[prost(sint64, optional, tag = "6")]
    pub sint_value: Option<i64>,
    #[prost(bool, optional, tag = "7")]
    pub bool_value: Option<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum GeomType {
    Unknown = 0,
    Point = 1,
    LineString = 2,
    Polygon = 3,
}

impl GeomType {
    pub fn from_i32(val: i32) -> Self {
        match val {
            1 => Self::Point,
            2 => Self::LineString,
            3 => Self::Polygon,
            _ => Self::Unknown,
        }
    }
}
