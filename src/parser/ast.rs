use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimitiveType {
    Bool,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    String,
    Bytes,
    Uuid,
    Timestamp,
}

impl PrimitiveType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PrimitiveType::Bool => "bool",
            PrimitiveType::U8 => "u8",
            PrimitiveType::U16 => "u16",
            PrimitiveType::U32 => "u32",
            PrimitiveType::U64 => "u64",
            PrimitiveType::I8 => "i8",
            PrimitiveType::I16 => "i16",
            PrimitiveType::I32 => "i32",
            PrimitiveType::I64 => "i64",
            PrimitiveType::F32 => "f32",
            PrimitiveType::F64 => "f64",
            PrimitiveType::String => "string",
            PrimitiveType::Bytes => "bytes",
            PrimitiveType::Uuid => "uuid",
            PrimitiveType::Timestamp => "timestamp",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldType {
    Primitive(PrimitiveType),
    List(Box<FieldType>),
    Map(Box<FieldType>, Box<FieldType>),
    Option(Box<FieldType>),
    Result {
        ok: Box<FieldType>,
        err: Box<FieldType>,
    },
    Custom(String),
}

impl FieldType {
    pub fn is_stream_compatible(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldModifier {
    Required,
    Optional,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LiteralValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<LiteralValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub id: u32,
    pub name: String,
    pub field_type: FieldType,
    pub modifier: FieldModifier,
    pub default_value: Option<LiteralValue>,
    pub attributes: Vec<Attribute>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<Field>,
    pub attributes: Vec<Attribute>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub value: i32,
    pub doc: Option<String>,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub attributes: Vec<Attribute>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamingMode {
    Unary,
    ServerStreaming,
    ClientStreaming,
    BiDirectional,
}

impl StreamingMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            StreamingMode::Unary => "Unary",
            StreamingMode::ServerStreaming => "ServerStreaming",
            StreamingMode::ClientStreaming => "ClientStreaming",
            StreamingMode::BiDirectional => "BiDirectional",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodDef {
    pub id: u32,
    pub name: String,
    pub request_type: FieldType,
    pub response_type: FieldType,
    pub streaming_mode: StreamingMode,
    pub attributes: Vec<Attribute>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceDef {
    pub name: String,
    pub methods: Vec<MethodDef>,
    pub attributes: Vec<Attribute>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Import {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Namespace {
    pub parts: Vec<String>,
}

impl Namespace {
    pub fn to_string(&self) -> String {
        self.parts.join(".")
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Schema {
    pub namespace: Option<Namespace>,
    pub imports: Vec<Import>,
    pub structs: Vec<StructDef>,
    pub enums: Vec<EnumDef>,
    pub services: Vec<ServiceDef>,
}

impl Schema {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn find_struct(&self, name: &str) -> Option<&StructDef> {
        self.structs.iter().find(|s| s.name == name)
    }

    pub fn find_enum(&self, name: &str) -> Option<&EnumDef> {
        self.enums.iter().find(|e| e.name == name)
    }

    pub fn find_service(&self, name: &str) -> Option<&ServiceDef> {
        self.services.iter().find(|s| s.name == name)
    }
}
