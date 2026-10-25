// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0

//! Pure selection from model.config; filesystem and URI policy belong to callers.
use crate::{
    Diagnostic, ErrorCode, Source,
    xml::{Document, Node, legacy_whitespace},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelConfigWarningKind {
    NewerVersion(String),
    MissingVersion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelConfigWarning {
    pub kind: ModelConfigWarningKind,
    pub source: Source,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelConfigSelection {
    /// XML text, not a resolved or validated filesystem path.
    pub file_name: String,
    pub declared_version: Option<String>,
    pub warnings: Vec<ModelConfigWarning>,
}

/// Select a file using the pinned native model.config version rules (parser 1.12).
///
/// Highest supported version wins, retaining the first tie. If none improves on
/// 0.0, native retains the first sdf tag, even if it declares a future version.
/// The selected file still needs normal SDF loading and capability validation.
/// This function performs no IO, path validation or manifest.xml fallback.
/// Native numeric-version exceptions become InvalidValue errors; warning
/// construction is safe when the optional model name is absent.
pub fn select_model_config(xml: &str, source: &str) -> Result<ModelConfigSelection, Diagnostic> {
    let doc = Document::parse(xml).map_err(|error| Diagnostic {
        code: error.code,
        source: Source {
            uri: source.into(),
            line: error.pos().row,
            column: error.pos().col,
        },
        message: error.to_string(),
    })?;
    let root = doc.root_element();
    let err = |node: Node<'_, '_>, code, message: &str| Diagnostic {
        code,
        source: location(node, source),
        message: message.into(),
    };
    if !root.has_tag_name("model") {
        return Err(err(
            root,
            ErrorCode::Missing,
            "model.config requires a model element",
        ));
    }
    let mut entries = root.children().filter(|node| node.has_tag_name("sdf"));
    let first = entries.next().ok_or_else(|| {
        err(
            root,
            ErrorCode::Missing,
            "model.config requires an sdf element",
        )
    })?;
    let mut selected = first;
    let mut best = Version::default();
    let mut warnings = Vec::new();
    for entry in std::iter::once(first).chain(entries) {
        if let Some(text) = entry.attribute("version") {
            let version = Version::parse(text).map_err(|()| {
                err(
                    entry,
                    ErrorCode::InvalidValue,
                    "invalid model.config version integer",
                )
            })?;
            if version.key() > best.key() {
                if version.key() <= (1, 12, 0, true) {
                    selected = entry;
                    best = version;
                } else {
                    warnings.push(ModelConfigWarning {
                        kind: ModelConfigWarningKind::NewerVersion(text.into()),
                        source: location(entry, source),
                    });
                }
            }
        }
    }
    // TinyXML GetText distinguishes an empty CDATA node from no text node.
    let text = selected
        .text()
        .or_else(|| selected.text_is_cdata().then_some(""))
        .ok_or_else(|| {
            err(
                selected,
                ErrorCode::Missing,
                "selected sdf element has no filename text",
            )
        })?;
    let file_name = if selected.text_is_cdata() {
        text.into()
    } else {
        text.split(legacy_whitespace)
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    };
    let declared_version = selected.attribute("version").map(str::to_owned);
    if declared_version.is_none() {
        warnings.push(ModelConfigWarning {
            kind: ModelConfigWarningKind::MissingVersion,
            source: location(selected, source),
        });
    }
    Ok(ModelConfigSelection {
        file_name,
        declared_version,
        warnings,
    })
}

fn location(node: Node<'_, '_>, uri: &str) -> Source {
    let pos = node.document().text_pos_at(node.range().start);
    Source {
        uri: uri.into(),
        line: pos.row,
        column: pos.col,
    }
}

#[derive(Default)]
struct Version {
    numbers: [u32; 3],
    prerelease: bool,
}
impl Version {
    fn key(&self) -> (u32, u32, u32, bool) {
        (
            self.numbers[0],
            self.numbers[1],
            self.numbers[2],
            !self.prerelease,
        )
    }
    // gz-math SemanticVersion is intentionally looser than SemVer: signed
    // integer prefixes, missing components and ignored prerelease labels matter.
    fn parse(text: &str) -> Result<Self, ()> {
        let mut result = Self::default();
        let pre = text.find('-');
        let build = text.find('+');
        if matches!((build, pre), (Some(b), Some(p)) if b < p) {
            return Ok(result);
        }
        result.prerelease = pre.is_some_and(|p| build.unwrap_or(text.len()) > p + 1);
        let end = pre.or(build).unwrap_or(text.len());
        for (index, token) in text[..end].split_terminator('.').enumerate() {
            let token = token.trim_start_matches(legacy_whitespace);
            let sign = usize::from(token.starts_with(['+', '-']));
            let count = sign + token[sign..].bytes().take_while(u8::is_ascii_digit).count();
            let value: i32 = token[..count].parse().map_err(|_| ())?;
            result.numbers[if index > 2 { 0 } else { index }] = value as u32;
        }
        Ok(result)
    }
}
