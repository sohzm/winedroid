use std::fmt;

use crate::error::{Result, WinedroidError};

const RES_STRING_POOL_TYPE: u16 = 0x0001;
const RES_XML_TYPE: u16 = 0x0003;
const RES_XML_START_NAMESPACE_TYPE: u16 = 0x0100;
const RES_XML_END_NAMESPACE_TYPE: u16 = 0x0101;
const RES_XML_START_ELEMENT_TYPE: u16 = 0x0102;
const RES_XML_END_ELEMENT_TYPE: u16 = 0x0103;
const RES_XML_CDATA_TYPE: u16 = 0x0104;
const RES_XML_RESOURCE_MAP_TYPE: u16 = 0x0180;

const UTF8_FLAG: u32 = 0x0000_0100;
const NO_INDEX: u32 = 0xffff_ffff;

const TYPE_NULL: u8 = 0x00;
const TYPE_REFERENCE: u8 = 0x01;
const TYPE_ATTRIBUTE: u8 = 0x02;
const TYPE_STRING: u8 = 0x03;
const TYPE_FLOAT: u8 = 0x04;
const TYPE_DIMENSION: u8 = 0x05;
const TYPE_FRACTION: u8 = 0x06;
const TYPE_INT_DEC: u8 = 0x10;
const TYPE_INT_HEX: u8 = 0x11;
const TYPE_INT_BOOLEAN: u8 = 0x12;
const TYPE_FIRST_COLOR_INT: u8 = 0x1c;
const TYPE_LAST_COLOR_INT: u8 = 0x1f;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XmlEvent {
    Start(XmlStartElement),
    End(XmlEndElement),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlStartElement {
    pub name: String,
    pub namespace: Option<String>,
    pub attributes: Vec<XmlAttribute>,
}

impl XmlStartElement {
    pub fn attribute(&self, name: &str) -> Option<&XmlAttribute> {
        self.attributes
            .iter()
            .find(|attribute| attribute.name == name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlEndElement {
    pub name: String,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlAttribute {
    pub name: String,
    pub namespace: Option<String>,
    pub value: XmlValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XmlValue {
    Null,
    String(String),
    Boolean(bool),
    Integer(i32),
    Hex(u32),
    Reference(u32),
    AttributeReference(u32),
    Float(u32),
    Dimension(u32),
    Fraction(u32),
    Color(u32),
    Unknown {
        data_type: u8,
        data: u32,
        raw_value: Option<String>,
    },
}

impl XmlValue {
    pub fn as_text(&self) -> String {
        match self {
            Self::Null => String::new(),
            Self::String(value) => value.clone(),
            Self::Boolean(value) => value.to_string(),
            Self::Integer(value) => value.to_string(),
            Self::Hex(value) => format!("0x{value:08x}"),
            Self::Reference(value) => format!("@0x{value:08x}"),
            Self::AttributeReference(value) => format!("?0x{value:08x}"),
            Self::Float(value) => format!("float-bits:0x{value:08x}"),
            Self::Dimension(value) => format!("dimension:0x{value:08x}"),
            Self::Fraction(value) => format!("fraction:0x{value:08x}"),
            Self::Color(value) => format!("#{value:08x}"),
            Self::Unknown {
                data_type,
                data,
                raw_value,
            } => {
                if let Some(raw_value) = raw_value {
                    return raw_value.clone();
                }

                format!("type:0x{data_type:02x} data:0x{data:08x}")
            }
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(value) => Some(*value),
            Self::String(value) if value == "true" => Some(true),
            Self::String(value) if value == "false" => Some(false),
            _ => None,
        }
    }
}

impl fmt::Display for XmlValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.as_text())
    }
}

pub fn parse_binary_xml(data: &[u8]) -> Result<Vec<XmlEvent>> {
    let root = read_chunk_header(data, 0)?;

    if root.chunk_type != RES_XML_TYPE {
        return Err(WinedroidError::new(format!(
            "AndroidManifest.xml is not binary XML: root chunk type is 0x{:04x}",
            root.chunk_type
        )));
    }

    let document_end = checked_chunk_end(data, 0, root.size)?;
    let mut offset = root.header_size as usize;
    let mut string_pool = None;
    let mut events = Vec::new();

    while offset < document_end {
        let header = read_chunk_header(data, offset)?;
        let chunk_end = checked_chunk_end(data, offset, header.size)?;

        match header.chunk_type {
            RES_STRING_POOL_TYPE => {
                string_pool = Some(StringPool::parse(data, offset, chunk_end, header)?);
            }
            RES_XML_START_ELEMENT_TYPE => {
                let strings = require_string_pool(&string_pool)?;
                events.push(parse_start_element(
                    data, offset, chunk_end, header, strings,
                )?);
            }
            RES_XML_END_ELEMENT_TYPE => {
                let strings = require_string_pool(&string_pool)?;
                events.push(parse_end_element(data, offset, chunk_end, header, strings)?);
            }
            RES_XML_START_NAMESPACE_TYPE
            | RES_XML_END_NAMESPACE_TYPE
            | RES_XML_CDATA_TYPE
            | RES_XML_RESOURCE_MAP_TYPE => {}
            unsupported_type => {
                return Err(WinedroidError::new(format!(
                    "unsupported binary XML chunk type 0x{unsupported_type:04x} at offset {offset}"
                )));
            }
        }

        offset = chunk_end;
    }

    Ok(events)
}

pub fn local_name(qualified_name: &str) -> &str {
    match qualified_name.split_once(':') {
        Some((_, local)) => local,
        None => qualified_name,
    }
}

pub fn parse_plain_xml(data: &[u8]) -> Result<Vec<XmlEvent>> {
    let data = data.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(data);
    let mut events = Vec::new();
    let mut index = 0;

    while index < data.len() {
        if data[index] != b'<' {
            index += 1;
            continue;
        }

        let rest = &data[index..];
        if let Some(skip) = skip_delimited(rest, b"<?", b"?>") {
            index += skip;
            continue;
        }
        if let Some(skip) = skip_delimited(rest, b"<!--", b"-->") {
            index += skip;
            continue;
        }
        if let Some(skip) = skip_delimited(rest, b"<![CDATA[", b"]]>") {
            index += skip;
            continue;
        }
        if rest.starts_with(b"<!") {
            index += find_subsequence(rest, b">")
                .ok_or_else(|| WinedroidError::new("unterminated XML markup declaration"))?
                + 1;
            continue;
        }

        let close = plain_xml_tag_close(&data[index + 1..])
            .ok_or_else(|| WinedroidError::new("unterminated XML tag"))?;
        let inner = &data[index + 1..index + 1 + close];
        index += 1 + close + 1;

        if let Some(name_bytes) = inner.strip_prefix(b"/") {
            let name = String::from_utf8_lossy(name_bytes).trim().to_owned();
            events.push(XmlEvent::End(XmlEndElement {
                name,
                namespace: None,
            }));
            continue;
        }

        let self_closing = inner.ends_with(b"/");
        let inner = if self_closing {
            &inner[..inner.len() - 1]
        } else {
            inner
        };
        let (name, attributes) = parse_plain_xml_start_tag(inner)?;
        events.push(XmlEvent::Start(XmlStartElement {
            name: name.clone(),
            namespace: None,
            attributes,
        }));
        if self_closing {
            events.push(XmlEvent::End(XmlEndElement {
                name,
                namespace: None,
            }));
        }
    }

    Ok(events)
}

fn skip_delimited(data: &[u8], open: &[u8], close: &[u8]) -> Option<usize> {
    if !data.starts_with(open) {
        return None;
    }
    let after_open = &data[open.len()..];
    let end = find_subsequence(after_open, close)?;
    Some(open.len() + end + close.len())
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn plain_xml_tag_close(after_lt: &[u8]) -> Option<usize> {
    let mut quote: Option<u8> = None;
    for (offset, &byte) in after_lt.iter().enumerate() {
        match quote {
            Some(open) if byte == open => quote = None,
            Some(_) => {}
            None => match byte {
                b'"' | b'\'' => quote = Some(byte),
                b'>' => return Some(offset),
                _ => {}
            },
        }
    }
    None
}

fn parse_plain_xml_start_tag(inner: &[u8]) -> Result<(String, Vec<XmlAttribute>)> {
    let mut index = 0;
    while index < inner.len() && !inner[index].is_ascii_whitespace() {
        index += 1;
    }
    let name = String::from_utf8_lossy(&inner[..index]).into_owned();
    if name.is_empty() {
        return Err(WinedroidError::new("XML start tag has no element name"));
    }

    let mut attributes = Vec::new();
    while index < inner.len() {
        while index < inner.len() && inner[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= inner.len() {
            break;
        }

        let name_start = index;
        while index < inner.len() && inner[index] != b'=' && !inner[index].is_ascii_whitespace() {
            index += 1;
        }
        let attribute_name = String::from_utf8_lossy(&inner[name_start..index]).into_owned();

        while index < inner.len() && inner[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= inner.len() || inner[index] != b'=' {
            if !attribute_name.is_empty() {
                attributes.push(XmlAttribute {
                    name: attribute_name,
                    namespace: None,
                    value: XmlValue::String(String::new()),
                });
            }
            continue;
        }
        index += 1;
        while index < inner.len() && inner[index].is_ascii_whitespace() {
            index += 1;
        }

        let Some(&quote) = inner.get(index) else {
            return Err(WinedroidError::new("XML attribute is missing its value"));
        };
        if quote != b'"' && quote != b'\'' {
            return Err(WinedroidError::new("XML attribute value must be quoted"));
        }
        index += 1;
        let value_start = index;
        while index < inner.len() && inner[index] != quote {
            index += 1;
        }
        if index >= inner.len() {
            return Err(WinedroidError::new("unterminated XML attribute value"));
        }
        let raw_value = String::from_utf8_lossy(&inner[value_start..index]).into_owned();
        index += 1;

        attributes.push(XmlAttribute {
            name: attribute_name,
            namespace: None,
            value: XmlValue::String(decode_xml_entities(&raw_value)),
        });
    }

    Ok((name, attributes))
}

fn decode_xml_entities(input: &str) -> String {
    if !input.contains('&') {
        return input.to_owned();
    }

    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut index = 0;
    while index < input.len() {
        if bytes[index] == b'&' {
            if let Some(semicolon) = input[index + 1..].find(';') {
                let entity = &input[index + 1..index + 1 + semicolon];
                let decoded = match entity {
                    "amp" => Some('&'),
                    "lt" => Some('<'),
                    "gt" => Some('>'),
                    "quot" => Some('"'),
                    "apos" => Some('\''),
                    _ => entity
                        .strip_prefix("#x")
                        .or_else(|| entity.strip_prefix("#X"))
                        .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                        .or_else(|| entity.strip_prefix('#').and_then(|dec| dec.parse().ok()))
                        .and_then(char::from_u32),
                };
                if let Some(character) = decoded {
                    out.push(character);
                    index += 1 + semicolon + 1;
                    continue;
                }
            }
        }

        let character = input[index..]
            .chars()
            .next()
            .expect("index is on a char boundary");
        out.push(character);
        index += character.len_utf8();
    }
    out
}

fn require_string_pool<'a>(string_pool: &'a Option<StringPool>) -> Result<&'a StringPool> {
    string_pool
        .as_ref()
        .ok_or_else(|| WinedroidError::new("binary XML element appeared before string pool"))
}

fn parse_start_element(
    data: &[u8],
    offset: usize,
    chunk_end: usize,
    header: ChunkHeader,
    strings: &StringPool,
) -> Result<XmlEvent> {
    if header.header_size < 16 {
        return Err(WinedroidError::new(format!(
            "start element header is too small at offset {offset}"
        )));
    }

    let extension_offset = offset + header.header_size as usize;
    require_range(data, extension_offset, 20, "start element extension")?;

    let namespace = strings.optional(read_u32(data, extension_offset)?)?;
    let name = strings.required(read_u32(data, extension_offset + 4)?)?;
    let attribute_start = read_u16(data, extension_offset + 8)? as usize;
    let attribute_size = read_u16(data, extension_offset + 10)? as usize;
    let attribute_count = read_u16(data, extension_offset + 12)? as usize;

    if attribute_size < 20 {
        return Err(WinedroidError::new(format!(
            "attribute size is too small for element `{name}`"
        )));
    }

    let attribute_base = extension_offset + attribute_start;
    let mut attributes = Vec::with_capacity(attribute_count);

    for index in 0..attribute_count {
        let attribute_offset = attribute_base + index * attribute_size;

        if attribute_offset + 20 > chunk_end {
            return Err(WinedroidError::new(format!(
                "attribute {index} for element `{name}` exceeds its chunk"
            )));
        }

        attributes.push(parse_attribute(data, attribute_offset, strings)?);
    }

    Ok(XmlEvent::Start(XmlStartElement {
        name,
        namespace,
        attributes,
    }))
}

fn parse_attribute(data: &[u8], offset: usize, strings: &StringPool) -> Result<XmlAttribute> {
    let namespace = strings.optional(read_u32(data, offset)?)?;
    let name = strings.required(read_u32(data, offset + 4)?)?;
    let raw_value = strings.optional(read_u32(data, offset + 8)?)?;
    let value_size = read_u16(data, offset + 12)?;
    let data_type = read_u8(data, offset + 15)?;
    let data_value = read_u32(data, offset + 16)?;

    if value_size < 8 {
        return Err(WinedroidError::new(format!(
            "typed value for attribute `{name}` is too small"
        )));
    }

    let value = parse_typed_value(data_type, data_value, raw_value, strings)?;

    Ok(XmlAttribute {
        name,
        namespace,
        value,
    })
}

fn parse_typed_value(
    data_type: u8,
    data: u32,
    raw_value: Option<String>,
    strings: &StringPool,
) -> Result<XmlValue> {
    match data_type {
        TYPE_NULL => Ok(XmlValue::Null),
        TYPE_STRING => {
            let value = strings.required(data)?;
            Ok(XmlValue::String(value))
        }
        TYPE_INT_DEC => Ok(XmlValue::Integer(data as i32)),
        TYPE_INT_HEX => Ok(XmlValue::Hex(data)),
        TYPE_INT_BOOLEAN => Ok(XmlValue::Boolean(data != 0)),
        TYPE_REFERENCE => Ok(XmlValue::Reference(data)),
        TYPE_ATTRIBUTE => Ok(XmlValue::AttributeReference(data)),
        TYPE_FLOAT => Ok(XmlValue::Float(data)),
        TYPE_DIMENSION => Ok(XmlValue::Dimension(data)),
        TYPE_FRACTION => Ok(XmlValue::Fraction(data)),
        TYPE_FIRST_COLOR_INT..=TYPE_LAST_COLOR_INT => Ok(XmlValue::Color(data)),
        _ => Ok(XmlValue::Unknown {
            data_type,
            data,
            raw_value,
        }),
    }
}

fn parse_end_element(
    data: &[u8],
    offset: usize,
    _chunk_end: usize,
    header: ChunkHeader,
    strings: &StringPool,
) -> Result<XmlEvent> {
    if header.header_size < 16 {
        return Err(WinedroidError::new(format!(
            "end element header is too small at offset {offset}"
        )));
    }

    let extension_offset = offset + header.header_size as usize;
    require_range(data, extension_offset, 8, "end element extension")?;

    let namespace = strings.optional(read_u32(data, extension_offset)?)?;
    let name = strings.required(read_u32(data, extension_offset + 4)?)?;

    Ok(XmlEvent::End(XmlEndElement { name, namespace }))
}

#[derive(Debug, Clone, Copy)]
struct ChunkHeader {
    chunk_type: u16,
    header_size: u16,
    size: u32,
}

fn read_chunk_header(data: &[u8], offset: usize) -> Result<ChunkHeader> {
    require_range(data, offset, 8, "chunk header")?;

    let header = ChunkHeader {
        chunk_type: read_u16(data, offset)?,
        header_size: read_u16(data, offset + 2)?,
        size: read_u32(data, offset + 4)?,
    };

    if header.header_size < 8 {
        return Err(WinedroidError::new(format!(
            "chunk at offset {offset} has invalid header size {}",
            header.header_size
        )));
    }

    if header.size < u32::from(header.header_size) {
        return Err(WinedroidError::new(format!(
            "chunk at offset {offset} has invalid size {}",
            header.size
        )));
    }

    Ok(header)
}

fn checked_chunk_end(data: &[u8], offset: usize, size: u32) -> Result<usize> {
    let size = size as usize;
    let chunk_end = offset
        .checked_add(size)
        .ok_or_else(|| WinedroidError::new("chunk size overflow"))?;

    if chunk_end > data.len() {
        return Err(WinedroidError::new(format!(
            "chunk at offset {offset} extends past the end of the file"
        )));
    }

    Ok(chunk_end)
}

#[derive(Debug, Clone)]
struct StringPool {
    strings: Vec<String>,
}

impl StringPool {
    fn parse(data: &[u8], offset: usize, chunk_end: usize, header: ChunkHeader) -> Result<Self> {
        if header.header_size < 28 {
            return Err(WinedroidError::new(format!(
                "string pool header is too small at offset {offset}"
            )));
        }

        let string_count = read_u32(data, offset + 8)? as usize;
        let flags = read_u32(data, offset + 16)?;
        let strings_start = read_u32(data, offset + 20)? as usize;
        let offsets_start = offset + header.header_size as usize;
        let string_data_start = offset + strings_start;
        let is_utf8 = flags & UTF8_FLAG != 0;

        require_range(data, offsets_start, string_count * 4, "string offsets")?;

        if string_data_start > chunk_end {
            return Err(WinedroidError::new(format!(
                "string data for pool at offset {offset} starts outside its chunk"
            )));
        }

        let mut strings = Vec::with_capacity(string_count);

        for index in 0..string_count {
            let relative_offset = read_u32(data, offsets_start + index * 4)? as usize;
            let string_offset = string_data_start + relative_offset;

            if string_offset >= chunk_end {
                return Err(WinedroidError::new(format!(
                    "string {index} starts outside the string pool"
                )));
            }

            let bytes = &data[string_offset..chunk_end];
            let string = if is_utf8 {
                decode_utf8_string(bytes)?
            } else {
                decode_utf16_string(bytes)?
            };

            strings.push(string);
        }

        Ok(Self { strings })
    }

    fn optional(&self, index: u32) -> Result<Option<String>> {
        if index == NO_INDEX {
            return Ok(None);
        }

        self.required(index).map(Some)
    }

    fn required(&self, index: u32) -> Result<String> {
        self.strings
            .get(index as usize)
            .cloned()
            .ok_or_else(|| WinedroidError::new(format!("string pool index {index} is invalid")))
    }
}

fn decode_utf8_string(data: &[u8]) -> Result<String> {
    let (_, first_length_size) = read_utf8_length(data, 0)?;
    let (byte_length, second_length_size) = read_utf8_length(data, first_length_size)?;
    let value_offset = first_length_size + second_length_size;
    let terminator_offset = value_offset + byte_length;

    require_range(data, value_offset, byte_length + 1, "UTF-8 string")?;

    if data[terminator_offset] != 0 {
        return Err(WinedroidError::new("UTF-8 string is not null-terminated"));
    }

    crate::resource_table::decode_string_pool_utf8(&data[value_offset..terminator_offset])
}

fn decode_utf16_string(data: &[u8]) -> Result<String> {
    let (code_unit_count, length_size) = read_utf16_length(data, 0)?;
    let value_offset = length_size;
    let byte_length = code_unit_count * 2;
    let terminator_offset = value_offset + byte_length;

    require_range(data, value_offset, byte_length + 2, "UTF-16 string")?;

    if data[terminator_offset] != 0 || data[terminator_offset + 1] != 0 {
        return Err(WinedroidError::new("UTF-16 string is not null-terminated"));
    }

    let mut code_units = Vec::with_capacity(code_unit_count);

    for index in 0..code_unit_count {
        code_units.push(read_u16(data, value_offset + index * 2)?);
    }

    String::from_utf16(&code_units)
        .map_err(|error| WinedroidError::with_source("invalid UTF-16 string", error))
}

fn read_utf8_length(data: &[u8], offset: usize) -> Result<(usize, usize)> {
    let first = read_u8(data, offset)?;

    if first & 0x80 == 0 {
        return Ok((first as usize, 1));
    }

    let second = read_u8(data, offset + 1)?;
    let value = (((first & 0x7f) as usize) << 8) | second as usize;

    Ok((value, 2))
}

fn read_utf16_length(data: &[u8], offset: usize) -> Result<(usize, usize)> {
    let first = read_u16(data, offset)?;

    if first & 0x8000 == 0 {
        return Ok((first as usize, 2));
    }

    let second = read_u16(data, offset + 2)?;
    let value = (((first & 0x7fff) as usize) << 16) | second as usize;

    Ok((value, 4))
}

fn read_u8(data: &[u8], offset: usize) -> Result<u8> {
    require_range(data, offset, 1, "u8")?;
    Ok(data[offset])
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
    fn local_name_strips_namespace_prefix() {
        assert_eq!(local_name("gesmes:Envelope"), "Envelope");
        assert_eq!(local_name("Cube"), "Cube");
        assert_eq!(local_name("currency"), "currency");
    }

    #[test]
    fn parses_plain_xml_elements_and_attributes() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<!-- a comment -->
<gesmes:Envelope xmlns:gesmes="urn:x">
  <Cube currency='USD' rate="1.19"/>
  <Cube currency='J&amp;Y' rate='182.79' />
  <note>text is skipped</note>
</gesmes:Envelope>"#;

        let events = parse_plain_xml(xml).expect("plain XML should parse");

        let names: Vec<_> = events
            .iter()
            .map(|event| match event {
                XmlEvent::Start(element) => format!("start:{}", element.name),
                XmlEvent::End(element) => format!("end:{}", element.name),
            })
            .collect();
        assert_eq!(
            names,
            vec![
                "start:gesmes:Envelope",
                "start:Cube",
                "end:Cube",
                "start:Cube",
                "end:Cube",
                "start:note",
                "end:note",
                "end:gesmes:Envelope",
            ]
        );

        let XmlEvent::Start(first_cube) = &events[1] else {
            panic!("expected a Cube start element");
        };
        assert_eq!(first_cube.attributes[0].name, "currency");
        assert_eq!(first_cube.attributes[0].value.as_text(), "USD");
        assert_eq!(first_cube.attributes[1].value.as_text(), "1.19");

        let XmlEvent::Start(second_cube) = &events[3] else {
            panic!("expected a Cube start element");
        };
        assert_eq!(second_cube.attributes[0].value.as_text(), "J&Y");
    }

    #[test]
    fn decodes_short_utf8_string() {
        let mut data = Vec::new();
        data.push(4);
        data.push(4);
        data.extend_from_slice(b"test");
        data.push(0);

        let decoded = decode_utf8_string(&data).expect("UTF-8 string should decode");

        assert_eq!(decoded, "test");
    }

    #[test]
    fn decodes_short_utf16_string() {
        let data = [
            0x04, 0x00, b't', 0x00, b'e', 0x00, b's', 0x00, b't', 0x00, 0x00, 0x00,
        ];

        let decoded = decode_utf16_string(&data).expect("UTF-16 string should decode");

        assert_eq!(decoded, "test");
    }
}
