// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Fail-closed syntax evidence for the changed-line coverage gate.
use std::collections::BTreeSet;
use std::io::{self, Read};
use syn::{
    GenericArgument, Item, PathArguments, Type, Visibility,
    spanned::Spanned,
    visit::{self, Visit},
};

fn docs_only(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().all(|attribute| {
        attribute.path().is_ident("doc")
            && matches!(&attribute.meta, syn::Meta::NameValue(value)
                if matches!(&value.value, syn::Expr::Lit(lit) if matches!(lit.lit, syn::Lit::Str(_))))
    })
}
fn debug_or_docs(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().all(|attribute| {
        docs_only(std::slice::from_ref(attribute))
            || (attribute.path().is_ident("derive")
                && attribute
                    .parse_args::<syn::Path>()
                    .is_ok_and(|p| p.is_ident("Debug")))
    })
}

fn exposes_builtin(tree: &syn::UseTree) -> bool {
    match tree {
        syn::UseTree::Path(path) => exposes_builtin(&path.tree),
        syn::UseTree::Name(import) => [
            "Debug",
            "Clone",
            "Copy",
            "PartialEq",
            "Eq",
            "Default",
            "default",
        ]
        .iter()
        .any(|name| import.ident == *name),
        syn::UseTree::Rename(rename) => [
            "Debug",
            "Clone",
            "Copy",
            "PartialEq",
            "Eq",
            "Default",
            "default",
        ]
        .iter()
        .any(|name| rename.rename == *name),
        syn::UseTree::Glob(_) => true,
        syn::UseTree::Group(group) => group.items.iter().any(exposes_builtin),
    }
}
fn safe_module_attribute(attribute: &syn::Attribute) -> bool {
    docs_only(std::slice::from_ref(attribute))
        || attribute.path().is_ident("allow")
        || attribute.path().is_ident("cfg")
}
struct AncestorAudit {
    opaque: bool,
}
impl<'ast> Visit<'ast> for AncestorAudit {
    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        if !safe_module_attribute(attribute) {
            self.opaque = true;
        }
    }
    fn visit_item_macro(&mut self, _: &'ast syn::ItemMacro) {
        self.opaque = true;
    }
    fn visit_item_extern_crate(&mut self, _: &'ast syn::ItemExternCrate) {
        self.opaque = true;
    }
}
fn standard_debug_scope(target: &syn::File, ancestor: &syn::File, module: &str) -> bool {
    let mut ancestor_audit = AncestorAudit { opaque: false };
    ancestor_audit.visit_file(ancestor);
    if ancestor_audit.opaque
        || !docs_only(&target.attrs)
        || !docs_only(&ancestor.attrs)
        || module.is_empty()
    {
        return false;
    }
    if target.items.iter().any(|item| match item {
        Item::Use(item) => exposes_builtin(&item.tree),
        Item::Macro(_) | Item::ExternCrate(_) | Item::Verbatim(_) => true,
        _ => false,
    }) {
        return false;
    }
    let mut found = false;
    for item in &ancestor.items {
        match item {
            Item::Mod(item) if item.ident == module => {
                if found || item.content.is_some() || !docs_only(&item.attrs) {
                    return false;
                }
                found = true;
            }
            Item::Mod(item) => {
                if !item.attrs.iter().all(safe_module_attribute) {
                    return false;
                }
            }
            Item::Use(item) => {
                if !docs_only(&item.attrs) {
                    return false;
                }
            }
            Item::Macro(_) | Item::ExternCrate(_) => return false,
            _ if !found => return false,
            _ => {}
        }
    }
    found
}

fn public_vec_ident(field: &syn::Field) -> bool {
    let Visibility::Public(_) = field.vis else {
        return false;
    };
    let Type::Path(ty) = &field.ty else {
        return false;
    };
    if ty.qself.is_some() || ty.path.segments.len() != 1 {
        return false;
    }
    let segment = &ty.path.segments[0];
    if segment.ident != "Vec" {
        return false;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return false;
    };
    if arguments.args.len() != 1 {
        return false;
    }
    let GenericArgument::Type(Type::Path(item)) = &arguments.args[0] else {
        return false;
    };
    item.qself.is_none()
        && item.path.segments.len() == 1
        && matches!(item.path.segments[0].arguments, PathArguments::None)
}
struct Audit {
    opaque: bool,
    closings: BTreeSet<usize>,
    opaque_lines: BTreeSet<usize>,
    declarations: BTreeSet<usize>,
    item_macro: bool,
    standard_scope: bool,
    default_variant: bool,
}
impl Audit {
    fn mark(&mut self, span: proc_macro2::Span) {
        self.opaque_lines
            .extend(span.start().line..=span.end().line);
    }
}
impl<'ast> Visit<'ast> for Audit {
    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        if !docs_only(std::slice::from_ref(attribute)) {
            let valid = self.standard_scope && self.default_variant && attribute.path().is_ident("default")
                || self.standard_scope && attribute.path().is_ident("derive")
                && attribute
                    .parse_args_with(
                        syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                    )
                    .is_ok_and(|paths| {
                        !paths.is_empty()
                            && paths.iter().all(|path| {
                                ["Debug", "Clone", "Copy", "PartialEq", "Eq", "Default"]
                                    .iter()
                                    .any(|name| path.is_ident(name))
                            })
                    });
            if !valid {
                self.opaque = true;
            }
            self.mark(attribute.span());
        }
    }
    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        for attr in &item.attrs {
            self.visit_attribute(attr);
        }
        self.visit_visibility(&item.vis);
        self.visit_generics(&item.generics);
        let has_default = item.attrs.iter().any(|attr| {
            attr.path().is_ident("derive")
                && attr
                    .parse_args_with(
                        syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                    )
                    .is_ok_and(|paths| paths.iter().any(|path| path.is_ident("Default")))
        });
        for variant in &item.variants {
            self.default_variant = has_default;
            for attr in &variant.attrs {
                self.visit_attribute(attr);
            }
            self.default_variant = false;
            self.visit_fields(&variant.fields);
            if let Some((_, expression)) = &variant.discriminant {
                self.visit_expr(expression);
            }
        }
    }
    fn visit_item_extern_crate(&mut self, item: &'ast syn::ItemExternCrate) {
        self.opaque = true;
        visit::visit_item_extern_crate(self, item);
    }
    fn visit_item_macro(&mut self, item: &'ast syn::ItemMacro) {
        self.item_macro = true;
        self.opaque = true;
        visit::visit_item_macro(self, item);
    }
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        self.mark(item.span());
    }
    fn visit_lit(&mut self, item: &'ast syn::Lit) {
        self.mark(item.span());
    }
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        if docs_only(&item.attrs)
            && item.trait_.is_none()
            && item.generics.params.is_empty()
            && item.generics.where_clause.is_none()
            && matches!(&*item.self_ty, Type::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 && matches!(path.path.segments[0].arguments, PathArguments::None))
        {
            self.declarations.insert(item.impl_token.span.start().line);
        }
        visit::visit_item_impl(self, item);
    }
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        if exposes_builtin(&item.tree) {
            self.opaque = true;
        }
        if docs_only(&item.attrs) && item.span().start().line == item.span().end().line {
            self.declarations.insert(item.span().start().line);
        }
        visit::visit_item_use(self, item);
    }
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        if docs_only(&item.attrs) && item.content.is_none() {
            self.declarations.insert(item.span().start().line);
        }
        visit::visit_item_mod(self, item);
    }
    fn visit_expr_call(&mut self, expression: &'ast syn::ExprCall) {
        self.closings.insert(expression.span().end().line);
        visit::visit_expr_call(self, expression);
    }
    fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
        self.closings.insert(expression.span().end().line);
        visit::visit_expr_method_call(self, expression);
    }
}
// Independent evidence for an unannotated function body. Derive macros on
// separate types add items; they do not rewrite the original function body.
struct CallAncestorAudit {
    opaque: bool,
}
impl<'ast> Visit<'ast> for CallAncestorAudit {
    fn visit_attribute(&mut self, attr: &'ast syn::Attribute) {
        if !docs_only(std::slice::from_ref(attr)) && !attr.path().is_ident("allow") {
            self.opaque = true;
        }
    }
    fn visit_item_macro(&mut self, _: &'ast syn::ItemMacro) {
        self.opaque = true;
    }
    fn visit_item_extern_crate(&mut self, _: &'ast syn::ItemExternCrate) {
        self.opaque = true;
    }
}
struct CallAudit {
    opaque: bool,
    in_function: bool,
    type_attributes: bool,
    closings: BTreeSet<usize>,
}
impl<'ast> Visit<'ast> for CallAudit {
    fn visit_attribute(&mut self, attr: &'ast syn::Attribute) {
        if docs_only(std::slice::from_ref(attr)) {
            return;
        }
        let derive = self.type_attributes
            && attr.path().is_ident("derive")
            && attr
                .parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                )
                .is_ok_and(|paths| {
                    !paths.is_empty()
                        && paths.iter().all(|p| {
                            ["Debug", "Clone", "Copy", "PartialEq", "Eq", "Default"]
                                .iter()
                                .any(|name| p.is_ident(name))
                        })
                });
        if !derive {
            self.opaque = true;
        }
    }
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        self.type_attributes = true;
        for attr in &item.attrs {
            self.visit_attribute(attr);
        }
        self.type_attributes = false;
        self.visit_fields(&item.fields);
    }
    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        self.type_attributes = true;
        for attr in &item.attrs {
            self.visit_attribute(attr);
        }
        self.type_attributes = false;
        for variant in &item.variants {
            self.visit_variant(variant);
        }
    }
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        for attr in &item.attrs {
            self.visit_attribute(attr);
        }
        let old = self.in_function;
        self.in_function = true;
        self.visit_block(&item.block);
        self.in_function = old;
    }
    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        for attr in &item.attrs {
            self.visit_attribute(attr);
        }
        let old = self.in_function;
        self.in_function = true;
        self.visit_block(&item.block);
        self.in_function = old;
    }
    fn visit_item_macro(&mut self, _: &'ast syn::ItemMacro) {
        self.opaque = true;
    }
    fn visit_expr_call(&mut self, expression: &'ast syn::ExprCall) {
        if self.in_function {
            self.closings.insert(expression.span().end().line);
        }
        visit::visit_expr_call(self, expression);
    }
    fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
        if self.in_function {
            self.closings.insert(expression.span().end().line);
        }
        visit::visit_expr_method_call(self, expression);
    }
}
struct Evidence {
    fields: BTreeSet<usize>,
    closings: BTreeSet<usize>,
    opaque: BTreeSet<usize>,
    declarations: BTreeSet<usize>,
}
fn classify(source: &str, ancestor: &str, module: &str) -> Result<Evidence, syn::Error> {
    let syntax = syn::parse_file(source)?;
    let ancestor_syntax = syn::parse_file(ancestor)?;
    let mut fields = BTreeSet::new();
    let standard_scope = standard_debug_scope(&syntax, &ancestor_syntax, module);
    if standard_scope {
        for item in &syntax.items {
            if let Item::Struct(item) = item
                && debug_or_docs(&item.attrs)
            {
                for field in &item.fields {
                    if docs_only(&field.attrs)
                        && public_vec_ident(field)
                        && let Some(name) = &field.ident
                    {
                        fields.insert(name.span().start().line);
                    }
                }
            }
        }
    }
    let mut audit = Audit {
        opaque: false,
        closings: BTreeSet::new(),
        opaque_lines: BTreeSet::new(),
        declarations: BTreeSet::new(),
        item_macro: false,
        standard_scope,
        default_variant: false,
    };
    audit.visit_file(&syntax);
    let mut calls = CallAudit {
        opaque: false,
        in_function: false,
        type_attributes: false,
        closings: BTreeSet::new(),
    };
    calls.visit_file(&syntax);
    // AncestorAudit rejects attributes, extern crates and item macros across
    // the entire ancestor, independent of any builtin-name imports.
    let mut ancestor_audit = CallAncestorAudit { opaque: false };
    ancestor_audit.visit_file(&ancestor_syntax);
    let modules: Vec<_> = ancestor_syntax
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Mod(m) if m.ident == module => Some(m),
            _ => None,
        })
        .collect();
    if calls.opaque
        || ancestor_audit.opaque
        || module.is_empty()
        || modules.len() != 1
        || modules[0].content.is_some()
        || !docs_only(&modules[0].attrs)
    {
        calls.closings.clear();
    }
    if audit.opaque || audit.item_macro {
        fields.clear();
    }
    if audit.opaque {
        audit.closings.clear();
        audit.declarations.clear();
    }
    Ok(Evidence {
        fields,
        closings: calls.closings,
        opaque: audit.opaque_lines,
        declarations: audit.declarations,
    })
}

#[test]
fn syntax_query() {
    if std::env::var_os("SDF_COVERAGE_SYNTAX_QUERY").is_none() {
        let sample = "#[derive(Debug)]\npub struct World {\n/// docs\npub extensions: Vec<Thing>,\n}\nfn f() { x.extend(\n vec![]\n); }";
        let evidence = classify(sample, "mod types;", "types").unwrap();
        assert_eq!(evidence.fields, BTreeSet::from([4]));
        assert!(evidence.closings.contains(&8));
        return;
    }
    let mut request = String::new();
    io::stdin().read_to_string(&mut request).unwrap();
    let request: serde_json::Value = serde_json::from_str(&request).unwrap();
    let source = request["source"].as_str().unwrap();
    let ancestor = request["ancestor"].as_str().unwrap();
    let module = request["module"].as_str().unwrap();
    let evidence = classify(source, ancestor, module).expect("source must parse");
    println!(
        "SDF_COVERAGE_SYNTAX_V1={}",
        serde_json::json!({"source": source, "ancestor": ancestor, "module": module, "helper_source": include_str!("coverage_syntax.rs"), "fields": evidence.fields, "closings": evidence.closings, "opaque": evidence.opaque, "declarations": evidence.declarations})
    );
}

#[test]
fn syntax_evidence_refuses_opaque_attributes_and_macro_tokens() {
    for source in [
        "#[cfg(feature=\"x\")] pub struct World { pub extensions: Vec<Thing>, }",
        "#[derive(Custom)] pub struct World { pub extensions: Vec<Thing>, }",
        "#[derive(Debug, Clone)] pub struct World { pub extensions: Vec<Thing>, }",
        "pub struct World { #[cfg(feature=\"x\")] pub extensions: Vec<Thing>, }",
        "pub struct World { #[doc = include_str!(\"x\")] pub extensions: Vec<Thing>, }",
        "macro_rules! m { () => { pub struct World { pub extensions: Vec<Thing>, } } }",
        "const RAW: &str = r#\"pub struct World { pub extensions: Vec<Thing>, }\"#;",
        "mod nested { pub struct World { pub extensions: Vec<Thing>, } }",
    ] {
        assert!(
            classify(source, "mod types;", "types")
                .unwrap()
                .fields
                .is_empty(),
            "{source}"
        );
    }
    let evidence = classify(
        "#[cfg(feature=\"x\")] fn f() { x.extend(\n vec![]\n); }",
        "mod types;",
        "types",
    )
    .unwrap();
    assert!(evidence.closings.is_empty());
    assert!(classify("pub struct Broken {", "mod types;", "types").is_err());
}

#[test]
fn syntax_evidence_marks_macro_and_multiline_string_contents_opaque() {
    let source = "macro_rules! m {\n () => {\n  // inside macro\n  pub struct World { pub extensions: Vec<Thing>, }\n };\n}\nconst RAW: &str = r#\"\n// inside string\n);\n\"#;\n";
    let evidence = classify(source, "mod types;", "types").unwrap();
    assert!(evidence.fields.is_empty());
    assert!(evidence.closings.is_empty());
    assert!(evidence.opaque.contains(&3));
    assert!(evidence.opaque.contains(&4));
    assert!(evidence.opaque.contains(&8));
    assert!(evidence.opaque.contains(&9));
}

#[test]
fn standard_debug_scope_fails_closed_on_shadowing_and_opaque_ancestry() {
    let field = "#[derive(Debug)]\npub struct World {\npub extensions: Vec<Thing>,\n}";
    assert_eq!(
        classify(field, "mod types;", "types").unwrap().fields,
        BTreeSet::from([3])
    );
    for source in [
        format!("use shadow_derive::Debug;\n{field}"),
        format!("use shadow_derive::Clone;\n{field}"),
        format!("use shadow_derive::{{Other as Debug}};\n{field}"),
        format!("use shadow_derive::*;\n{field}"),
        format!("macro_rules! m {{ () => {{}} }}\n{field}"),
        format!("#[cfg(feature=\"x\")] use shadow_derive::Debug;\n{field}"),
    ] {
        assert!(
            classify(&source, "mod types;", "types")
                .unwrap()
                .fields
                .is_empty(),
            "{source}"
        );
    }
    for ancestor in [
        "#[macro_use] mod earlier; mod types;",
        "macro_rules! m { () => {} } mod types;",
        "mod types; #[macro_use] extern crate shadow;",
        "mod types; #[macro_use] mod shadow;",
        "mod types; #[shadow::expand] fn later() {}",
        "#[path=\"other.rs\"] mod types;",
        "mod other;",
        "mod types; mod types;",
    ] {
        assert!(
            classify(field, ancestor, "types")
                .unwrap()
                .fields
                .is_empty(),
            "{ancestor}"
        );
    }
}

#[test]
fn spans_follow_lf_when_strings_contain_unicode_separators() {
    let source = "const S: &str = \"a\u{2028}b\";\n#[derive(Debug)]\npub struct World {\n    pub extensions: Vec<Thing>,\n}\n";
    let evidence = classify(source, "mod types;", "types").unwrap();
    assert_eq!(evidence.fields, BTreeSet::from([4]));
}

#[test]
fn spans_keep_crlf_lines_and_lone_carriage_return_fails_closed() {
    let source =
        "#[derive(Debug)]\r\npub struct World {\r\n    pub extensions: Vec<Thing>,\r\n}\r\n";
    let evidence = classify(source, "mod types;", "types").unwrap();
    assert_eq!(evidence.fields, BTreeSet::from([3]));
    let lone_cr = "const S: &str = r#\"a\rb\"#;\n#[derive(Debug)]\npub struct World { pub extensions: Vec<Thing>, }";
    assert!(classify(lone_cr, "mod types;", "types").is_err());
}

#[test]
fn opaque_attributes_and_shadowed_derives_never_classify_other_lines() {
    let ancestor = "mod types;";
    for source in [
        "use shadow::default;\n#[default] fn f() { drop(\n 1\n); }",
        "use shadow::Debug;\n#[derive(Debug)] struct X;\nfn f() { drop(\n 1\n); }",
        "#[derive(Default)] enum E { #[default] A { #[default] x: u8 } }\nfn f() { drop(\n 1\n); }",
    ] {
        let evidence = classify(source, ancestor, "types").unwrap();
        assert!(evidence.declarations.is_empty());
    }
}

#[test]
fn compiler_confirms_shadowed_macros_are_not_classified() {
    use std::{fs, process::Command};
    let dir = std::env::temp_dir().join(format!("sdf-syntax-shadow-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let macro_source = r#"extern crate proc_macro;
#[proc_macro_derive(Debug)]
pub fn debug(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    "compile_error!(\"custom Debug selected\");".parse().unwrap()
}
#[proc_macro_attribute]
pub fn default(_: proc_macro::TokenStream, _: proc_macro::TokenStream) -> proc_macro::TokenStream {
    "compile_error!(\"custom default selected\");".parse().unwrap()
}"#;
    fs::write(dir.join("shadow.rs"), macro_source).unwrap();
    let library = dir.join(format!(
        "{}shadow{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    ));
    assert!(
        Command::new("rustc")
            .args(["--crate-name", "shadow", "--crate-type", "proc-macro"])
            .arg(dir.join("shadow.rs"))
            .arg("-o")
            .arg(&library)
            .status()
            .unwrap()
            .success()
    );
    let ancestor = "mod types;\n#[macro_use] extern crate shadow;";
    let field = "type Thing = String;\n#[derive(Debug)]\npub struct World {\npub extensions: Vec<Thing>,\n}";
    fs::write(dir.join("lib.rs"), ancestor).unwrap();
    fs::write(dir.join("types.rs"), field).unwrap();
    let output = Command::new("rustc")
        .args(["--crate-type", "lib", "--edition", "2021"])
        .arg(dir.join("lib.rs"))
        .arg("--extern")
        .arg(format!("shadow={}", library.display()))
        .arg("-o")
        .arg(dir.join("out.rlib"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("custom Debug selected")
    );
    assert!(
        classify(field, ancestor, "types")
            .unwrap()
            .fields
            .is_empty()
    );
    let attribute = "use shadow::default;\n#[default] fn f() { std::mem::drop(\n 1\n); }";
    fs::write(dir.join("lib.rs"), attribute).unwrap();
    let output = Command::new("rustc")
        .args(["--crate-type", "lib", "--edition", "2021"])
        .arg(dir.join("lib.rs"))
        .arg("--extern")
        .arg(format!("shadow={}", library.display()))
        .arg("-o")
        .arg(dir.join("out.rlib"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("custom default selected")
    );
    let evidence = classify(attribute, "mod types;", "types").unwrap();
    assert!(evidence.closings.is_empty() && evidence.declarations.is_empty());
    let adding_derive = r#"extern crate proc_macro;
#[proc_macro_derive(Debug)]
pub fn debug(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    "const DERIVE_SELECTED: i32 = 42;".parse().unwrap()
}"#;
    fs::write(dir.join("shadow.rs"), adding_derive).unwrap();
    assert!(
        Command::new("rustc")
            .args(["--crate-name", "shadow", "--crate-type", "proc-macro"])
            .arg(dir.join("shadow.rs"))
            .arg("-o")
            .arg(&library)
            .status()
            .unwrap()
            .success()
    );
    let src = "use shadow::Debug;\n#[derive(Debug)] struct X;\nfn main() { assert_eq!(DERIVE_SELECTED, 42); std::mem::drop(\n 1\n); }";
    fs::write(dir.join("lib.rs"), src).unwrap();
    let executable = dir.join("probe");
    let output = Command::new("rustc")
        .args(["--edition", "2021"])
        .arg(dir.join("lib.rs"))
        .arg("--extern")
        .arg(format!("shadow={}", library.display()))
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(Command::new(executable).status().unwrap().success());
    assert!(
        classify(src, "mod types;", "types")
            .unwrap()
            .closings
            .contains(&5)
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn call_closing_uses_enclosing_context_not_unrelated_derive_resolution() {
    let source =
        "use crate::*;\n#[derive(Debug)] struct X;\nimpl X { fn f(&self) { self.g(\n 1\n); } }";
    let evidence = classify(source, "mod types;", "types").unwrap();
    assert!(evidence.closings.contains(&5));
    assert!(evidence.fields.is_empty());
    for changed in [
        source.replace("fn f", "#[default] fn f"),
        source.replace("fn f", "#[cfg(any())] fn f"),
        source.replace("impl X", "#[shadow::transform] impl X"),
        source.replace("impl X", "mod shadow { #![cfg(any())] } impl X"),
    ] {
        assert!(
            classify(&changed, "mod types;", "types")
                .unwrap()
                .closings
                .is_empty()
        );
    }
    assert!(
        classify(source, "#[cfg(any())] mod types;", "types")
            .unwrap()
            .closings
            .is_empty()
    );
    assert!(
        classify(
            source,
            "mod types; #[shadow::transform] fn later() {}",
            "types"
        )
        .unwrap()
        .closings
        .is_empty()
    );
}
