/// MYTHOS-CAN Value type
///
/// Represents any MYTHOS-CAN encoded value.
/// All values are self-describing with type tags.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// NULL value (tag 0x00)
    Null,

    /// Boolean value (tag 0x01 for false, 0x02 for true)
    Bool(bool),

    /// Unsigned integer, LEB128 encoded (tag 0x03)
    UVarint(u64),

    /// Signed integer, zigzag + LEB128 encoded (tag 0x04)
    IVarint(i64),

    /// Raw byte string (tag 0x05)
    Bytes(Vec<u8>),

    /// UTF-8 text string (tag 0x06)
    Text(String),

    /// List of values (tag 0x07)
    List(Vec<Value>),

    /// Map of key-value pairs (tag 0x08)
    /// Keys are kept in canonical sorted order by encoded bytes
    Map(Vec<(Value, Value)>),
}

// Type tag constants
pub mod tags {
    pub const NULL: u8 = 0x00;
    pub const BOOL_FALSE: u8 = 0x01;
    pub const BOOL_TRUE: u8 = 0x02;
    pub const UVARINT: u8 = 0x03;
    pub const IVARINT: u8 = 0x04;
    pub const BYTES: u8 = 0x05;
    pub const TEXT: u8 = 0x06;
    pub const LIST: u8 = 0x07;
    pub const MAP: u8 = 0x08;
}
