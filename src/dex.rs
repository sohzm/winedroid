use std::collections::BTreeMap;

use crate::error::{Result, WinedroidError};

const DEX_MAGIC_PREFIX: &[u8; 4] = b"dex\n";
const DEX_MAGIC_SUFFIX: u8 = 0;
const DEX_HEADER_SIZE: u32 = 0x70;
const DEX_ENDIAN_CONSTANT: u32 = 0x1234_5678;
const STRING_ID_SIZE: usize = 4;
const TYPE_ID_SIZE: usize = 4;
const PROTO_ID_SIZE: usize = 12;
const FIELD_ID_SIZE: usize = 8;
const METHOD_ID_SIZE: usize = 8;
const CLASS_DEF_SIZE: usize = 32;
const DEX_METADATA_CACHE_MAGIC: &[u8; 12] = b"WDDEXIDX0001";
const DEX_METADATA_CACHE_FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const DEX_METADATA_CACHE_FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DexSummary {
    pub version: String,
    pub checksum: u32,
    pub declared_file_size: u32,
    pub string_count: u32,
    pub type_count: u32,
    pub method_count: u32,
    pub class_count: u32,
    pub class_descriptors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DexFile {
    source_name: String,
    data: Vec<u8>,
    strings: Vec<String>,
    type_descriptors: Vec<String>,
    protos: Vec<DexProto>,
    fields: Vec<DexFieldId>,
    methods: Vec<DexMethodId>,
    classes: Vec<DexClassDef>,
    class_by_descriptor: BTreeMap<String, usize>,
}

impl DexFile {
    pub fn source_name(&self) -> &str {
        &self.source_name
    }

    pub fn method(&self, method_index: u32) -> Result<&DexMethodId> {
        self.methods.get(method_index as usize).ok_or_else(|| {
            WinedroidError::new(format!(
                "DEX method index {method_index} is outside the method table"
            ))
        })
    }

    pub fn field(&self, field_index: u32) -> Result<&DexFieldId> {
        self.fields.get(field_index as usize).ok_or_else(|| {
            WinedroidError::new(format!(
                "DEX field index {field_index} is outside the field table"
            ))
        })
    }

    pub fn string(&self, string_index: u32) -> Result<&str> {
        self.strings
            .get(string_index as usize)
            .map(String::as_str)
            .ok_or_else(|| {
                WinedroidError::new(format!(
                    "DEX string index {string_index} is outside the string table"
                ))
            })
    }

    pub fn type_descriptor(&self, type_index: u32) -> Result<&str> {
        self.type_descriptors
            .get(type_index as usize)
            .map(String::as_str)
            .ok_or_else(|| {
                WinedroidError::new(format!(
                    "DEX type index {type_index} is outside the type table"
                ))
            })
    }

    pub fn find_class(&self, descriptor: &str) -> Option<&DexClassDef> {
        self.class_by_descriptor
            .get(descriptor)
            .and_then(|index| self.classes.get(*index))
    }

    pub fn parse_code_item(&self, code_offset: u32) -> Result<DexCodeItem> {
        read_code_item(&self.data, code_offset as usize, &self.type_descriptors)
    }

    // cite: art dex_file_structs.h#AnnotationsDirectoryItem, AnnotationSetItem, AnnotationItem
    pub fn class_annotations(&self, class_def: &DexClassDef) -> Result<Vec<DexAnnotation>> {
        if class_def.annotations_offset == 0 {
            return Ok(Vec::new());
        }

        let directory_offset = class_def.annotations_offset as usize;
        let class_annotations_offset = read_u32(&self.data, directory_offset)? as usize;
        if class_annotations_offset == 0 {
            return Ok(Vec::new());
        }

        let entry_count = read_u32(&self.data, class_annotations_offset)? as usize;
        let mut annotations = Vec::with_capacity(entry_count);
        for entry_index in 0..entry_count {
            let entry_offset = class_annotations_offset
                .checked_add(4 + entry_index * 4)
                .ok_or_else(|| WinedroidError::new("annotation set entry offset overflow"))?;
            let annotation_offset = read_u32(&self.data, entry_offset)? as usize;
            require_range(&self.data, annotation_offset, 1, "annotation item")?;
            let visibility = self.data[annotation_offset];
            let (annotation, _) = self.read_encoded_annotation(annotation_offset + 1)?;

            annotations.push(DexAnnotation {
                visibility,
                type_descriptor: annotation.type_descriptor,
                elements: annotation.elements,
            });
        }

        Ok(annotations)
    }

    fn read_encoded_annotation(&self, offset: usize) -> Result<(DexEncodedAnnotation, usize)> {
        let (type_index, next_offset) = read_uleb128(&self.data, offset)?;
        let (element_count, mut offset) = read_uleb128(&self.data, next_offset)?;
        let type_descriptor = type_descriptor_at(&self.type_descriptors, type_index)?.to_owned();

        let mut elements = Vec::with_capacity(element_count as usize);
        for _ in 0..element_count {
            let (name_index, next_offset) = read_uleb128(&self.data, offset)?;
            let name = self
                .strings
                .get(name_index as usize)
                .ok_or_else(|| {
                    WinedroidError::new(format!(
                        "annotation element name index {name_index} is outside the string table"
                    ))
                })?
                .clone();
            let (value, next_offset) = self.read_encoded_value(next_offset)?;
            offset = next_offset;

            elements.push(DexAnnotationElement { name, value });
        }

        Ok((
            DexEncodedAnnotation {
                type_descriptor,
                elements,
            },
            offset,
        ))
    }

    // cite: art dex_file.h#kDexAnnotationValueTypeMask, kDexAnnotationValueArgShift
    fn read_encoded_value(&self, offset: usize) -> Result<(DexAnnotationValue, usize)> {
        require_range(&self.data, offset, 1, "encoded value header")?;
        let header = self.data[offset];
        let value_type = header & DEX_ANNOTATION_VALUE_TYPE_MASK;
        let value_arg = usize::from(header >> DEX_ANNOTATION_VALUE_ARG_SHIFT);
        let offset = offset + 1;

        // cite: art dex_file.h#kDexAnnotationByte, kDexAnnotationShort, kDexAnnotationChar, kDexAnnotationInt, kDexAnnotationLong
        let value = match value_type {
            DEX_ANNOTATION_BYTE | DEX_ANNOTATION_SHORT | DEX_ANNOTATION_INT => {
                let signed = self.read_sized_signed(offset, value_arg + 1)?;
                let signed = i32::try_from(signed).map_err(|_| {
                    WinedroidError::new(format!(
                        "encoded int value at offset {offset} does not fit in 32 bits"
                    ))
                })?;
                (DexAnnotationValue::Int(signed), offset + value_arg + 1)
            }
            DEX_ANNOTATION_LONG => {
                let signed = self.read_sized_signed(offset, value_arg + 1)?;
                (DexAnnotationValue::Long(signed), offset + value_arg + 1)
            }
            DEX_ANNOTATION_CHAR => {
                let unsigned = self.read_sized_unsigned(offset, value_arg + 1)?;
                (
                    DexAnnotationValue::Int(unsigned as u16 as i32),
                    offset + value_arg + 1,
                )
            }
            // cite: art dex_file.h#kDexAnnotationFloat, kDexAnnotationDouble
            DEX_ANNOTATION_FLOAT => {
                let size = value_arg + 1;
                if size > 4 {
                    return Err(WinedroidError::new(format!(
                        "encoded float value at offset {offset} declares {size} bytes"
                    )));
                }
                let unsigned = self.read_sized_unsigned(offset, size)?;
                let bits = (unsigned as u32) << (8 * (4 - size));
                (
                    DexAnnotationValue::Float(f32::from_bits(bits)),
                    offset + size,
                )
            }
            DEX_ANNOTATION_DOUBLE => {
                let size = value_arg + 1;
                let unsigned = self.read_sized_unsigned(offset, size)?;
                let bits = unsigned << (8 * (8 - size));
                (
                    DexAnnotationValue::Double(f64::from_bits(bits)),
                    offset + size,
                )
            }
            // cite: art dex_file.h#kDexAnnotationString, kDexAnnotationType
            DEX_ANNOTATION_STRING => {
                let index = self.read_sized_unsigned(offset, value_arg + 1)? as u32;
                let value = self.string(index)?.to_owned();
                (DexAnnotationValue::String(value), offset + value_arg + 1)
            }
            DEX_ANNOTATION_TYPE => {
                let index = self.read_sized_unsigned(offset, value_arg + 1)? as u32;
                let descriptor = self.type_descriptor(index)?.to_owned();
                (DexAnnotationValue::Type(descriptor), offset + value_arg + 1)
            }
            // cite: art dex_file.h#kDexAnnotationEnum
            DEX_ANNOTATION_ENUM => {
                let index = self.read_sized_unsigned(offset, value_arg + 1)? as u32;
                let field = self.field(index)?.clone();
                (DexAnnotationValue::Enum(field), offset + value_arg + 1)
            }
            // cite: art dex_file.h#kDexAnnotationArray
            DEX_ANNOTATION_ARRAY => {
                let (element_count, mut offset) = read_uleb128(&self.data, offset)?;
                let mut values = Vec::with_capacity(element_count as usize);
                for _ in 0..element_count {
                    let (value, next_offset) = self.read_encoded_value(offset)?;
                    offset = next_offset;
                    values.push(value);
                }
                (DexAnnotationValue::Array(values), offset)
            }
            // cite: art dex_file.h#kDexAnnotationAnnotation
            DEX_ANNOTATION_ANNOTATION => {
                let (annotation, offset) = self.read_encoded_annotation(offset)?;
                (DexAnnotationValue::Annotation(annotation), offset)
            }
            // cite: art dex_file.h#kDexAnnotationNull, kDexAnnotationBoolean
            DEX_ANNOTATION_NULL => (DexAnnotationValue::Null, offset),
            DEX_ANNOTATION_BOOLEAN => (DexAnnotationValue::Boolean(value_arg != 0), offset),
            // cite: art dex_file.h#kDexAnnotationMethodType, kDexAnnotationMethodHandle, kDexAnnotationField, kDexAnnotationMethod
            DEX_ANNOTATION_METHOD_TYPE
            | DEX_ANNOTATION_METHOD_HANDLE
            | DEX_ANNOTATION_FIELD
            | DEX_ANNOTATION_METHOD => (
                DexAnnotationValue::Unsupported { value_type },
                offset + value_arg + 1,
            ),
            _ => {
                return Err(WinedroidError::new(format!(
                    "encoded value type {value_type:#04x} at offset {offset} is not a DEX value type"
                )));
            }
        };

        Ok(value)
    }

    fn read_sized_signed(&self, offset: usize, size: usize) -> Result<i64> {
        let unsigned = self.read_sized_unsigned(offset, size)?;
        let shift = 64 - 8 * size as u32;

        Ok(((unsigned << shift) as i64) >> shift)
    }

    fn read_sized_unsigned(&self, offset: usize, size: usize) -> Result<u64> {
        if size == 0 || size > 8 {
            return Err(WinedroidError::new(format!(
                "encoded value at offset {offset} declares {size} bytes"
            )));
        }
        require_range(&self.data, offset, size, "encoded value")?;
        let mut value = 0u64;
        for index in (0..size).rev() {
            value = (value << 8) | u64::from(self.data[offset + index]);
        }

        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DexProto {
    pub shorty: String,
    pub return_type: String,
    pub parameters: Vec<String>,
}

impl DexProto {
    pub fn descriptor(&self) -> String {
        format!("({}){}", self.parameters.join(""), self.return_type)
    }

    pub fn descriptor_is(&self, expected: &str) -> bool {
        let Some(mut rest) = expected.strip_prefix('(') else {
            return false;
        };
        for parameter in &self.parameters {
            let Some(remainder) = rest.strip_prefix(parameter.as_str()) else {
                return false;
            };
            rest = remainder;
        }
        let Some(remainder) = rest.strip_prefix(')') else {
            return false;
        };
        remainder == self.return_type
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DexMethodId {
    pub class_descriptor: String,
    pub name: String,
    pub proto: DexProto,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DexFieldId {
    pub class_descriptor: String,
    pub type_descriptor: String,
    pub name: String,
}

impl DexFieldId {
    pub fn display_signature(&self) -> String {
        format!(
            "{}.{}:{}",
            self.class_descriptor, self.name, self.type_descriptor
        )
    }
}

impl DexMethodId {
    pub fn display_signature(&self) -> String {
        format!("{}{}", self.name, self.proto.descriptor())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DexClassDef {
    pub descriptor: String,
    pub superclass_descriptor: Option<String>,
    pub interface_descriptors: Vec<String>,
    pub access_flags: u32,
    pub annotations_offset: u32,
    pub class_data_offset: u32,
    pub static_fields: Vec<DexEncodedField>,
    pub instance_fields: Vec<DexEncodedField>,
    pub direct_methods: Vec<DexEncodedMethod>,
    pub virtual_methods: Vec<DexEncodedMethod>,
    pub static_values: Vec<DexStaticValue>,
}

// cite: art dex_file_structs.h#static_values_off_
// cite: art dex_file.h#kDexAnnotationByte, kDexAnnotationValueTypeMask, kDexAnnotationValueArgShift
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DexStaticValue {
    Int(i32),
    Long(i64),
    FloatBits(u32),
    DoubleBits(u64),
    String(String),
    Null,
    Boolean(bool),
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DexEncodedField {
    pub field_index: u32,
    pub access_flags: u32,
    pub kind: DexFieldKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DexFieldKind {
    Static,
    Instance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DexEncodedMethod {
    pub method_index: u32,
    pub access_flags: u32,
    pub code_offset: u32,
    pub kind: DexMethodKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DexMethodKind {
    Direct,
    Virtual,
}

impl std::fmt::Display for DexMethodKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Direct => write!(formatter, "direct"),
            Self::Virtual => write!(formatter, "virtual"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DexCodeItem {
    pub registers_size: u16,
    pub ins_size: u16,
    pub outs_size: u16,
    pub tries_size: u16,
    pub debug_info_offset: u32,
    pub instructions: Vec<u16>,
    pub try_blocks: Vec<DexTryBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DexTryBlock {
    pub start_address: u32,
    pub instruction_count: u16,
    pub handlers: Vec<DexExceptionHandler>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DexExceptionHandler {
    pub type_descriptor: Option<String>,
    pub address: u32,
}

// cite: art dex_file.h#kDexVisibilityRuntime
pub const DEX_VISIBILITY_RUNTIME: u8 = 0x01;

// cite: art dex_file.h#kDexAnnotationByte, kDexAnnotationShort, kDexAnnotationChar, kDexAnnotationInt, kDexAnnotationLong, kDexAnnotationFloat, kDexAnnotationDouble, kDexAnnotationMethodType, kDexAnnotationMethodHandle, kDexAnnotationString, kDexAnnotationType, kDexAnnotationField, kDexAnnotationMethod, kDexAnnotationEnum, kDexAnnotationArray, kDexAnnotationAnnotation, kDexAnnotationNull, kDexAnnotationBoolean, kDexAnnotationValueTypeMask, kDexAnnotationValueArgShift
const DEX_ANNOTATION_BYTE: u8 = 0x00;
const DEX_ANNOTATION_SHORT: u8 = 0x02;
const DEX_ANNOTATION_CHAR: u8 = 0x03;
const DEX_ANNOTATION_INT: u8 = 0x04;
const DEX_ANNOTATION_LONG: u8 = 0x06;
const DEX_ANNOTATION_FLOAT: u8 = 0x10;
const DEX_ANNOTATION_DOUBLE: u8 = 0x11;
const DEX_ANNOTATION_METHOD_TYPE: u8 = 0x15;
const DEX_ANNOTATION_METHOD_HANDLE: u8 = 0x16;
const DEX_ANNOTATION_STRING: u8 = 0x17;
const DEX_ANNOTATION_TYPE: u8 = 0x18;
const DEX_ANNOTATION_FIELD: u8 = 0x19;
const DEX_ANNOTATION_METHOD: u8 = 0x1a;
const DEX_ANNOTATION_ENUM: u8 = 0x1b;
const DEX_ANNOTATION_ARRAY: u8 = 0x1c;
const DEX_ANNOTATION_ANNOTATION: u8 = 0x1d;
const DEX_ANNOTATION_NULL: u8 = 0x1e;
const DEX_ANNOTATION_BOOLEAN: u8 = 0x1f;
const DEX_ANNOTATION_VALUE_TYPE_MASK: u8 = 0x1f;
const DEX_ANNOTATION_VALUE_ARG_SHIFT: u8 = 5;

#[derive(Debug, Clone, PartialEq)]
pub struct DexAnnotation {
    pub visibility: u8,
    pub type_descriptor: String,
    pub elements: Vec<DexAnnotationElement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DexEncodedAnnotation {
    pub type_descriptor: String,
    pub elements: Vec<DexAnnotationElement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DexAnnotationElement {
    pub name: String,
    pub value: DexAnnotationValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DexAnnotationValue {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    Type(String),
    Enum(DexFieldId),
    Array(Vec<DexAnnotationValue>),
    Annotation(DexEncodedAnnotation),
    Null,
    Boolean(bool),
    Unsupported { value_type: u8 },
}

pub fn parse_dex_file(source_name: impl Into<String>, data: Vec<u8>) -> Result<DexFile> {
    let header = DexHeader::parse(&data)?;
    let strings = read_strings(&data, &header)?;
    let type_descriptors = read_type_descriptors(&data, &header, &strings)?;
    let protos = read_protos(&data, &header, &strings, &type_descriptors)?;
    let fields = read_fields(&data, &header, &type_descriptors, &strings)?;
    let methods = read_methods(&data, &header, &type_descriptors, &strings, &protos)?;
    let classes = read_class_defs(&data, &header, &type_descriptors, &strings)?;
    let mut class_by_descriptor = BTreeMap::new();
    for (index, class_def) in classes.iter().enumerate() {
        class_by_descriptor
            .entry(class_def.descriptor.clone())
            .or_insert(index);
    }

    Ok(DexFile {
        source_name: source_name.into(),
        data,
        strings,
        type_descriptors,
        protos,
        fields,
        methods,
        classes,
        class_by_descriptor,
    })
}

pub fn encode_dex_metadata_cache(file: &DexFile) -> Result<Vec<u8>> {
    let header = DexHeader::parse(&file.data)?;
    let mut payload = Vec::new();
    write_cache_string(&mut payload, &header.version)?;
    write_cache_u32(&mut payload, header.checksum);
    write_cache_u32(&mut payload, header.file_size);
    write_cache_string_vec(&mut payload, &file.strings)?;
    write_cache_string_vec(&mut payload, &file.type_descriptors)?;
    write_cache_protos(&mut payload, &file.protos)?;
    write_cache_fields(&mut payload, &file.fields)?;
    write_cache_methods(&mut payload, &file.methods)?;
    write_cache_classes(&mut payload, &file.classes)?;

    let mut cache = Vec::with_capacity(DEX_METADATA_CACHE_MAGIC.len() + 8 + payload.len());
    cache.extend_from_slice(DEX_METADATA_CACHE_MAGIC);
    write_cache_u64(&mut cache, dex_metadata_cache_checksum(&payload));
    cache.extend_from_slice(&payload);

    Ok(cache)
}

pub fn parse_dex_file_with_metadata_cache(
    source_name: impl Into<String>,
    data: Vec<u8>,
    cache: &[u8],
) -> Result<DexFile> {
    let source_name = source_name.into();
    let header = DexHeader::parse(&data)?;
    let mut reader = DexMetadataCacheReader::new(cache)?;
    let version = reader.read_string()?;
    let checksum = reader.read_u32()?;
    let file_size = reader.read_u32()?;
    if version != header.version || checksum != header.checksum || file_size != header.file_size {
        return Err(WinedroidError::new(format!(
            "DEX metadata cache for `{source_name}` does not match the DEX header"
        )));
    }

    let strings = reader.read_string_vec()?;
    require_cache_count(
        &source_name,
        "string",
        strings.len(),
        header.string_ids_size,
    )?;
    let type_descriptors = reader.read_string_vec()?;
    require_cache_count(
        &source_name,
        "type",
        type_descriptors.len(),
        header.type_ids_size,
    )?;
    let protos = reader.read_protos()?;
    require_cache_count(&source_name, "proto", protos.len(), header.proto_ids_size)?;
    let fields = reader.read_fields()?;
    require_cache_count(&source_name, "field", fields.len(), header.field_ids_size)?;
    let methods = reader.read_methods()?;
    require_cache_count(
        &source_name,
        "method",
        methods.len(),
        header.method_ids_size,
    )?;
    let classes = reader.read_classes()?;
    require_cache_count(&source_name, "class", classes.len(), header.class_defs_size)?;
    reader.require_finished()?;

    let mut class_by_descriptor = BTreeMap::new();
    for (index, class_def) in classes.iter().enumerate() {
        class_by_descriptor
            .entry(class_def.descriptor.clone())
            .or_insert(index);
    }

    Ok(DexFile {
        source_name,
        data,
        strings,
        type_descriptors,
        protos,
        fields,
        methods,
        classes,
        class_by_descriptor,
    })
}

pub fn parse_dex(data: &[u8]) -> Result<DexSummary> {
    let header = DexHeader::parse(data)?;
    let mut class_descriptors = Vec::with_capacity(header.class_defs_size as usize);

    for index in 0..header.class_defs_size {
        let class_def_offset = checked_table_item_offset(
            header.class_defs_off,
            index,
            CLASS_DEF_SIZE,
            "class definition",
        )?;
        require_range(data, class_def_offset, CLASS_DEF_SIZE, "class definition")?;

        let class_idx = read_u32(data, class_def_offset)?;
        let descriptor = read_type_descriptor(data, &header, class_idx)?;
        class_descriptors.push(descriptor);
    }

    class_descriptors.sort();

    Ok(DexSummary {
        version: header.version,
        checksum: header.checksum,
        declared_file_size: header.file_size,
        string_count: header.string_ids_size,
        type_count: header.type_ids_size,
        method_count: header.method_ids_size,
        class_count: header.class_defs_size,
        class_descriptors,
    })
}

pub fn is_dex_file(name: &str) -> bool {
    if name == "classes.dex" {
        return true;
    }

    let Some(number) = name
        .strip_prefix("classes")
        .and_then(|name| name.strip_suffix(".dex"))
    else {
        return false;
    };

    !number.is_empty() && number.chars().all(|character| character.is_ascii_digit())
}

fn read_type_descriptor(data: &[u8], header: &DexHeader, type_index: u32) -> Result<String> {
    if type_index >= header.type_ids_size {
        return Err(WinedroidError::new(format!(
            "DEX type index {type_index} is outside the type table"
        )));
    }

    let type_offset =
        checked_table_item_offset(header.type_ids_off, type_index, TYPE_ID_SIZE, "type id")?;
    require_range(data, type_offset, TYPE_ID_SIZE, "type id")?;

    let descriptor_index = read_u32(data, type_offset)?;
    read_string(data, header, descriptor_index)
}

fn read_strings(data: &[u8], header: &DexHeader) -> Result<Vec<String>> {
    let mut strings = Vec::with_capacity(header.string_ids_size as usize);

    for index in 0..header.string_ids_size {
        strings.push(read_string(data, header, index)?);
    }

    Ok(strings)
}

fn read_type_descriptors(
    data: &[u8],
    header: &DexHeader,
    strings: &[String],
) -> Result<Vec<String>> {
    let mut descriptors = Vec::with_capacity(header.type_ids_size as usize);

    for index in 0..header.type_ids_size {
        let type_offset =
            checked_table_item_offset(header.type_ids_off, index, TYPE_ID_SIZE, "type id")?;
        require_range(data, type_offset, TYPE_ID_SIZE, "type id")?;

        let descriptor_index = read_u32(data, type_offset)?;
        descriptors.push(string_at(strings, descriptor_index)?.to_owned());
    }

    Ok(descriptors)
}

fn read_protos(
    data: &[u8],
    header: &DexHeader,
    strings: &[String],
    type_descriptors: &[String],
) -> Result<Vec<DexProto>> {
    let mut protos = Vec::with_capacity(header.proto_ids_size as usize);

    for index in 0..header.proto_ids_size {
        let proto_offset =
            checked_table_item_offset(header.proto_ids_off, index, PROTO_ID_SIZE, "proto id")?;
        require_range(data, proto_offset, PROTO_ID_SIZE, "proto id")?;

        let shorty_index = read_u32(data, proto_offset)?;
        let return_type_index = read_u32(data, proto_offset + 4)?;
        let parameters_offset = read_u32(data, proto_offset + 8)?;

        protos.push(DexProto {
            shorty: string_at(strings, shorty_index)?.to_owned(),
            return_type: type_descriptor_at(type_descriptors, return_type_index)?.to_owned(),
            parameters: read_proto_parameters(data, type_descriptors, parameters_offset)?,
        });
    }

    Ok(protos)
}

fn read_proto_parameters(
    data: &[u8],
    type_descriptors: &[String],
    parameters_offset: u32,
) -> Result<Vec<String>> {
    read_type_list(data, type_descriptors, parameters_offset, "proto parameter")
}

fn read_type_list(
    data: &[u8],
    type_descriptors: &[String],
    type_list_offset: u32,
    label: &str,
) -> Result<Vec<String>> {
    if type_list_offset == 0 {
        return Ok(Vec::new());
    }

    let offset = type_list_offset as usize;
    require_range(data, offset, 4, &format!("{label} type list"))?;
    let size = read_u32(data, offset)?;
    let mut type_list = Vec::with_capacity(size as usize);
    let mut item_offset = offset + 4;

    for _ in 0..size {
        require_range(data, item_offset, 2, &format!("{label} type item"))?;
        let type_index = u32::from(read_u16(data, item_offset)?);
        type_list.push(type_descriptor_at(type_descriptors, type_index)?.to_owned());
        item_offset += 2;
    }

    Ok(type_list)
}

fn read_methods(
    data: &[u8],
    header: &DexHeader,
    type_descriptors: &[String],
    strings: &[String],
    protos: &[DexProto],
) -> Result<Vec<DexMethodId>> {
    let mut methods = Vec::with_capacity(header.method_ids_size as usize);

    for index in 0..header.method_ids_size {
        let method_offset =
            checked_table_item_offset(header.method_ids_off, index, METHOD_ID_SIZE, "method id")?;
        require_range(data, method_offset, METHOD_ID_SIZE, "method id")?;

        let class_index = u32::from(read_u16(data, method_offset)?);
        let proto_index = u32::from(read_u16(data, method_offset + 2)?);
        let name_index = read_u32(data, method_offset + 4)?;

        methods.push(DexMethodId {
            class_descriptor: type_descriptor_at(type_descriptors, class_index)?.to_owned(),
            name: string_at(strings, name_index)?.to_owned(),
            proto: proto_at(protos, proto_index)?.clone(),
        });
    }

    Ok(methods)
}

fn read_fields(
    data: &[u8],
    header: &DexHeader,
    type_descriptors: &[String],
    strings: &[String],
) -> Result<Vec<DexFieldId>> {
    let mut fields = Vec::with_capacity(header.field_ids_size as usize);

    for index in 0..header.field_ids_size {
        let field_offset =
            checked_table_item_offset(header.field_ids_off, index, FIELD_ID_SIZE, "field id")?;
        require_range(data, field_offset, FIELD_ID_SIZE, "field id")?;

        let class_index = u32::from(read_u16(data, field_offset)?);
        let type_index = u32::from(read_u16(data, field_offset + 2)?);
        let name_index = read_u32(data, field_offset + 4)?;

        fields.push(DexFieldId {
            class_descriptor: type_descriptor_at(type_descriptors, class_index)?.to_owned(),
            type_descriptor: type_descriptor_at(type_descriptors, type_index)?.to_owned(),
            name: string_at(strings, name_index)?.to_owned(),
        });
    }

    Ok(fields)
}

fn read_class_defs(
    data: &[u8],
    header: &DexHeader,
    type_descriptors: &[String],
    strings: &[String],
) -> Result<Vec<DexClassDef>> {
    let mut classes = Vec::with_capacity(header.class_defs_size as usize);

    for index in 0..header.class_defs_size {
        let class_def_offset = checked_table_item_offset(
            header.class_defs_off,
            index,
            CLASS_DEF_SIZE,
            "class definition",
        )?;
        require_range(data, class_def_offset, CLASS_DEF_SIZE, "class definition")?;

        let class_index = read_u32(data, class_def_offset)?;
        let access_flags = read_u32(data, class_def_offset + 4)?;
        let superclass_index = read_u32(data, class_def_offset + 8)?;
        let interfaces_offset = read_u32(data, class_def_offset + 12)?;
        // cite: art dex_file_structs.h#ClassDef
        let annotations_offset = read_u32(data, class_def_offset + 20)?;
        let class_data_offset = read_u32(data, class_def_offset + 24)?;
        let static_values_offset = read_u32(data, class_def_offset + 28)?;
        let static_values = read_encoded_static_values(data, static_values_offset, strings)?;
        let class_data = read_class_data(
            data,
            header.field_ids_size,
            header.method_ids_size,
            class_data_offset,
        )?;
        let superclass_descriptor = if superclass_index == u32::MAX {
            None
        } else {
            Some(type_descriptor_at(type_descriptors, superclass_index)?.to_owned())
        };

        classes.push(DexClassDef {
            descriptor: type_descriptor_at(type_descriptors, class_index)?.to_owned(),
            superclass_descriptor,
            interface_descriptors: read_type_list(
                data,
                type_descriptors,
                interfaces_offset,
                "class interface",
            )?,
            access_flags,
            annotations_offset,
            class_data_offset,
            static_fields: class_data.static_fields,
            instance_fields: class_data.instance_fields,
            direct_methods: class_data.direct_methods,
            virtual_methods: class_data.virtual_methods,
            static_values,
        });
    }

    Ok(classes)
}

// cite: art dex_file.h#kDexAnnotationByte, kDexAnnotationValueTypeMask, kDexAnnotationValueArgShift
fn read_encoded_static_values(
    data: &[u8],
    static_values_offset: u32,
    strings: &[String],
) -> Result<Vec<DexStaticValue>> {
    if static_values_offset == 0 {
        return Ok(Vec::new());
    }

    let (size, mut offset) = read_uleb128(data, static_values_offset as usize)?;
    let mut values = Vec::with_capacity(size as usize);
    for _ in 0..size {
        let (value, next_offset) = read_encoded_value(data, offset, strings)?;
        values.push(value);
        offset = next_offset;
    }

    Ok(values)
}

fn read_encoded_value(
    data: &[u8],
    offset: usize,
    strings: &[String],
) -> Result<(DexStaticValue, usize)> {
    let header = *data.get(offset).ok_or_else(|| {
        WinedroidError::new("encoded value header extends past end of DEX data".to_owned())
    })?;
    let value_type = header & 0x1f;
    let value_arg = (header >> 5) as usize;
    let offset = offset + 1;

    let payload = |size: usize| -> Result<&[u8]> {
        data.get(offset..offset + size).ok_or_else(|| {
            WinedroidError::new("encoded value payload extends past end of DEX data".to_owned())
        })
    };
    let unsigned = |size: usize| -> Result<u64> {
        let bytes = payload(size)?;
        let mut value = 0u64;
        for (index, byte) in bytes.iter().enumerate() {
            value |= (*byte as u64) << (8 * index);
        }
        Ok(value)
    };
    let signed = |size: usize| -> Result<i64> {
        let value = unsigned(size)? as i64;
        let shift = 64 - 8 * size as u32;
        Ok(value.wrapping_shl(shift).wrapping_shr(shift))
    };

    let size = value_arg + 1;
    match value_type {
        0x00 => Ok((DexStaticValue::Int(signed(1)? as i32), offset + 1)),
        0x02 | 0x04 => Ok((DexStaticValue::Int(signed(size)? as i32), offset + size)),
        0x03 => Ok((DexStaticValue::Int(unsigned(size)? as i32), offset + size)),
        0x06 => Ok((DexStaticValue::Long(signed(size)?), offset + size)),
        0x10 => {
            let bits = (unsigned(size)? as u32) << (8 * (4 - size) as u32);
            Ok((DexStaticValue::FloatBits(bits), offset + size))
        }
        0x11 => {
            let bits = unsigned(size)? << (8 * (8 - size) as u32);
            Ok((DexStaticValue::DoubleBits(bits), offset + size))
        }
        0x17 => {
            let string_index = unsigned(size)? as u32;
            let value = string_at(strings, string_index)?.to_owned();
            Ok((DexStaticValue::String(value), offset + size))
        }
        0x15 | 0x16 | 0x18 | 0x19 | 0x1a | 0x1b => Ok((DexStaticValue::Unsupported, offset + size)),
        0x1c => {
            let (count, mut next_offset) = read_uleb128(data, offset)?;
            for _ in 0..count {
                let (_, after_value) = read_encoded_value(data, next_offset, strings)?;
                next_offset = after_value;
            }
            Ok((DexStaticValue::Unsupported, next_offset))
        }
        0x1d => {
            let (_type_index, after_type) = read_uleb128(data, offset)?;
            let (count, mut next_offset) = read_uleb128(data, after_type)?;
            for _ in 0..count {
                let (_name_index, after_name) = read_uleb128(data, next_offset)?;
                let (_, after_value) = read_encoded_value(data, after_name, strings)?;
                next_offset = after_value;
            }
            Ok((DexStaticValue::Unsupported, next_offset))
        }
        0x1e => Ok((DexStaticValue::Null, offset)),
        0x1f => Ok((DexStaticValue::Boolean(value_arg != 0), offset)),
        other => Err(WinedroidError::new(format!(
            "unsupported DEX encoded value type 0x{other:02x}"
        ))),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DexClassData {
    static_fields: Vec<DexEncodedField>,
    instance_fields: Vec<DexEncodedField>,
    direct_methods: Vec<DexEncodedMethod>,
    virtual_methods: Vec<DexEncodedMethod>,
}

fn read_class_data(
    data: &[u8],
    field_count: u32,
    method_count: u32,
    class_data_offset: u32,
) -> Result<DexClassData> {
    if class_data_offset == 0 {
        return Ok(DexClassData {
            static_fields: Vec::new(),
            instance_fields: Vec::new(),
            direct_methods: Vec::new(),
            virtual_methods: Vec::new(),
        });
    }

    let mut offset = class_data_offset as usize;
    let (static_fields_size, next_offset) = read_uleb128(data, offset)?;
    offset = next_offset;
    let (instance_fields_size, next_offset) = read_uleb128(data, offset)?;
    offset = next_offset;
    let (direct_methods_size, next_offset) = read_uleb128(data, offset)?;
    offset = next_offset;
    let (virtual_methods_size, next_offset) = read_uleb128(data, offset)?;
    offset = next_offset;

    let (static_fields, next_offset) = read_encoded_fields(
        data,
        offset,
        static_fields_size,
        field_count,
        DexFieldKind::Static,
    )?;
    let (instance_fields, next_offset) = read_encoded_fields(
        data,
        next_offset,
        instance_fields_size,
        field_count,
        DexFieldKind::Instance,
    )?;

    let (direct_methods, next_offset) = read_encoded_methods(
        data,
        next_offset,
        direct_methods_size,
        method_count,
        DexMethodKind::Direct,
    )?;
    let (virtual_methods, _) = read_encoded_methods(
        data,
        next_offset,
        virtual_methods_size,
        method_count,
        DexMethodKind::Virtual,
    )?;

    Ok(DexClassData {
        static_fields,
        instance_fields,
        direct_methods,
        virtual_methods,
    })
}

fn read_encoded_fields(
    data: &[u8],
    mut offset: usize,
    field_count: u32,
    total_field_count: u32,
    kind: DexFieldKind,
) -> Result<(Vec<DexEncodedField>, usize)> {
    let mut fields = Vec::with_capacity(field_count as usize);
    let mut field_index = 0u32;

    for _ in 0..field_count {
        let (field_index_diff, next_offset) = read_uleb128(data, offset)?;
        offset = next_offset;
        let (access_flags, next_offset) = read_uleb128(data, offset)?;
        offset = next_offset;
        field_index = field_index
            .checked_add(field_index_diff)
            .ok_or_else(|| WinedroidError::new("DEX encoded field index overflow"))?;

        if field_index >= total_field_count {
            return Err(WinedroidError::new(format!(
                "DEX encoded field index {field_index} is outside the field table"
            )));
        }

        fields.push(DexEncodedField {
            field_index,
            access_flags,
            kind,
        });
    }

    Ok((fields, offset))
}

fn read_encoded_methods(
    data: &[u8],
    mut offset: usize,
    method_count: u32,
    total_method_count: u32,
    kind: DexMethodKind,
) -> Result<(Vec<DexEncodedMethod>, usize)> {
    let mut methods = Vec::with_capacity(method_count as usize);
    let mut method_index = 0u32;

    for _ in 0..method_count {
        let (method_index_diff, next_offset) = read_uleb128(data, offset)?;
        offset = next_offset;
        let (access_flags, next_offset) = read_uleb128(data, offset)?;
        offset = next_offset;
        let (code_offset, next_offset) = read_uleb128(data, offset)?;
        offset = next_offset;
        method_index = method_index
            .checked_add(method_index_diff)
            .ok_or_else(|| WinedroidError::new("DEX encoded method index overflow"))?;

        if method_index >= total_method_count {
            return Err(WinedroidError::new(format!(
                "DEX encoded method index {method_index} is outside the method table"
            )));
        }

        methods.push(DexEncodedMethod {
            method_index,
            access_flags,
            code_offset,
            kind,
        });
    }

    Ok((methods, offset))
}

fn read_code_item(data: &[u8], offset: usize, type_descriptors: &[String]) -> Result<DexCodeItem> {
    require_range(data, offset, 16, "code item")?;

    let registers_size = read_u16(data, offset)?;
    let ins_size = read_u16(data, offset + 2)?;
    let outs_size = read_u16(data, offset + 4)?;
    let tries_size = read_u16(data, offset + 6)?;
    let debug_info_offset = read_u32(data, offset + 8)?;
    let instruction_count = read_u32(data, offset + 12)?;
    let instruction_bytes = (instruction_count as usize)
        .checked_mul(2)
        .ok_or_else(|| WinedroidError::new("code item instruction size overflow"))?;
    let instructions_offset = offset + 16;
    require_range(
        data,
        instructions_offset,
        instruction_bytes,
        "code item instructions",
    )?;

    let mut instructions = Vec::with_capacity(instruction_count as usize);
    for index in 0..instruction_count as usize {
        instructions.push(read_u16(data, instructions_offset + index * 2)?);
    }
    let try_blocks = read_try_blocks(
        data,
        instructions_offset + instruction_bytes,
        tries_size,
        instruction_count,
        type_descriptors,
    )?;

    Ok(DexCodeItem {
        registers_size,
        ins_size,
        outs_size,
        tries_size,
        debug_info_offset,
        instructions,
        try_blocks,
    })
}

fn read_try_blocks(
    data: &[u8],
    instructions_end_offset: usize,
    tries_size: u16,
    instruction_count: u32,
    type_descriptors: &[String],
) -> Result<Vec<DexTryBlock>> {
    if tries_size == 0 {
        return Ok(Vec::new());
    }

    let padding = if instruction_count % 2 == 1 { 2 } else { 0 };
    let tries_offset = instructions_end_offset
        .checked_add(padding)
        .ok_or_else(|| WinedroidError::new("code item tries offset overflow"))?;
    let tries_bytes = usize::from(tries_size)
        .checked_mul(8)
        .ok_or_else(|| WinedroidError::new("code item tries size overflow"))?;
    require_range(data, tries_offset, tries_bytes, "code item tries")?;

    let handlers_offset = tries_offset
        .checked_add(tries_bytes)
        .ok_or_else(|| WinedroidError::new("code item handlers offset overflow"))?;
    let handler_lists = read_encoded_catch_handler_list(data, handlers_offset, type_descriptors)?;
    let mut try_blocks = Vec::with_capacity(usize::from(tries_size));

    for index in 0..usize::from(tries_size) {
        let item_offset = tries_offset + index * 8;
        let start_address = read_u32(data, item_offset)?;
        let instruction_count = read_u16(data, item_offset + 4)?;
        let handler_offset = usize::from(read_u16(data, item_offset + 6)?);
        let Some((_, handlers)) = handler_lists
            .iter()
            .find(|(offset, _handlers)| *offset == handler_offset)
        else {
            return Err(WinedroidError::new(format!(
                "DEX try item references missing catch handler offset {handler_offset}"
            )));
        };

        try_blocks.push(DexTryBlock {
            start_address,
            instruction_count,
            handlers: handlers.clone(),
        });
    }

    Ok(try_blocks)
}

fn read_encoded_catch_handler_list(
    data: &[u8],
    offset: usize,
    type_descriptors: &[String],
) -> Result<Vec<(usize, Vec<DexExceptionHandler>)>> {
    let (handler_count, mut current_offset) = read_uleb128(data, offset)?;
    let mut handlers = Vec::with_capacity(handler_count as usize);

    for _ in 0..handler_count {
        let handler_relative_offset = current_offset
            .checked_sub(offset)
            .ok_or_else(|| WinedroidError::new("catch handler offset underflow"))?;
        let (size, next_offset) = read_sleb128(data, current_offset)?;
        current_offset = next_offset;
        let typed_handler_count = size.unsigned_abs();
        let mut handler_entries = Vec::with_capacity(typed_handler_count as usize + 1);

        for _ in 0..typed_handler_count {
            let (type_index, next_offset) = read_uleb128(data, current_offset)?;
            current_offset = next_offset;
            let (address, next_offset) = read_uleb128(data, current_offset)?;
            current_offset = next_offset;
            let type_descriptor = type_descriptors
                .get(type_index as usize)
                .cloned()
                .ok_or_else(|| {
                    WinedroidError::new(format!(
                        "DEX catch handler type index {type_index} is outside the type table"
                    ))
                })?;

            handler_entries.push(DexExceptionHandler {
                type_descriptor: Some(type_descriptor),
                address,
            });
        }

        if size <= 0 {
            let (address, next_offset) = read_uleb128(data, current_offset)?;
            current_offset = next_offset;
            handler_entries.push(DexExceptionHandler {
                type_descriptor: None,
                address,
            });
        }

        handlers.push((handler_relative_offset, handler_entries));
    }

    Ok(handlers)
}

fn string_at(strings: &[String], index: u32) -> Result<&str> {
    strings
        .get(index as usize)
        .map(String::as_str)
        .ok_or_else(|| {
            WinedroidError::new(format!(
                "DEX string index {index} is outside the string table"
            ))
        })
}

fn type_descriptor_at(type_descriptors: &[String], index: u32) -> Result<&str> {
    type_descriptors
        .get(index as usize)
        .map(String::as_str)
        .ok_or_else(|| {
            WinedroidError::new(format!("DEX type index {index} is outside the type table"))
        })
}

fn proto_at(protos: &[DexProto], index: u32) -> Result<&DexProto> {
    protos.get(index as usize).ok_or_else(|| {
        WinedroidError::new(format!(
            "DEX proto index {index} is outside the proto table"
        ))
    })
}

fn read_string(data: &[u8], header: &DexHeader, string_index: u32) -> Result<String> {
    if string_index >= header.string_ids_size {
        return Err(WinedroidError::new(format!(
            "DEX string index {string_index} is outside the string table"
        )));
    }

    let string_id_offset = checked_table_item_offset(
        header.string_ids_off,
        string_index,
        STRING_ID_SIZE,
        "string id",
    )?;
    require_range(data, string_id_offset, STRING_ID_SIZE, "string id")?;

    let string_data_offset = read_u32(data, string_id_offset)? as usize;
    require_range(data, string_data_offset, 1, "string data")?;

    let (utf16_size, value_offset) = read_uleb128(data, string_data_offset)?;
    let value_end = find_null_byte(data, value_offset)?;
    decode_mutf8_string(&data[value_offset..value_end], utf16_size)
}

fn decode_mutf8_string(data: &[u8], expected_utf16_size: u32) -> Result<String> {
    let mut code_units = Vec::new();
    let mut offset = 0usize;

    while offset < data.len() {
        let byte = data[offset];

        if byte == 0 {
            return Err(WinedroidError::new(
                "DEX MUTF-8 string contains an unexpected null byte",
            ));
        }

        if byte & 0x80 == 0 {
            code_units.push(u16::from(byte));
            offset += 1;
            continue;
        }

        if byte & 0xe0 == 0xc0 {
            require_range(data, offset, 2, "DEX MUTF-8 two-byte sequence")?;
            let second = data[offset + 1];
            require_mutf8_continuation(second, offset + 1)?;

            let value = (u16::from(byte & 0x1f) << 6) | u16::from(second & 0x3f);
            code_units.push(value);
            offset += 2;
            continue;
        }

        if byte & 0xf0 == 0xe0 {
            require_range(data, offset, 3, "DEX MUTF-8 three-byte sequence")?;
            let second = data[offset + 1];
            let third = data[offset + 2];
            require_mutf8_continuation(second, offset + 1)?;
            require_mutf8_continuation(third, offset + 2)?;

            let value = (u16::from(byte & 0x0f) << 12)
                | (u16::from(second & 0x3f) << 6)
                | u16::from(third & 0x3f);
            code_units.push(value);
            offset += 3;
            continue;
        }

        return Err(WinedroidError::new(format!(
            "DEX MUTF-8 string contains unsupported byte 0x{byte:02x} at offset {offset}"
        )));
    }

    if code_units.len() != expected_utf16_size as usize {
        return Err(WinedroidError::new(format!(
            "DEX MUTF-8 string decoded to {} UTF-16 code unit(s), expected {expected_utf16_size}",
            code_units.len()
        )));
    }

    // cite: art utf.cc#ConvertModifiedUtf8ToUtf16
    Ok(String::from_utf16_lossy(&code_units))
}

fn require_mutf8_continuation(byte: u8, offset: usize) -> Result<()> {
    if byte & 0xc0 == 0x80 {
        return Ok(());
    }

    Err(WinedroidError::new(format!(
        "DEX MUTF-8 continuation byte at offset {offset} is invalid"
    )))
}

fn find_null_byte(data: &[u8], offset: usize) -> Result<usize> {
    for index in offset..data.len() {
        if data[index] == 0 {
            return Ok(index);
        }
    }

    Err(WinedroidError::new(format!(
        "DEX string at offset {offset} is not null-terminated"
    )))
}

#[derive(Debug, Clone)]
struct DexHeader {
    version: String,
    checksum: u32,
    file_size: u32,
    string_ids_size: u32,
    string_ids_off: u32,
    type_ids_size: u32,
    type_ids_off: u32,
    proto_ids_size: u32,
    proto_ids_off: u32,
    field_ids_size: u32,
    field_ids_off: u32,
    method_ids_size: u32,
    method_ids_off: u32,
    class_defs_size: u32,
    class_defs_off: u32,
}

impl DexHeader {
    fn parse(data: &[u8]) -> Result<Self> {
        require_range(data, 0, DEX_HEADER_SIZE as usize, "DEX header")?;
        validate_magic(data)?;

        let header_size = read_u32(data, 36)?;
        if header_size != DEX_HEADER_SIZE {
            return Err(WinedroidError::new(format!(
                "DEX header size is {header_size}, expected {DEX_HEADER_SIZE}"
            )));
        }

        let endian_tag = read_u32(data, 40)?;
        if endian_tag != DEX_ENDIAN_CONSTANT {
            return Err(WinedroidError::new(format!(
                "DEX endian tag is 0x{endian_tag:08x}, expected 0x{DEX_ENDIAN_CONSTANT:08x}"
            )));
        }

        let file_size = read_u32(data, 32)?;
        if file_size < DEX_HEADER_SIZE {
            return Err(WinedroidError::new(format!(
                "DEX declared file size {file_size} is smaller than the header"
            )));
        }

        if file_size as usize != data.len() {
            return Err(WinedroidError::new(format!(
                "DEX declared file size {file_size} does not match available data {}",
                data.len()
            )));
        }

        let header = Self {
            version: read_version(data)?,
            checksum: read_u32(data, 8)?,
            file_size,
            string_ids_size: read_u32(data, 56)?,
            string_ids_off: read_u32(data, 60)?,
            type_ids_size: read_u32(data, 64)?,
            type_ids_off: read_u32(data, 68)?,
            proto_ids_size: read_u32(data, 72)?,
            proto_ids_off: read_u32(data, 76)?,
            field_ids_size: read_u32(data, 80)?,
            field_ids_off: read_u32(data, 84)?,
            method_ids_size: read_u32(data, 88)?,
            method_ids_off: read_u32(data, 92)?,
            class_defs_size: read_u32(data, 96)?,
            class_defs_off: read_u32(data, 100)?,
        };

        validate_table(
            data,
            header.string_ids_size,
            header.string_ids_off,
            STRING_ID_SIZE,
            "string id table",
        )?;
        validate_table(
            data,
            header.type_ids_size,
            header.type_ids_off,
            TYPE_ID_SIZE,
            "type id table",
        )?;
        validate_table(
            data,
            header.proto_ids_size,
            header.proto_ids_off,
            PROTO_ID_SIZE,
            "proto id table",
        )?;
        validate_table(
            data,
            header.field_ids_size,
            header.field_ids_off,
            FIELD_ID_SIZE,
            "field id table",
        )?;
        validate_table(
            data,
            header.method_ids_size,
            header.method_ids_off,
            METHOD_ID_SIZE,
            "method id table",
        )?;
        validate_table(
            data,
            header.class_defs_size,
            header.class_defs_off,
            CLASS_DEF_SIZE,
            "class definition table",
        )?;

        Ok(header)
    }
}

fn validate_magic(data: &[u8]) -> Result<()> {
    if &data[0..4] != DEX_MAGIC_PREFIX || data[7] != DEX_MAGIC_SUFFIX {
        return Err(WinedroidError::new(
            "file does not start with a DEX magic header",
        ));
    }

    for byte in &data[4..7] {
        if !byte.is_ascii_digit() {
            return Err(WinedroidError::new("DEX version contains a non-digit byte"));
        }
    }

    Ok(())
}

fn read_version(data: &[u8]) -> Result<String> {
    std::str::from_utf8(&data[4..7])
        .map(str::to_owned)
        .map_err(|error| WinedroidError::with_source("DEX version is not valid UTF-8", error))
}

fn validate_table(
    data: &[u8],
    item_count: u32,
    table_offset: u32,
    item_size: usize,
    label: &str,
) -> Result<()> {
    if item_count == 0 {
        return Ok(());
    }

    if table_offset == 0 {
        return Err(WinedroidError::new(format!(
            "{label} has entries but offset is zero"
        )));
    }

    let table_size = (item_count as usize)
        .checked_mul(item_size)
        .ok_or_else(|| WinedroidError::new(format!("{label} size overflow")))?;

    require_range(data, table_offset as usize, table_size, label)
}

fn checked_table_item_offset(
    table_offset: u32,
    index: u32,
    item_size: usize,
    label: &str,
) -> Result<usize> {
    let byte_offset = (index as usize)
        .checked_mul(item_size)
        .ok_or_else(|| WinedroidError::new(format!("{label} offset overflow")))?;

    (table_offset as usize)
        .checked_add(byte_offset)
        .ok_or_else(|| WinedroidError::new(format!("{label} offset overflow")))
}

fn read_uleb128(data: &[u8], offset: usize) -> Result<(u32, usize)> {
    let mut value = 0u32;
    let mut shift = 0;

    for index in 0..5 {
        let current_offset = offset
            .checked_add(index)
            .ok_or_else(|| WinedroidError::new("ULEB128 offset overflow"))?;
        require_range(data, current_offset, 1, "ULEB128")?;

        let byte = data[current_offset];
        value |= u32::from(byte & 0x7f) << shift;

        if byte & 0x80 == 0 {
            return Ok((value, current_offset + 1));
        }

        shift += 7;
    }

    Err(WinedroidError::new(format!(
        "ULEB128 at offset {offset} is too large"
    )))
}

fn read_sleb128(data: &[u8], offset: usize) -> Result<(i32, usize)> {
    let mut value = 0i32;
    let mut shift = 0;

    for index in 0..5 {
        let current_offset = offset
            .checked_add(index)
            .ok_or_else(|| WinedroidError::new("SLEB128 offset overflow"))?;
        require_range(data, current_offset, 1, "SLEB128")?;

        let byte = data[current_offset];
        value |= i32::from(byte & 0x7f) << shift;
        shift += 7;

        if byte & 0x80 == 0 {
            if shift < 32 && byte & 0x40 != 0 {
                value |= !0 << shift;
            }

            return Ok((value, current_offset + 1));
        }
    }

    Err(WinedroidError::new(format!(
        "SLEB128 at offset {offset} is too large"
    )))
}

fn read_u16(data: &[u8], offset: usize) -> Result<u16> {
    require_range(data, offset, 2, "u16")?;
    Ok(u16::from_le_bytes([data[offset], data[offset + 1]]))
}

fn read_u32(data: &[u8], offset: usize) -> Result<u32> {
    require_range(data, offset, 4, "u32")?;
    Ok(u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]))
}

fn write_cache_u8(data: &mut Vec<u8>, value: u8) {
    data.push(value);
}

fn write_cache_u32(data: &mut Vec<u8>, value: u32) {
    data.extend_from_slice(&value.to_le_bytes());
}

fn write_cache_i32(data: &mut Vec<u8>, value: i32) {
    data.extend_from_slice(&value.to_le_bytes());
}

fn write_cache_i64(data: &mut Vec<u8>, value: i64) {
    data.extend_from_slice(&value.to_le_bytes());
}

fn write_cache_u64(data: &mut Vec<u8>, value: u64) {
    data.extend_from_slice(&value.to_le_bytes());
}

fn write_cache_bool(data: &mut Vec<u8>, value: bool) {
    write_cache_u8(data, u8::from(value));
}

fn write_cache_len(data: &mut Vec<u8>, len: usize) -> Result<()> {
    let len = u32::try_from(len)
        .map_err(|_| WinedroidError::new("DEX metadata cache length exceeds u32"))?;
    write_cache_u32(data, len);

    Ok(())
}

fn write_cache_string(data: &mut Vec<u8>, value: &str) -> Result<()> {
    write_cache_len(data, value.len())?;
    data.extend_from_slice(value.as_bytes());

    Ok(())
}

fn write_cache_string_option(data: &mut Vec<u8>, value: &Option<String>) -> Result<()> {
    write_cache_bool(data, value.is_some());
    if let Some(value) = value {
        write_cache_string(data, value)?;
    }

    Ok(())
}

fn write_cache_string_vec(data: &mut Vec<u8>, values: &[String]) -> Result<()> {
    write_cache_len(data, values.len())?;
    for value in values {
        write_cache_string(data, value)?;
    }

    Ok(())
}

fn write_cache_protos(data: &mut Vec<u8>, values: &[DexProto]) -> Result<()> {
    write_cache_len(data, values.len())?;
    for value in values {
        write_cache_string(data, &value.shorty)?;
        write_cache_string(data, &value.return_type)?;
        write_cache_string_vec(data, &value.parameters)?;
    }

    Ok(())
}

fn write_cache_fields(data: &mut Vec<u8>, values: &[DexFieldId]) -> Result<()> {
    write_cache_len(data, values.len())?;
    for value in values {
        write_cache_string(data, &value.class_descriptor)?;
        write_cache_string(data, &value.type_descriptor)?;
        write_cache_string(data, &value.name)?;
    }

    Ok(())
}

fn write_cache_methods(data: &mut Vec<u8>, values: &[DexMethodId]) -> Result<()> {
    write_cache_len(data, values.len())?;
    for value in values {
        write_cache_string(data, &value.class_descriptor)?;
        write_cache_string(data, &value.name)?;
        write_cache_string(data, &value.proto.shorty)?;
        write_cache_string(data, &value.proto.return_type)?;
        write_cache_string_vec(data, &value.proto.parameters)?;
    }

    Ok(())
}

fn write_cache_field_kind(data: &mut Vec<u8>, value: DexFieldKind) {
    write_cache_u8(
        data,
        match value {
            DexFieldKind::Static => 0,
            DexFieldKind::Instance => 1,
        },
    );
}

fn write_cache_method_kind(data: &mut Vec<u8>, value: DexMethodKind) {
    write_cache_u8(
        data,
        match value {
            DexMethodKind::Direct => 0,
            DexMethodKind::Virtual => 1,
        },
    );
}

fn write_cache_encoded_fields(data: &mut Vec<u8>, values: &[DexEncodedField]) -> Result<()> {
    write_cache_len(data, values.len())?;
    for value in values {
        write_cache_u32(data, value.field_index);
        write_cache_u32(data, value.access_flags);
        write_cache_field_kind(data, value.kind);
    }

    Ok(())
}

fn write_cache_encoded_methods(data: &mut Vec<u8>, values: &[DexEncodedMethod]) -> Result<()> {
    write_cache_len(data, values.len())?;
    for value in values {
        write_cache_u32(data, value.method_index);
        write_cache_u32(data, value.access_flags);
        write_cache_u32(data, value.code_offset);
        write_cache_method_kind(data, value.kind);
    }

    Ok(())
}

fn write_cache_static_values(data: &mut Vec<u8>, values: &[DexStaticValue]) -> Result<()> {
    write_cache_len(data, values.len())?;
    for value in values {
        match value {
            DexStaticValue::Int(value) => {
                write_cache_u8(data, 0);
                write_cache_i32(data, *value);
            }
            DexStaticValue::Long(value) => {
                write_cache_u8(data, 1);
                write_cache_i64(data, *value);
            }
            DexStaticValue::FloatBits(value) => {
                write_cache_u8(data, 2);
                write_cache_u32(data, *value);
            }
            DexStaticValue::DoubleBits(value) => {
                write_cache_u8(data, 3);
                write_cache_u64(data, *value);
            }
            DexStaticValue::String(value) => {
                write_cache_u8(data, 4);
                write_cache_string(data, value)?;
            }
            DexStaticValue::Null => write_cache_u8(data, 5),
            DexStaticValue::Boolean(value) => {
                write_cache_u8(data, 6);
                write_cache_bool(data, *value);
            }
            DexStaticValue::Unsupported => write_cache_u8(data, 7),
        }
    }

    Ok(())
}

fn write_cache_classes(data: &mut Vec<u8>, values: &[DexClassDef]) -> Result<()> {
    write_cache_len(data, values.len())?;
    for value in values {
        write_cache_string(data, &value.descriptor)?;
        write_cache_string_option(data, &value.superclass_descriptor)?;
        write_cache_string_vec(data, &value.interface_descriptors)?;
        write_cache_u32(data, value.access_flags);
        write_cache_u32(data, value.annotations_offset);
        write_cache_u32(data, value.class_data_offset);
        write_cache_encoded_fields(data, &value.static_fields)?;
        write_cache_encoded_fields(data, &value.instance_fields)?;
        write_cache_encoded_methods(data, &value.direct_methods)?;
        write_cache_encoded_methods(data, &value.virtual_methods)?;
        write_cache_static_values(data, &value.static_values)?;
    }

    Ok(())
}

fn dex_metadata_cache_checksum(data: &[u8]) -> u64 {
    let mut hash = DEX_METADATA_CACHE_FNV_OFFSET;
    for byte in data {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(DEX_METADATA_CACHE_FNV_PRIME);
    }

    hash
}

fn require_cache_count(source_name: &str, label: &str, actual: usize, expected: u32) -> Result<()> {
    if actual == expected as usize {
        return Ok(());
    }

    Err(WinedroidError::new(format!(
        "DEX metadata cache for `{source_name}` has {actual} {label} record(s), expected {expected}"
    )))
}

struct DexMetadataCacheReader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> DexMetadataCacheReader<'a> {
    fn new(data: &'a [u8]) -> Result<Self> {
        require_range(
            data,
            0,
            DEX_METADATA_CACHE_MAGIC.len() + 8,
            "DEX metadata cache header",
        )?;
        if &data[..DEX_METADATA_CACHE_MAGIC.len()] != DEX_METADATA_CACHE_MAGIC {
            return Err(WinedroidError::new(
                "DEX metadata cache has an unsupported magic header",
            ));
        }
        let checksum_offset = DEX_METADATA_CACHE_MAGIC.len();
        let expected = u64::from_le_bytes([
            data[checksum_offset],
            data[checksum_offset + 1],
            data[checksum_offset + 2],
            data[checksum_offset + 3],
            data[checksum_offset + 4],
            data[checksum_offset + 5],
            data[checksum_offset + 6],
            data[checksum_offset + 7],
        ]);
        let payload_offset = checksum_offset + 8;
        let actual = dex_metadata_cache_checksum(&data[payload_offset..]);
        if actual != expected {
            return Err(WinedroidError::new(
                "DEX metadata cache checksum does not match its payload",
            ));
        }

        Ok(Self {
            data,
            offset: payload_offset,
        })
    }

    fn read_exact(&mut self, length: usize, label: &str) -> Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or_else(|| WinedroidError::new(format!("{label} offset overflow")))?;
        let slice = self.data.get(self.offset..end).ok_or_else(|| {
            WinedroidError::new(format!("DEX metadata cache {label} extends past end"))
        })?;
        self.offset = end;

        Ok(slice)
    }

    fn read_u8(&mut self) -> Result<u8> {
        Ok(self.read_exact(1, "u8")?[0])
    }

    fn read_bool(&mut self) -> Result<bool> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            other => Err(WinedroidError::new(format!(
                "DEX metadata cache boolean tag {other} is invalid"
            ))),
        }
    }

    fn read_u32(&mut self) -> Result<u32> {
        let bytes = self.read_exact(4, "u32")?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_i32(&mut self) -> Result<i32> {
        let bytes = self.read_exact(4, "i32")?;
        Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_i64(&mut self) -> Result<i64> {
        let bytes = self.read_exact(8, "i64")?;
        Ok(i64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn read_u64(&mut self) -> Result<u64> {
        let bytes = self.read_exact(8, "u64")?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn read_len(&mut self) -> Result<usize> {
        Ok(self.read_u32()? as usize)
    }

    fn read_string(&mut self) -> Result<String> {
        let len = self.read_len()?;
        let bytes = self.read_exact(len, "string")?;
        String::from_utf8(bytes.to_vec()).map_err(|error| {
            WinedroidError::with_source("DEX metadata cache string is not valid UTF-8", error)
        })
    }

    fn read_string_option(&mut self) -> Result<Option<String>> {
        if self.read_bool()? {
            Ok(Some(self.read_string()?))
        } else {
            Ok(None)
        }
    }

    fn read_string_vec(&mut self) -> Result<Vec<String>> {
        let len = self.read_len()?;
        let mut values = Vec::with_capacity(len);
        for _ in 0..len {
            values.push(self.read_string()?);
        }

        Ok(values)
    }

    fn read_protos(&mut self) -> Result<Vec<DexProto>> {
        let len = self.read_len()?;
        let mut values = Vec::with_capacity(len);
        for _ in 0..len {
            values.push(DexProto {
                shorty: self.read_string()?,
                return_type: self.read_string()?,
                parameters: self.read_string_vec()?,
            });
        }

        Ok(values)
    }

    fn read_fields(&mut self) -> Result<Vec<DexFieldId>> {
        let len = self.read_len()?;
        let mut values = Vec::with_capacity(len);
        for _ in 0..len {
            values.push(DexFieldId {
                class_descriptor: self.read_string()?,
                type_descriptor: self.read_string()?,
                name: self.read_string()?,
            });
        }

        Ok(values)
    }

    fn read_methods(&mut self) -> Result<Vec<DexMethodId>> {
        let len = self.read_len()?;
        let mut values = Vec::with_capacity(len);
        for _ in 0..len {
            let class_descriptor = self.read_string()?;
            let name = self.read_string()?;
            let shorty = self.read_string()?;
            let return_type = self.read_string()?;
            let parameters = self.read_string_vec()?;
            values.push(DexMethodId {
                class_descriptor,
                name,
                proto: DexProto {
                    shorty,
                    return_type,
                    parameters,
                },
            });
        }

        Ok(values)
    }

    fn read_field_kind(&mut self) -> Result<DexFieldKind> {
        match self.read_u8()? {
            0 => Ok(DexFieldKind::Static),
            1 => Ok(DexFieldKind::Instance),
            other => Err(WinedroidError::new(format!(
                "DEX metadata cache field kind tag {other} is invalid"
            ))),
        }
    }

    fn read_method_kind(&mut self) -> Result<DexMethodKind> {
        match self.read_u8()? {
            0 => Ok(DexMethodKind::Direct),
            1 => Ok(DexMethodKind::Virtual),
            other => Err(WinedroidError::new(format!(
                "DEX metadata cache method kind tag {other} is invalid"
            ))),
        }
    }

    fn read_encoded_fields(&mut self) -> Result<Vec<DexEncodedField>> {
        let len = self.read_len()?;
        let mut values = Vec::with_capacity(len);
        for _ in 0..len {
            let field_index = self.read_u32()?;
            let access_flags = self.read_u32()?;
            let kind = self.read_field_kind()?;
            values.push(DexEncodedField {
                field_index,
                access_flags,
                kind,
            });
        }

        Ok(values)
    }

    fn read_encoded_methods(&mut self) -> Result<Vec<DexEncodedMethod>> {
        let len = self.read_len()?;
        let mut values = Vec::with_capacity(len);
        for _ in 0..len {
            let method_index = self.read_u32()?;
            let access_flags = self.read_u32()?;
            let code_offset = self.read_u32()?;
            let kind = self.read_method_kind()?;
            values.push(DexEncodedMethod {
                method_index,
                access_flags,
                code_offset,
                kind,
            });
        }

        Ok(values)
    }

    fn read_static_values(&mut self) -> Result<Vec<DexStaticValue>> {
        let len = self.read_len()?;
        let mut values = Vec::with_capacity(len);
        for _ in 0..len {
            let tag = self.read_u8()?;
            values.push(match tag {
                0 => DexStaticValue::Int(self.read_i32()?),
                1 => DexStaticValue::Long(self.read_i64()?),
                2 => DexStaticValue::FloatBits(self.read_u32()?),
                3 => DexStaticValue::DoubleBits(self.read_u64()?),
                4 => DexStaticValue::String(self.read_string()?),
                5 => DexStaticValue::Null,
                6 => DexStaticValue::Boolean(self.read_bool()?),
                7 => DexStaticValue::Unsupported,
                other => {
                    return Err(WinedroidError::new(format!(
                        "DEX metadata cache static value tag {other} is invalid"
                    )));
                }
            });
        }

        Ok(values)
    }

    fn read_classes(&mut self) -> Result<Vec<DexClassDef>> {
        let len = self.read_len()?;
        let mut values = Vec::with_capacity(len);
        for _ in 0..len {
            values.push(DexClassDef {
                descriptor: self.read_string()?,
                superclass_descriptor: self.read_string_option()?,
                interface_descriptors: self.read_string_vec()?,
                access_flags: self.read_u32()?,
                annotations_offset: self.read_u32()?,
                class_data_offset: self.read_u32()?,
                static_fields: self.read_encoded_fields()?,
                instance_fields: self.read_encoded_fields()?,
                direct_methods: self.read_encoded_methods()?,
                virtual_methods: self.read_encoded_methods()?,
                static_values: self.read_static_values()?,
            });
        }

        Ok(values)
    }

    fn require_finished(&self) -> Result<()> {
        if self.offset == self.data.len() {
            return Ok(());
        }

        Err(WinedroidError::new(format!(
            "DEX metadata cache has {} trailing byte(s)",
            self.data.len() - self.offset
        )))
    }
}

fn require_range(data: &[u8], offset: usize, length: usize, label: &str) -> Result<()> {
    let end = offset
        .checked_add(length)
        .ok_or_else(|| WinedroidError::new(format!("{label} range overflow")))?;

    if end > data.len() {
        return Err(WinedroidError::new(format!(
            "{label} at offset {offset} extends past the end of the file"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_dex_class_descriptor() {
        let dex = minimal_dex("Lcom/example/MainActivity;");
        let summary = parse_dex(&dex).expect("DEX should parse");

        assert_eq!(summary.version, "035");
        assert_eq!(summary.string_count, 1);
        assert_eq!(summary.type_count, 1);
        assert_eq!(summary.class_count, 1);
        assert_eq!(
            summary.class_descriptors,
            vec!["Lcom/example/MainActivity;".to_owned()]
        );
    }

    #[test]
    fn parses_encoded_values_per_dex_format_value_types() {
        // cite: art dex_file.h#kDexAnnotationByte, kDexAnnotationValueTypeMask, kDexAnnotationValueArgShift
        let strings = vec!["hello".to_owned()];

        let (value, next) = read_encoded_value(&[0x04, 0x2a], 0, &strings).expect("int");
        assert_eq!(value, DexStaticValue::Int(42));
        assert_eq!(next, 2);

        let (value, _) = read_encoded_value(&[0x24, 0xff, 0xff], 0, &strings).expect("int");
        assert_eq!(value, DexStaticValue::Int(-1));

        let (value, _) = read_encoded_value(&[0x00, 0x80], 0, &strings).expect("byte");
        assert_eq!(value, DexStaticValue::Int(-128));

        let (value, _) = read_encoded_value(&[0x06, 0xfe], 0, &strings).expect("long");
        assert_eq!(value, DexStaticValue::Long(-2));

        let (value, _) = read_encoded_value(&[0x30, 0x40, 0x49], 0, &strings).expect("float");
        assert_eq!(value, DexStaticValue::FloatBits(0x4940_0000));

        let (value, _) = read_encoded_value(&[0x17, 0x00], 0, &strings).expect("string");
        assert_eq!(value, DexStaticValue::String("hello".to_owned()));

        let (value, next) = read_encoded_value(&[0x1e], 0, &strings).expect("null");
        assert_eq!(value, DexStaticValue::Null);
        assert_eq!(next, 1);

        let (value, next) = read_encoded_value(&[0x3f], 0, &strings).expect("boolean");
        assert_eq!(value, DexStaticValue::Boolean(true));
        assert_eq!(next, 1);

        let (value, next) =
            read_encoded_value(&[0x1c, 0x02, 0x04, 0x01, 0x1e], 0, &strings).expect("array");
        assert_eq!(value, DexStaticValue::Unsupported);
        assert_eq!(next, 5);
    }

    #[test]
    fn rejects_invalid_magic() {
        let mut dex = minimal_dex("Lcom/example/MainActivity;");
        dex[0] = b'x';

        let error = parse_dex(&dex).expect_err("invalid magic should fail");

        assert!(
            error.to_string().contains("DEX magic"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn decodes_mutf8_encoded_null() {
        let value = decode_mutf8_string(&[0xc0, 0x80], 1).expect("string should decode");

        assert_eq!(value, "\0");
    }

    #[test]
    fn decodes_mutf8_surrogate_pair() {
        let value = decode_mutf8_string(&[0xed, 0xa0, 0xbd, 0xed, 0xb8, 0x80], 2)
            .expect("string should decode");

        assert_eq!(value, "😀");
    }

    #[test]
    fn parses_method_and_code_item() {
        let dex = minimal_method_dex();
        let file = parse_dex_file("classes.dex", dex).expect("DEX should parse");
        let class_def = file
            .find_class("Lcom/example/Test;")
            .expect("class should exist");
        let encoded_method = &class_def.direct_methods[0];
        let field = file.field(0).expect("field should exist");
        let method = file
            .method(encoded_method.method_index)
            .expect("method should exist");
        let code = file
            .parse_code_item(encoded_method.code_offset)
            .expect("code item should parse");

        assert_eq!(class_def.superclass_descriptor, None);
        assert_eq!(class_def.interface_descriptors, Vec::<String>::new());
        assert_eq!(class_def.access_flags, 1);
        assert_eq!(class_def.static_fields, Vec::<DexEncodedField>::new());
        assert_eq!(class_def.instance_fields, Vec::<DexEncodedField>::new());
        assert_eq!(field.display_signature(), "Lcom/example/Test;.value:I");
        assert_eq!(method.display_signature(), "answer()I");
        assert_eq!(code.registers_size, 1);
        assert_eq!(code.instructions, vec![0x7012, 0x000f]);
    }

    #[test]
    fn dex_metadata_cache_roundtrips_parsed_tables_and_raw_code() {
        let dex = minimal_method_dex();
        let parsed = parse_dex_file("classes.dex", dex.clone()).expect("DEX should parse");
        let cache = encode_dex_metadata_cache(&parsed).expect("cache should encode");

        let cached = parse_dex_file_with_metadata_cache("classes.dex", dex, &cache)
            .expect("cache should decode");
        let class_def = cached
            .find_class("Lcom/example/Test;")
            .expect("class should exist");
        let code = cached
            .parse_code_item(class_def.direct_methods[0].code_offset)
            .expect("code item should parse");

        assert_eq!(cached, parsed);
        assert_eq!(code.instructions, vec![0x7012, 0x000f]);
    }

    #[test]
    fn dex_metadata_cache_preserves_raw_annotation_reads() {
        let dex = minimal_annotated_class_dex();
        let parsed = parse_dex_file("classes.dex", dex.clone()).expect("DEX should parse");
        let cache = encode_dex_metadata_cache(&parsed).expect("cache should encode");
        let cached = parse_dex_file_with_metadata_cache("classes.dex", dex, &cache)
            .expect("cache should decode");
        let class_def = cached
            .find_class("Lcom/example/Test;")
            .expect("class should exist");
        let annotations = cached
            .class_annotations(class_def)
            .expect("annotations should parse");

        assert_eq!(annotations[0].type_descriptor, "Lcom/example/Name;");
        assert_eq!(
            annotations[0].elements[0].value,
            DexAnnotationValue::String("hello".to_owned())
        );
    }

    #[test]
    fn dex_metadata_cache_rejects_stale_dex_header() {
        let mut dex = minimal_method_dex();
        let parsed = parse_dex_file("classes.dex", dex.clone()).expect("DEX should parse");
        let cache = encode_dex_metadata_cache(&parsed).expect("cache should encode");
        dex[8] = 1;

        let error = parse_dex_file_with_metadata_cache("classes.dex", dex, &cache)
            .expect_err("stale cache should fail");

        assert!(
            error.to_string().contains("does not match"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn dex_metadata_cache_rejects_payload_corruption() {
        let dex = minimal_method_dex();
        let parsed = parse_dex_file("classes.dex", dex.clone()).expect("DEX should parse");
        let mut cache = encode_dex_metadata_cache(&parsed).expect("cache should encode");
        let last = cache.last_mut().expect("cache payload");
        *last ^= 1;

        let error = parse_dex_file_with_metadata_cache("classes.dex", dex, &cache)
            .expect_err("corrupt cache should fail");

        assert!(
            error.to_string().contains("checksum"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn parses_class_data_fields() {
        let dex = minimal_field_class_dex();
        let file = parse_dex_file("classes.dex", dex).expect("DEX should parse");
        let class_def = file
            .find_class("Lcom/example/Test;")
            .expect("class should exist");
        let static_field = file
            .field(class_def.static_fields[0].field_index)
            .expect("static field should exist");
        let instance_field = file
            .field(class_def.instance_fields[0].field_index)
            .expect("instance field should exist");

        assert_eq!(class_def.static_fields.len(), 1);
        assert_eq!(class_def.static_fields[0].access_flags, 0x0009);
        assert_eq!(class_def.static_fields[0].kind, DexFieldKind::Static);
        assert_eq!(
            static_field.display_signature(),
            "Lcom/example/Test;.staticValue:I"
        );
        assert_eq!(class_def.instance_fields.len(), 1);
        assert_eq!(class_def.instance_fields[0].access_flags, 0x0002);
        assert_eq!(class_def.instance_fields[0].kind, DexFieldKind::Instance);
        assert_eq!(
            instance_field.display_signature(),
            "Lcom/example/Test;.value:I"
        );
    }

    #[test]
    fn parses_class_interface_descriptors() {
        let dex = minimal_interface_dex();
        let file = parse_dex_file("classes.dex", dex).expect("DEX should parse");
        let class_def = file
            .find_class("Lcom/example/Test;")
            .expect("class should exist");

        assert_eq!(
            class_def.interface_descriptors,
            vec!["Lcom/example/Marker;".to_owned()]
        );
    }

    // cite: art dex_file_structs.h#AnnotationsDirectoryItem, AnnotationSetItem, AnnotationItem
    // cite: art dex_file.h#kDexVisibilityRuntime, kDexAnnotationString
    #[test]
    fn parses_runtime_class_annotation_with_string_element() {
        let dex = minimal_annotated_class_dex();
        let file = parse_dex_file("classes.dex", dex).expect("DEX should parse");
        let class_def = file
            .find_class("Lcom/example/Test;")
            .expect("class should exist");
        let annotations = file
            .class_annotations(class_def)
            .expect("annotations should parse");

        assert_eq!(annotations.len(), 1);
        assert_eq!(annotations[0].visibility, DEX_VISIBILITY_RUNTIME);
        assert_eq!(annotations[0].type_descriptor, "Lcom/example/Name;");
        assert_eq!(annotations[0].elements.len(), 1);
        assert_eq!(annotations[0].elements[0].name, "value");
        assert_eq!(
            annotations[0].elements[0].value,
            DexAnnotationValue::String("hello".to_owned())
        );
    }

    fn minimal_annotated_class_dex() -> Vec<u8> {
        let header_size = DEX_HEADER_SIZE as usize;
        let string_ids_off = header_size;
        let type_ids_off = string_ids_off + STRING_ID_SIZE * 4;
        let class_defs_off = type_ids_off + TYPE_ID_SIZE * 2;
        let data_off = class_defs_off + CLASS_DEF_SIZE;
        let mut data = vec![0; data_off];
        let strings = ["Lcom/example/Test;", "Lcom/example/Name;", "value", "hello"];
        let mut string_offsets = Vec::new();

        for value in strings {
            string_offsets.push(data.len() as u32);
            write_uleb128(&mut data, value.len() as u32);
            data.extend_from_slice(value.as_bytes());
            data.push(0);
        }

        let annotation_item_off = data.len() as u32;
        data.push(DEX_VISIBILITY_RUNTIME);
        write_uleb128(&mut data, 1);
        write_uleb128(&mut data, 1);
        write_uleb128(&mut data, 2);
        data.push(DEX_ANNOTATION_STRING);
        data.push(3);

        while data.len() % 4 != 0 {
            data.push(0);
        }
        let annotation_set_off = data.len() as u32;
        data.extend_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(&annotation_item_off.to_le_bytes());

        let annotations_directory_off = data.len() as u32;
        data.extend_from_slice(&annotation_set_off.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());

        let file_size = data.len();
        data[0..8].copy_from_slice(b"dex\n035\0");
        write_u32(&mut data, 32, file_size as u32);
        write_u32(&mut data, 36, DEX_HEADER_SIZE);
        write_u32(&mut data, 40, DEX_ENDIAN_CONSTANT);
        write_u32(&mut data, 56, 4);
        write_u32(&mut data, 60, string_ids_off as u32);
        write_u32(&mut data, 64, 2);
        write_u32(&mut data, 68, type_ids_off as u32);
        write_u32(&mut data, 96, 1);
        write_u32(&mut data, 100, class_defs_off as u32);
        write_u32(&mut data, 104, data_off as u32);
        write_u32(&mut data, 108, (file_size - data_off) as u32);
        for (index, offset) in string_offsets.iter().enumerate() {
            write_u32(&mut data, string_ids_off + index * STRING_ID_SIZE, *offset);
        }
        write_u32(&mut data, type_ids_off, 0);
        write_u32(&mut data, type_ids_off + TYPE_ID_SIZE, 1);
        write_u32(&mut data, class_defs_off, 0);
        write_u32(&mut data, class_defs_off + 4, 1);
        write_u32(&mut data, class_defs_off + 8, u32::MAX);
        write_u32(&mut data, class_defs_off + 20, annotations_directory_off);

        data
    }

    fn minimal_dex(descriptor: &str) -> Vec<u8> {
        let header_size = DEX_HEADER_SIZE as usize;
        let string_ids_off = header_size;
        let type_ids_off = string_ids_off + STRING_ID_SIZE;
        let class_defs_off = type_ids_off + TYPE_ID_SIZE;
        let string_data_off = class_defs_off + CLASS_DEF_SIZE;
        let file_size = string_data_off + 1 + descriptor.len() + 1;
        let mut data = vec![0; file_size];

        data[0..8].copy_from_slice(b"dex\n035\0");
        write_u32(&mut data, 32, file_size as u32);
        write_u32(&mut data, 36, DEX_HEADER_SIZE);
        write_u32(&mut data, 40, DEX_ENDIAN_CONSTANT);
        write_u32(&mut data, 56, 1);
        write_u32(&mut data, 60, string_ids_off as u32);
        write_u32(&mut data, 64, 1);
        write_u32(&mut data, 68, type_ids_off as u32);
        write_u32(&mut data, 96, 1);
        write_u32(&mut data, 100, class_defs_off as u32);
        write_u32(&mut data, 104, string_data_off as u32);
        write_u32(&mut data, 108, (file_size - string_data_off) as u32);
        write_u32(&mut data, string_ids_off, string_data_off as u32);
        write_u32(&mut data, type_ids_off, 0);
        write_u32(&mut data, class_defs_off, 0);
        write_u32(&mut data, class_defs_off + 8, u32::MAX);

        data[string_data_off] = descriptor.len() as u8;
        data[string_data_off + 1..string_data_off + 1 + descriptor.len()]
            .copy_from_slice(descriptor.as_bytes());

        data
    }

    fn minimal_field_class_dex() -> Vec<u8> {
        let header_size = DEX_HEADER_SIZE as usize;
        let string_ids_off = header_size;
        let type_ids_off = string_ids_off + STRING_ID_SIZE * 4;
        let field_ids_off = type_ids_off + TYPE_ID_SIZE * 2;
        let class_defs_off = field_ids_off + FIELD_ID_SIZE * 2;
        let data_off = class_defs_off + CLASS_DEF_SIZE;
        let mut data = vec![0; data_off];
        let strings = ["Lcom/example/Test;", "I", "staticValue", "value"];
        let mut string_offsets = Vec::new();

        for value in strings {
            string_offsets.push(data.len() as u32);
            write_uleb128(&mut data, value.len() as u32);
            data.extend_from_slice(value.as_bytes());
            data.push(0);
        }

        let class_data_off = data.len() as u32;
        write_uleb128(&mut data, 1);
        write_uleb128(&mut data, 1);
        write_uleb128(&mut data, 0);
        write_uleb128(&mut data, 0);
        write_uleb128(&mut data, 0);
        write_uleb128(&mut data, 0x0009);
        write_uleb128(&mut data, 1);
        write_uleb128(&mut data, 0x0002);

        let file_size = data.len() as u32;
        data[0..8].copy_from_slice(b"dex\n035\0");
        write_u32(&mut data, 32, file_size);
        write_u32(&mut data, 36, DEX_HEADER_SIZE);
        write_u32(&mut data, 40, DEX_ENDIAN_CONSTANT);
        write_u32(&mut data, 56, 4);
        write_u32(&mut data, 60, string_ids_off as u32);
        write_u32(&mut data, 64, 2);
        write_u32(&mut data, 68, type_ids_off as u32);
        write_u32(&mut data, 80, 2);
        write_u32(&mut data, 84, field_ids_off as u32);
        write_u32(&mut data, 96, 1);
        write_u32(&mut data, 100, class_defs_off as u32);
        write_u32(&mut data, 104, file_size - data_off as u32);
        write_u32(&mut data, 108, data_off as u32);

        for (index, offset) in string_offsets.iter().enumerate() {
            write_u32(&mut data, string_ids_off + index * STRING_ID_SIZE, *offset);
        }

        write_u32(&mut data, type_ids_off, 0);
        write_u32(&mut data, type_ids_off + TYPE_ID_SIZE, 1);

        write_u16(&mut data, field_ids_off, 0);
        write_u16(&mut data, field_ids_off + 2, 1);
        write_u32(&mut data, field_ids_off + 4, 2);

        write_u16(&mut data, field_ids_off + FIELD_ID_SIZE, 0);
        write_u16(&mut data, field_ids_off + FIELD_ID_SIZE + 2, 1);
        write_u32(&mut data, field_ids_off + FIELD_ID_SIZE + 4, 3);

        write_u32(&mut data, class_defs_off, 0);
        write_u32(&mut data, class_defs_off + 4, 1);
        write_u32(&mut data, class_defs_off + 8, u32::MAX);
        write_u32(&mut data, class_defs_off + 16, u32::MAX);
        write_u32(&mut data, class_defs_off + 24, class_data_off);

        data
    }

    fn minimal_method_dex() -> Vec<u8> {
        let header_size = DEX_HEADER_SIZE as usize;
        let string_ids_off = header_size;
        let type_ids_off = string_ids_off + STRING_ID_SIZE * 4;
        let proto_ids_off = type_ids_off + TYPE_ID_SIZE * 2;
        let field_ids_off = proto_ids_off + PROTO_ID_SIZE;
        let method_ids_off = field_ids_off + FIELD_ID_SIZE;
        let class_defs_off = method_ids_off + METHOD_ID_SIZE;
        let data_off = class_defs_off + CLASS_DEF_SIZE;
        let mut data = vec![0; data_off];
        let strings = ["Lcom/example/Test;", "I", "answer", "value"];
        let mut string_offsets = Vec::new();

        for value in strings {
            string_offsets.push(data.len() as u32);
            write_uleb128(&mut data, value.len() as u32);
            data.extend_from_slice(value.as_bytes());
            data.push(0);
        }

        let class_data_off = data.len() as u32;
        let code_off = align_to_four(class_data_off as usize + 8) as u32;

        data.extend_from_slice(&[0, 0, 1, 0, 0, 0x09]);
        write_uleb128(&mut data, code_off);

        while data.len() < code_off as usize {
            data.push(0);
        }

        write_code_item(&mut data);

        let file_size = data.len() as u32;
        data[0..8].copy_from_slice(b"dex\n035\0");
        write_u32(&mut data, 32, file_size);
        write_u32(&mut data, 36, DEX_HEADER_SIZE);
        write_u32(&mut data, 40, DEX_ENDIAN_CONSTANT);
        write_u32(&mut data, 56, 4);
        write_u32(&mut data, 60, string_ids_off as u32);
        write_u32(&mut data, 64, 2);
        write_u32(&mut data, 68, type_ids_off as u32);
        write_u32(&mut data, 72, 1);
        write_u32(&mut data, 76, proto_ids_off as u32);
        write_u32(&mut data, 80, 1);
        write_u32(&mut data, 84, field_ids_off as u32);
        write_u32(&mut data, 88, 1);
        write_u32(&mut data, 92, method_ids_off as u32);
        write_u32(&mut data, 96, 1);
        write_u32(&mut data, 100, class_defs_off as u32);
        write_u32(&mut data, 104, file_size - data_off as u32);
        write_u32(&mut data, 108, data_off as u32);

        for (index, offset) in string_offsets.iter().enumerate() {
            write_u32(&mut data, string_ids_off + index * STRING_ID_SIZE, *offset);
        }

        write_u32(&mut data, type_ids_off, 0);
        write_u32(&mut data, type_ids_off + TYPE_ID_SIZE, 1);

        write_u32(&mut data, proto_ids_off, 1);
        write_u32(&mut data, proto_ids_off + 4, 1);
        write_u32(&mut data, proto_ids_off + 8, 0);

        write_u16(&mut data, field_ids_off, 0);
        write_u16(&mut data, field_ids_off + 2, 1);
        write_u32(&mut data, field_ids_off + 4, 3);

        write_u16(&mut data, method_ids_off, 0);
        write_u16(&mut data, method_ids_off + 2, 0);
        write_u32(&mut data, method_ids_off + 4, 2);

        write_u32(&mut data, class_defs_off, 0);
        write_u32(&mut data, class_defs_off + 4, 1);
        write_u32(&mut data, class_defs_off + 8, u32::MAX);
        write_u32(&mut data, class_defs_off + 16, u32::MAX);
        write_u32(&mut data, class_defs_off + 24, class_data_off);

        data
    }

    fn minimal_interface_dex() -> Vec<u8> {
        let header_size = DEX_HEADER_SIZE as usize;
        let string_ids_off = header_size;
        let type_ids_off = string_ids_off + STRING_ID_SIZE * 2;
        let class_defs_off = type_ids_off + TYPE_ID_SIZE * 2;
        let data_off = class_defs_off + CLASS_DEF_SIZE;
        let mut data = vec![0; data_off];
        let strings = ["Lcom/example/Test;", "Lcom/example/Marker;"];

        let interfaces_off = data.len() as u32;
        data.extend_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(&1u16.to_le_bytes());

        let mut string_offsets = Vec::new();
        for value in strings {
            string_offsets.push(data.len() as u32);
            write_uleb128(&mut data, value.len() as u32);
            data.extend_from_slice(value.as_bytes());
            data.push(0);
        }

        let file_size = data.len() as u32;
        data[0..8].copy_from_slice(b"dex\n035\0");
        write_u32(&mut data, 32, file_size);
        write_u32(&mut data, 36, DEX_HEADER_SIZE);
        write_u32(&mut data, 40, DEX_ENDIAN_CONSTANT);
        write_u32(&mut data, 56, 2);
        write_u32(&mut data, 60, string_ids_off as u32);
        write_u32(&mut data, 64, 2);
        write_u32(&mut data, 68, type_ids_off as u32);
        write_u32(&mut data, 96, 1);
        write_u32(&mut data, 100, class_defs_off as u32);
        write_u32(&mut data, 104, file_size - data_off as u32);
        write_u32(&mut data, 108, data_off as u32);

        for (index, offset) in string_offsets.iter().enumerate() {
            write_u32(&mut data, string_ids_off + index * STRING_ID_SIZE, *offset);
            write_u32(&mut data, type_ids_off + index * TYPE_ID_SIZE, index as u32);
        }

        write_u32(&mut data, class_defs_off, 0);
        write_u32(&mut data, class_defs_off + 8, u32::MAX);
        write_u32(&mut data, class_defs_off + 12, interfaces_off);
        write_u32(&mut data, class_defs_off + 16, u32::MAX);

        data
    }

    fn write_code_item(data: &mut Vec<u8>) {
        data.extend_from_slice(&1u16.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&2u32.to_le_bytes());
        data.extend_from_slice(&0x7012u16.to_le_bytes());
        data.extend_from_slice(&0x000fu16.to_le_bytes());
    }

    fn write_uleb128(data: &mut Vec<u8>, mut value: u32) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;

            if value != 0 {
                byte |= 0x80;
            }

            data.push(byte);

            if value == 0 {
                return;
            }
        }
    }

    fn align_to_four(value: usize) -> usize {
        (value + 3) & !3
    }

    fn write_u16(data: &mut [u8], offset: usize, value: u16) {
        data[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn write_u32(data: &mut [u8], offset: usize, value: u32) {
        data[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}
