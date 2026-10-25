// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0

//! SDF element cursor over the XML tokenizer. Names stay lexical, matching
//! TinyXML's legacy extension behavior; source ranges always index original XML.
use crate::ErrorCode;
use quick_xml::{Reader, events::Event};
use std::{borrow::Cow, cell::RefCell, fmt, ops::Range};
use xmlparser::{Reference, Stream, TextPos};

pub(crate) fn legacy_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{b}' | '\u{c}')
}

struct Element {
    name: String,
    attributes: Vec<(String, String)>,
    children: Vec<usize>,
    text: String,
    text_state: TextState,
    text_is_cdata: bool,
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
        if self.text_state == TextState::None
            && (reference || !text.trim_matches(legacy_whitespace).is_empty())
        {
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
    positions: RefCell<Vec<TextPos>>,
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
    fn element_position(&self, index: usize) -> TextPos {
        let mut positions = self.positions.borrow_mut();
        // Only the prefix requested by semantic loading is needed. Opaque
        // trailing plugin children need no position table. No callbacks or
        // nested position lookups occur while this private cache is borrowed.
        while positions.len() <= index {
            let next = positions.len();
            let offset = self.elements[next].range.start;
            let position = if let Some(previous) = positions.last() {
                let start = self.elements[next - 1].range.start;
                let fragment = &self.xml[start..offset];
                let delta = Stream::from(fragment).gen_text_pos_from(fragment.len());
                TextPos::new(
                    previous.row + (delta.row - 1),
                    if delta.row == 1 {
                        previous.col + (delta.col - 1)
                    } else {
                        delta.col
                    },
                )
            } else {
                self.text_pos_at(offset)
            };
            positions.push(position);
        }
        positions[index]
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
            positions: RefCell::default(),
        };
        let mut reader = Reader::from_str(xml);
        // The pinned slice tokenizer consumes an initial UTF-8 BOM without
        // counting its bytes in reported offsets. Ranges index the original XML.
        let bom_len = if xml.starts_with('\u{feff}') {
            '\u{feff}'.len_utf8()
        } else {
            0
        };
        reader.config_mut().check_comments = false;
        reader.config_mut().allow_dangling_amp = true;
        if let Some(offset) = xml.find('\0') {
            return Err(document.error(ErrorCode::Xml, offset, "NUL byte in XML input"));
        }
        let mut stack = Vec::<usize>::new();
        let mut prolog_closed = false;
        loop {
            let start = reader.buffer_position() as usize + bom_len;
            let event = reader.read_event().map_err(|error| {
                document.error(
                    ErrorCode::Xml,
                    reader.error_position() as usize + bom_len,
                    error.to_string(),
                )
            })?;
            let end = reader.buffer_position() as usize + bom_len;
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
                            // Normalize literal EOLs before reference expansion;
                            // a numeric CR reference must remain distinct.
                            let value = normalized(utf8(attribute.value.as_ref()));
                            attributes.push((name.into(), references(&value)));
                        }
                    }
                    let index = document.elements.len();
                    if let Some(parent) = stack.last() {
                        // A leading element blocks native GetText; comments do not.
                        document.elements[*parent].text_state = TextState::Finished;
                        document.elements[*parent].children.push(index);
                    }
                    document.elements.push(Element {
                        name,
                        attributes,
                        children: vec![],
                        text: String::new(),
                        text_state: TextState::None,
                        text_is_cdata: false,
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
                    let text = normalized(utf8(text.as_ref()));
                    if let Some(index) = stack.last() {
                        document.elements[*index].plain_text(&text, false);
                    } else if !text.trim().is_empty() {
                        return Err(document.error(ErrorCode::Xml, start, "text outside XML root"));
                    }
                }
                Event::CData(text) => {
                    let text = normalized(utf8(text.as_ref()));
                    if let Some(index) = stack.last() {
                        let element = &mut document.elements[*index];
                        if element.text_state == TextState::None {
                            element.text.push_str(&text);
                            element.text_is_cdata = true;
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
// TinyXML folds CR/LF and LF/CR pairs in one left-to-right pass. XML1.0's
// CR-only normalization loses that native rule; references are expanded later.
fn normalized(raw: &str) -> Cow<'_, str> {
    if !raw.contains('\r') {
        return Cow::Borrowed(raw);
    }
    let mut input = raw.chars().peekable();
    let mut output = String::with_capacity(raw.len());
    while let Some(ch) = input.next() {
        if matches!(ch, '\r' | '\n') {
            if input
                .peek()
                .is_some_and(|&next| matches!((ch, next), ('\r', '\n') | ('\n', '\r')))
            {
                input.next();
            }
            output.push('\n');
        } else {
            output.push(ch);
        }
    }
    Cow::Owned(output)
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
    pub fn position(self) -> TextPos {
        self.document.element_position(self.index)
    }
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
    pub fn text_is_cdata(self) -> bool {
        self.document.elements[self.index].text_is_cdata
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
