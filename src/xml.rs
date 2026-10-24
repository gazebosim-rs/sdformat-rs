//! SDF element cursor over the XML tokenizer. Names stay lexical, matching
//! TinyXML's legacy extension behavior; source ranges always index original XML.
use crate::ErrorCode;
use quick_xml::{Reader, events::Event};
use std::{fmt, ops::Range};
use xmlparser::{Reference, Stream, TextPos};

struct Element {
    name: String,
    attributes: Vec<(String, String)>,
    children: Vec<usize>,
    text: String,
    text_state: TextState,
    range: Range<usize>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum TextState {
    None,
    Active,
    Finished,
}
impl Element {
    // Entity events continue the same text node; markup closes its first segment.
    fn plain_text(&mut self, text: &str, reference: bool) {
        if self.text_state == TextState::None && (reference || !text.trim().is_empty()) {
            self.text_state = TextState::Active;
        }
        if self.text_state == TextState::Active {
            self.text.push_str(text);
        }
    }
    fn close_text(&mut self) {
        if self.text_state == TextState::Active {
            self.text_state = TextState::Finished;
        }
    }
}
pub(crate) struct Document<'input> {
    xml: &'input str,
    elements: Vec<Element>,
}
#[derive(Debug)]
pub(crate) struct XmlError {
    pub code: ErrorCode,
    position: TextPos,
    message: String,
}
impl XmlError {
    pub fn pos(&self) -> TextPos {
        self.position
    }
}
impl fmt::Display for XmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl Document<'_> {
    fn error(&self, code: ErrorCode, offset: usize, message: impl Into<String>) -> XmlError {
        XmlError {
            code,
            position: self.text_pos_at(offset),
            message: message.into(),
        }
    }
    pub fn text_pos_at(&self, offset: usize) -> TextPos {
        Stream::from(self.xml).gen_text_pos_from(offset)
    }
    pub fn root_element(&self) -> Node<'_, '_> {
        Node {
            document: self,
            index: 0,
        }
    }
}
impl<'input> Document<'input> {
    pub fn parse(xml: &'input str) -> Result<Self, XmlError> {
        let mut document = Self {
            xml,
            elements: vec![],
        };
        let mut reader = Reader::from_str(xml);
        reader.config_mut().check_comments = false;
        reader.config_mut().allow_dangling_amp = true;
        if let Some(offset) = xml.find('\0') {
            return Err(document.error(ErrorCode::Xml, offset, "NUL byte in XML input"));
        }
        let mut stack = Vec::<usize>::new();
        let mut prolog_closed = false;
        loop {
            let start = reader.buffer_position() as usize;
            let event = reader.read_event().map_err(|error| {
                document.error(
                    ErrorCode::Xml,
                    reader.error_position() as usize,
                    error.to_string(),
                )
            })?;
            let end = reader.buffer_position() as usize;
            let empty = matches!(&event, Event::Empty(_));
            match event {
                Event::Start(element) | Event::Empty(element) => {
                    // Native boundary:498 open nonempty elements are accepted;
                    // a self-closing child does not consume recursive depth.
                    if !empty && stack.len() >= 498 {
                        return Err(document.error(
                            ErrorCode::Xml,
                            start,
                            "XML element depth exceeded",
                        ));
                    }
                    prolog_closed = true;
                    if stack.is_empty() && !document.elements.is_empty() {
                        return Err(document.error(
                            ErrorCode::Xml,
                            start,
                            "multiple XML root elements",
                        ));
                    }
                    let name = utf8(element.name().as_ref()).to_string();
                    validate_name(&name)
                        .map_err(|error| document.error(ErrorCode::Xml, start, error))?;
                    let mut attributes = vec![];
                    for attribute in element.attributes() {
                        let attribute = attribute.map_err(|error| {
                            document.error(ErrorCode::Xml, start, error.to_string())
                        })?;
                        let name = utf8(attribute.key.as_ref());
                        validate_name(name)
                            .map_err(|error| document.error(ErrorCode::Xml, start, error))?;
                        // Namespace declarations are metadata, not SDF attributes.
                        if name != "xmlns" && !name.starts_with("xmlns:") {
                            attributes
                                .push((name.into(), references(utf8(attribute.value.as_ref()))));
                        }
                    }
                    let index = document.elements.len();
                    if let Some(parent) = stack.last() {
                        document.elements[*parent].close_text();
                        document.elements[*parent].children.push(index);
                    }
                    document.elements.push(Element {
                        name,
                        attributes,
                        children: vec![],
                        text: String::new(),
                        text_state: TextState::None,
                        range: start..end,
                    });
                    if !empty {
                        stack.push(index);
                    }
                }
                Event::End(_) => {
                    // quick-xml checks matched literal start/end names before here.
                    let index = stack.pop().expect("tokenizer checked matching start tag");
                    document.elements[index].range.end = end;
                }
                Event::Text(text) => {
                    let text = text.xml10_content().map_err(|error| {
                        document.error(ErrorCode::Xml, start, error.to_string())
                    })?;
                    if let Some(index) = stack.last() {
                        document.elements[*index].plain_text(&text, false);
                    } else if !text.trim().is_empty() {
                        return Err(document.error(ErrorCode::Xml, start, "text outside XML root"));
                    }
                }
                Event::CData(text) => {
                    let text = text.xml10_content().map_err(|error| {
                        document.error(ErrorCode::Xml, start, error.to_string())
                    })?;
                    if let Some(index) = stack.last() {
                        let element = &mut document.elements[*index];
                        if element.text_state == TextState::None {
                            element.text.push_str(&text);
                        }
                        element.text_state = TextState::Finished;
                    } else {
                        return Err(document.error(
                            ErrorCode::Xml,
                            start,
                            "CDATA outside XML root",
                        ));
                    }
                }
                Event::GeneralRef(entity) => {
                    let text = references(&format!("&{};", utf8(entity.as_ref())));
                    if let Some(index) = stack.last() {
                        document.elements[*index].plain_text(&text, true);
                    } else {
                        return Err(document.error(
                            ErrorCode::Xml,
                            start,
                            "reference outside XML root",
                        ));
                    }
                }
                Event::DocType(_) => {
                    return Err(document.error(
                        ErrorCode::Unsupported,
                        start,
                        "DTD declarations are not supported",
                    ));
                }
                Event::Eof => break,
                Event::Decl(_) | Event::PI(_) => {
                    if prolog_closed {
                        return Err(document.error(
                            ErrorCode::Xml,
                            start,
                            "XML declaration is only allowed in the prolog",
                        ));
                    }
                }
                Event::Comment(_) => {
                    prolog_closed = true;
                    if let Some(index) = stack.last() {
                        document.elements[*index].close_text();
                    }
                }
            }
        }
        if document.elements.is_empty() || !stack.is_empty() {
            return Err(document.error(ErrorCode::Xml, xml.len(), "missing or unclosed XML root"));
        }
        Ok(document)
    }
}
fn utf8(bytes: &[u8]) -> &str {
    // Reader::from_str yields spans of the UTF-8 input, never transcoded bytes.
    std::str::from_utf8(bytes).expect("token spans come from UTF-8 string input")
}
fn validate_name(name: &str) -> Result<(), &'static str> {
    let mut stream = Stream::from(name);
    let _ = stream.consume_name().map_err(|_| "invalid XML name")?;
    if !stream.at_end() {
        return Err("invalid XML name");
    }
    Ok(())
}
fn references(raw: &str) -> String {
    let mut stream = Stream::from(raw);
    let mut output = String::new();
    while !stream.at_end() {
        let start = stream.pos();
        if stream.curr_byte_unchecked() == b'&' {
            match stream.try_consume_reference() {
                Some(Reference::Char(value)) => output.push(value),
                Some(Reference::Entity(_)) => output.push_str(&raw[start..stream.pos()]),
                None => {
                    if let Some((value, length)) = legacy_numeric_reference(&raw[start..]) {
                        output.push(value);
                        stream.advance(length);
                    } else {
                        output.push('&');
                        stream.advance(1);
                    }
                }
            }
        } else {
            let _ = stream.consume_bytes(|_, byte| byte != b'&');
            output.push_str(&raw[start..stream.pos()]);
        }
    }
    output
}

// TinyXML accepts numeric control references excluded by XML1.0. Reuse the
// standard numeric parser for that narrow gap; typed C-string views truncate NUL.
fn legacy_numeric_reference(raw: &str) -> Option<(char, usize)> {
    let reference = raw.strip_prefix("&#")?;
    let end = reference.find(';')?;
    let digits = &reference[..end];
    let (digits, radix) = digits
        .strip_prefix('x')
        .map_or((digits, 10), |hex| (hex, 16));
    if !digits.bytes().all(|byte| {
        if radix == 16 {
            byte.is_ascii_hexdigit()
        } else {
            byte.is_ascii_digit()
        }
    }) {
        return None;
    }
    let value = char::from_u32(u32::from_str_radix(digits, radix).ok()?)?;
    Some((value, end + 3))
}

#[derive(Clone, Copy)]
pub(crate) struct Node<'a, 'input> {
    document: &'a Document<'input>,
    index: usize,
}
impl<'a, 'input> Node<'a, 'input> {
    pub fn document(self) -> &'a Document<'input> {
        self.document
    }
    pub fn range(self) -> Range<usize> {
        self.document.elements[self.index].range.clone()
    }
    pub fn tag_name(self) -> Name<'a> {
        Name {
            value: &self.document.elements[self.index].name,
        }
    }
    pub fn has_tag_name(self, name: &str) -> bool {
        self.tag_name().value == name
    }
    pub fn attribute(self, name: &str) -> Option<&'a str> {
        self.document.elements[self.index]
            .attributes
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.split('\0').next().expect("split has a first part"))
    }
    pub fn attributes(self) -> impl Iterator<Item = Name<'a>> {
        self.document.elements[self.index]
            .attributes
            .iter()
            .map(|(name, _)| Name { value: name })
    }
    pub fn children(self) -> impl Iterator<Item = Self> {
        self.document.elements[self.index]
            .children
            .iter()
            .map(move |index| Self {
                document: self.document,
                index: *index,
            })
    }
    pub fn text(self) -> Option<&'a str> {
        let text = &self.document.elements[self.index].text;
        if text.is_empty() {
            None
        } else {
            Some(text.split('\0').next().expect("split has a first part"))
        }
    }
}
pub(crate) struct Name<'a> {
    value: &'a str,
}
impl<'a> Name<'a> {
    pub fn name(&self) -> &'a str {
        self.value
    }
    pub fn namespace(&self) -> Option<&'a str> {
        self.value.split_once(':').map(|(prefix, _)| prefix)
    }
}
