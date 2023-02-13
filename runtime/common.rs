/// Enum of Data Type. each one corresponds to a primitive type in JavaScript
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum DataTypeEnum {
    /// undefined
    Unspecified = 0,
    /// bigint
    Bigint = 1,
    /// number
    Number = 2,
    /// null
    Null = 3,
    /// string
    String = 4,
    /// boolean
    Boolean = 5,
    /// object
    Object = 6,
    /// array
    Array = 7,
}
impl DataTypeEnum {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            DataTypeEnum::Unspecified => "DATA_TYPE_ENUM_UNSPECIFIED",
            DataTypeEnum::Bigint => "DATA_TYPE_ENUM_BIGINT",
            DataTypeEnum::Number => "DATA_TYPE_ENUM_NUMBER",
            DataTypeEnum::Null => "DATA_TYPE_ENUM_NULL",
            DataTypeEnum::String => "DATA_TYPE_ENUM_STRING",
            DataTypeEnum::Boolean => "DATA_TYPE_ENUM_BOOLEAN",
            DataTypeEnum::Object => "DATA_TYPE_ENUM_OBJECT",
            DataTypeEnum::Array => "DATA_TYPE_ENUM_ARRAY",
        }
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Entry {
    #[prost(enumeration = "DataTypeEnum", tag = "1")]
    pub data_type: i32,
    /// entry value
    #[prost(bytes = "vec", tag = "2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
