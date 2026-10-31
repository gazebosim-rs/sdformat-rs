// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Fail-closed syntax evidence for the changed-line coverage gate.
use std::collections::BTreeSet;
use std::io::{self, Read};
use syn::{
    GenericArgument, Item, PathArguments, Type, Visibility,
    ext::IdentExt,
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

// Keep the historical Vec derive boundary; only the two new field shapes
// use this broader set of known built-ins under the same proven scope.
fn string_field_derives_or_docs(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().all(|attribute| {
        docs_only(std::slice::from_ref(attribute))
            || (attribute.path().is_ident("derive")
                && attribute
                    .parse_args_with(
                        syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                    )
                    .is_ok_and(|paths| {
                        !paths.is_empty()
                            && paths.iter().all(|path| {
                                ["Debug", "Default", "Clone", "Copy", "PartialEq", "Eq"]
                                    .iter()
                                    .any(|name| path.is_ident(name))
                            })
                    }))
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
fn public_string_field(field: &syn::Field) -> bool {
    if !matches!(field.vis, Visibility::Public(_)) {
        return false;
    }
    let Type::Path(ty) = &field.ty else {
        return false;
    };
    if ty.qself.is_some() || ty.path.leading_colon.is_some() || ty.path.segments.len() != 1 {
        return false;
    }
    let segment = &ty.path.segments[0];
    if segment.ident == "String" && matches!(segment.arguments, PathArguments::None) {
        return true;
    }
    if segment.ident != "Option" {
        return false;
    }
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return false;
    };
    if args.colon2_token.is_some() || args.args.len() != 1 || args.args.trailing_punct() {
        return false;
    }
    let GenericArgument::Type(Type::Path(inner)) = &args.args[0] else {
        return false;
    };
    inner.qself.is_none()
        && inner.path.leading_colon.is_none()
        && inner.path.segments.len() == 1
        && inner.path.segments[0].ident == "String"
        && matches!(inner.path.segments[0].arguments, PathArguments::None)
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
    else_delimiters: BTreeSet<usize>,
    wildcard_prefixes: BTreeSet<usize>,
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
    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        if self.in_function
            && arm.attrs.is_empty()
            && let syn::Pat::Wild(pattern) = &arm.pat
            && pattern.attrs.is_empty()
            && let syn::Expr::Block(block) = &*arm.body
            && block.attrs.is_empty()
            && block.label.is_none()
        {
            let line = pattern.underscore_token.span.start().line;
            if arm.fat_arrow_token.span().start().line == line
                && block.block.brace_token.span.open().end().line == line
            {
                self.wildcard_prefixes.insert(line);
            }
        }
        visit::visit_arm(self, arm);
    }
    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        if self.in_function
            && docs_only(&expression.attrs)
            && let Some((token, branch)) = &expression.else_branch
            && let syn::Expr::Block(block) = &**branch
            && docs_only(&block.attrs)
        {
            let close = expression.then_branch.brace_token.span.close();
            let open = block.block.brace_token.span.open();
            let line = token.span.start().line;
            if close.start().line == line
                && close.end().line == line
                && token.span.end().line == line
                && open.start().line == line
                && open.end().line == line
                && close.end().column <= token.span.start().column
                && token.span.end().column <= open.start().column
            {
                self.else_delimiters.insert(line);
            }
        }
        visit::visit_expr_if(self, expression);
    }
    fn visit_expr_call(&mut self, expression: &'ast syn::ExprCall) {
        if self.in_function {
            self.closings.insert(expression.span().end().line);
        }
        visit::visit_expr_call(self, expression);
    }
    fn visit_expr_struct(&mut self, expression: &'ast syn::ExprStruct) {
        if self.in_function && docs_only(&expression.attrs) {
            for pair in expression.fields.pairs() {
                let (field, comma) = pair.into_tuple();
                if let syn::Expr::If(branch) = &field.expr
                    && docs_only(&field.attrs)
                    && docs_only(&branch.attrs)
                    && let Some((_, tail)) = &branch.else_branch
                    && let syn::Expr::Block(block) = &**tail
                    && docs_only(&block.attrs)
                    && comma.is_some_and(|comma| {
                        comma.span.start().line == block.block.brace_token.span.close().start().line
                    })
                {
                    self.closings
                        .insert(block.block.brace_token.span.close().start().line);
                }
            }
        }
        visit::visit_expr_struct(self, expression);
    }
    fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
        if self.in_function {
            self.closings.insert(expression.span().end().line);
        }
        visit::visit_expr_method_call(self, expression);
    }
}
fn plain_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Reference(value) => value.mutability.is_none() && plain_type(&value.elem),
        syn::Type::Path(value) if value.qself.is_none() => {
            value
                .path
                .segments
                .iter()
                .all(|segment| match &segment.arguments {
                    PathArguments::None => true,
                    PathArguments::AngleBracketed(args) => args.args.iter().all(|arg| match arg {
                        syn::GenericArgument::Type(ty) => plain_type(ty),
                        syn::GenericArgument::Lifetime(_) => true,
                        _ => false,
                    }),
                    _ => false,
                })
        }
        _ => false,
    }
}
// A declaration contains no runtime body only when every type is syntax-only.
// Arrays are limited to literal integer lengths, never arbitrary const syntax.
fn concrete_type(ty: &Type) -> bool {
    match ty {
        Type::Array(array) => {
            concrete_type(&array.elem)
                && matches!(&array.len, syn::Expr::Lit(lit)
                    if lit.attrs.is_empty() && matches!(lit.lit, syn::Lit::Int(_)))
        }
        _ => plain_type(ty),
    }
}
fn exclusive_span(source: &str, span: proc_macro2::Span, suffix: &str) -> bool {
    let (start, end) = (span.start(), span.end());
    let lines: Vec<_> = source.split('\n').collect();
    lines[start.line - 1]
        .chars()
        .take(start.column)
        .all(char::is_whitespace)
        && lines[end.line - 1]
            .chars()
            .skip(end.column)
            .collect::<String>()
            .trim()
            == suffix
}
fn plain_declarations(
    source: &str,
    file: &syn::File,
    ancestor: &syn::File,
    module: &str,
) -> BTreeSet<usize> {
    let mut out = BTreeSet::new();
    if !standard_debug_scope(file, ancestor, module) {
        return out;
    }
    let lines: Vec<_> = source.split('\n').collect();
    for item in &file.items {
        match item {
            Item::Struct(item)
                if string_field_derives_or_docs(&item.attrs)
                    && item.generics.params.is_empty()
                    && item.generics.where_clause.is_none() =>
            {
                let syn::Fields::Named(fields) = &item.fields else {
                    continue;
                };
                // Each field is proved independently; an opaque neighbour must
                // not lend it an unexamined type or attribute.
                for pair in fields.named.pairs() {
                    let (field, comma) = pair.into_tuple();
                    if !docs_only(&field.attrs)
                        || !matches!(field.vis, Visibility::Public(_))
                        || !concrete_type(&field.ty)
                    {
                        continue;
                    }
                    let Some(span) = field.vis.span().join(field.ty.span()) else {
                        continue;
                    };
                    if span.start().line == span.end().line
                        && comma.is_some_and(|comma| comma.span.start().line == span.end().line)
                        && exclusive_span(source, span, ",")
                    {
                        out.insert(span.start().line);
                    }
                }
                // Headers/derive lines are syntax-only, not the generated
                // executable bodies; measured DA and regions remain mandatory.
                if fields
                    .named
                    .iter()
                    .all(|f| docs_only(&f.attrs) && concrete_type(&f.ty))
                {
                    for attr in &item.attrs {
                        if attr.path().is_ident("derive") && exclusive_span(source, attr.span(), "")
                        {
                            out.extend(attr.span().start().line..=attr.span().end().line);
                        }
                    }
                    let start = item.struct_token.span.start();
                    let open = fields.brace_token.span.open();
                    if start.line == open.end().line
                        && lines[start.line - 1].trim() == format!("pub struct {} {{", item.ident)
                        && matches!(item.vis, Visibility::Public(_))
                    {
                        out.insert(start.line);
                    }
                }
            }
            Item::Enum(item)
                if string_field_derives_or_docs(&item.attrs)
                    && matches!(item.vis, Visibility::Public(_))
                    && item.generics.params.is_empty()
                    && item.generics.where_clause.is_none()
                    && item.variants.iter().all(|variant| {
                        docs_only(&variant.attrs)
                            && matches!(variant.fields, syn::Fields::Unit)
                            && variant.discriminant.is_none()
                    }) =>
            {
                for attr in &item.attrs {
                    if attr.path().is_ident("derive") && exclusive_span(source, attr.span(), "") {
                        out.extend(attr.span().start().line..=attr.span().end().line);
                    }
                }
                let start = item.enum_token.span.start();
                let open = item.brace_token.span.open();
                if start.line == open.end().line
                    && lines[start.line - 1].trim() == format!("pub enum {} {{", item.ident)
                {
                    out.insert(start.line);
                }
                for pair in item.variants.pairs() {
                    let (variant, comma) = pair.into_tuple();
                    let span = variant.ident.span();
                    if comma.is_some_and(|comma| comma.span.start().line == span.end().line)
                        && exclusive_span(source, span, ",")
                    {
                        out.insert(span.start().line);
                    }
                }
            }
            Item::Impl(item)
                if docs_only(&item.attrs)
                    && item.unsafety.is_none()
                    && item.modifiers.require_empty().is_ok()
                    && item.generics.params.is_empty()
                    && item.generics.where_clause.is_none()
                    && plain_type(&item.self_ty) =>
            {
                if let Some((path, _)) = &item.trait_
                    && path
                        .segments
                        .iter()
                        .any(|p| !matches!(p.arguments, PathArguments::None))
                {
                    continue;
                }
                let start = item.impl_token.span.start();
                let open = item.brace_token.span.open();
                if start.line == open.end().line {
                    let end = open.end();
                    let line = lines[start.line - 1];
                    // Use AST token positions, preserving visibility of any
                    // shared-line body rather than matching an impl regex.
                    if line.chars().take(start.column).all(char::is_whitespace)
                        && line.chars().skip(end.column).all(char::is_whitespace)
                    {
                        out.insert(start.line);
                    }
                }
            }
            _ => {}
        }
    }
    out
}
fn candidate_lines(source: &str) -> BTreeSet<usize> {
    let file = syn::parse_file(source).unwrap();
    let lines: Vec<_> = source.split('\n').collect();
    let mut out = BTreeSet::new();
    let mut items: Vec<&Item> = file.items.iter().collect();
    for item in &file.items {
        if let Item::Fn(function) = item
            && function.attrs.is_empty()
        {
            items.extend(function.block.stmts.iter().filter_map(|stmt| match stmt {
                syn::Stmt::Item(item) => Some(item),
                _ => None,
            }));
        }
    }
    for item in items {
        let supported = match item {
            Item::Use(item) => item.attrs.is_empty(),
            Item::Static(item) => {
                let syn::Expr::Call(call) = &*item.expr else {
                    continue;
                };
                let syn::Expr::Path(path) = &*call.func else {
                    continue;
                };
                item.attrs.is_empty()
                    && plain_type(&item.ty)
                    && call.attrs.is_empty()
                    && path.attrs.is_empty()
                    && path.qself.is_none()
                    && call.args.is_empty()
                    && path
                        .path
                        .segments
                        .iter()
                        .all(|segment| matches!(segment.arguments, PathArguments::None))
                    && item.span().start().line == item.span().end().line
            }
            _ => false,
        };
        if !supported {
            continue;
        }
        let span = item.span();
        let (start, end) = (span.start(), span.end());
        if !lines[start.line - 1]
            .chars()
            .take(start.column)
            .all(char::is_whitespace)
            || !lines[end.line - 1]
                .chars()
                .skip(end.column)
                .all(char::is_whitespace)
        {
            continue;
        }
        out.extend(start.line..=end.line);
    }
    out
}

fn context_lines(source: &str, ancestor: &str, module: &str) -> BTreeSet<usize> {
    let Ok(file) = syn::parse_file(source) else {
        return BTreeSet::new();
    };
    let Ok(parent) = syn::parse_file(ancestor) else {
        return BTreeSet::new();
    };
    if !docs_only(&file.attrs)
        || !docs_only(&parent.attrs)
        || module.is_empty()
        || parent.items.iter().any(|item| {
            matches!(
                item,
                Item::Macro(_) | Item::ExternCrate(_) | Item::Verbatim(_)
            )
        })
    {
        return BTreeSet::new();
    }
    let modules: Vec<_> = parent
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Mod(value) if value.ident == module => Some(value),
            _ => None,
        })
        .collect();
    if modules.len() != 1 || !modules[0].attrs.is_empty() || modules[0].content.is_some() {
        return BTreeSet::new();
    }
    candidate_lines(source)
}

// This is a closed audit of this crate's two public glob exports, not a Rust
// name resolver. Any additional export edge or binding requires review.
const CONTEXT_MACROS: &[&str] = &[
    "format",
    "vec",
    "write",
    "matches",
    "unreachable",
    "include_str",
];
fn reserved_constructor_name(name: &str) -> bool {
    [
        "Ok",
        "Scene",
        "SphericalCoordinates",
        "Gui",
        "Result",
        "std",
        "Debug",
        "Clone",
        "Copy",
        "Default",
        "default",
        "PartialEq",
        "Eq",
        "allow",
    ]
    .contains(&name)
        || CONTEXT_MACROS.contains(&name)
}
fn safe_macro_tokens(tokens: proc_macro2::TokenStream) -> bool {
    use proc_macro2::TokenTree;
    let tokens: Vec<_> = tokens.into_iter().collect();
    let mut at = 0;
    while at < tokens.len() {
        match &tokens[at] {
            TokenTree::Ident(name)
                if [
                    "macro_rules",
                    "macro_export",
                    "macro_use",
                    "macro",
                    "extern",
                    "Ok",
                    "Scene",
                    "SphericalCoordinates",
                    "Gui",
                ]
                .contains(&name.unraw().to_string().as_str()) =>
            {
                return false;
            }
            TokenTree::Ident(name)
                if tokens
                    .get(at + 1)
                    .is_some_and(|t| matches!(t, TokenTree::Punct(p) if p.as_char() == '!')) =>
            {
                if !CONTEXT_MACROS.contains(&name.unraw().to_string().as_str())
                    || at > 0
                        && matches!(&tokens[at - 1], TokenTree::Punct(p) if p.as_char() == ':')
                {
                    return false;
                }
                let Some(TokenTree::Group(group)) = tokens.get(at + 2) else {
                    return false;
                };
                if !safe_macro_tokens(group.stream()) {
                    return false;
                }
                at += 3;
                continue;
            }
            TokenTree::Punct(p) if ['#', '!'].contains(&p.as_char()) => return false,
            TokenTree::Group(group) if !safe_macro_tokens(group.stream()) => return false,
            _ => {}
        }
        at += 1;
    }
    true
}
fn import_mentions_reserved(tree: &syn::UseTree) -> bool {
    fn binding(tree: &syn::UseTree, parent: Option<&syn::Ident>) -> bool {
        match tree {
            syn::UseTree::Path(path) => binding(&path.tree, Some(&path.ident)),
            // `use hidden::Scene::{self}` binds Scene, not the word self.
            syn::UseTree::Name(name) if name.ident == "self" => {
                parent.is_none_or(|name| reserved_constructor_name(&name.unraw().to_string()))
            }
            syn::UseTree::Name(name) => reserved_constructor_name(&name.ident.unraw().to_string()),
            syn::UseTree::Rename(name) => {
                reserved_constructor_name(&name.rename.unraw().to_string())
            }
            syn::UseTree::Group(group) => group.items.iter().any(|tree| binding(tree, parent)),
            syn::UseTree::Glob(_) => true,
        }
    }
    binding(tree, None)
}
fn exact_glob(tree: &syn::UseTree, name: &str) -> bool {
    matches!(tree, syn::UseTree::Path(path) if path.ident == name && matches!(&*path.tree, syn::UseTree::Glob(_)))
}
struct ConstructorBoundary<'a> {
    role: &'a str,
    bad: bool,
    root_globs: BTreeSet<String>,
    target_globs: usize,
}
impl ConstructorBoundary<'_> {
    fn binding(&mut self, name: &syn::Ident, scene_declaration: bool) {
        if [
            "Ok",
            "Scene",
            "SphericalCoordinates",
            "Gui",
            "Result",
            "std",
        ]
        .contains(&name.unraw().to_string().as_str())
            && !(scene_declaration
                && self.role == "types"
                && ["Scene", "SphericalCoordinates", "Gui"]
                    .contains(&name.unraw().to_string().as_str()))
        {
            self.bad = true;
        }
    }
}
impl<'ast> Visit<'ast> for ConstructorBoundary<'_> {
    fn visit_item(&mut self, item: &'ast Item) {
        if matches!(item, Item::Verbatim(_) | Item::ForeignMod(_)) {
            self.bad = true;
        } else {
            visit::visit_item(self, item);
        }
    }
    fn visit_expr(&mut self, item: &'ast syn::Expr) {
        if matches!(item, syn::Expr::Verbatim(_)) {
            self.bad = true;
        } else {
            visit::visit_expr(self, item);
        }
    }
    fn visit_pat(&mut self, item: &'ast syn::Pat) {
        if matches!(item, syn::Pat::Verbatim(_)) {
            self.bad = true;
        } else {
            visit::visit_pat(self, item);
        }
    }
    fn visit_impl_item_const(&mut self, item: &'ast syn::ImplItemConst) {
        self.binding(&item.ident, false);
        visit::visit_impl_item_const(self, item);
    }
    fn visit_impl_item_type(&mut self, item: &'ast syn::ImplItemType) {
        self.binding(&item.ident, false);
        visit::visit_impl_item_type(self, item);
    }

    fn visit_attribute(&mut self, attr: &'ast syn::Attribute) {
        let standard = docs_only(std::slice::from_ref(attr))
            || string_field_derives_or_docs(std::slice::from_ref(attr))
            || attr.path().is_ident("default") && matches!(attr.meta, syn::Meta::Path(_))
            || attr.path().is_ident("allow")
                && attr
                    .parse_args_with(
                        syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                    )
                    .is_ok();
        self.bad |= !standard;
    }
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        if self.role == "root"
            && item.leading_colon.is_none()
            && matches!(item.vis, Visibility::Public(_))
        {
            for name in ["types", "camera"] {
                if exact_glob(&item.tree, name) {
                    if !docs_only(&item.attrs) || !self.root_globs.insert(name.into()) {
                        self.bad = true;
                    }
                    return;
                }
            }
        }
        if self.role == "parse"
            && item.leading_colon.is_none()
            && matches!(item.vis, Visibility::Inherited)
            && item.attrs.is_empty()
            && exact_glob(&item.tree, "crate")
        {
            self.target_globs += 1;
            return;
        }
        if self.role != "root" && matches!(item.vis, Visibility::Public(_)) {
            self.bad = true;
        }
        self.bad |= import_mentions_reserved(&item.tree) || !docs_only(&item.attrs);
    }
    fn visit_item_macro(&mut self, _: &'ast syn::ItemMacro) {
        self.bad = true;
    }
    fn visit_item_extern_crate(&mut self, _: &'ast syn::ItemExternCrate) {
        self.bad = true;
    }
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        self.bad |= item.path.segments.len() != 1
            || !CONTEXT_MACROS.iter().any(|name| item.path.is_ident(name))
            || !safe_macro_tokens(item.tokens.clone());
    }
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        self.binding(&item.ident, true);
        visit::visit_item_struct(self, item);
    }
    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        self.binding(&item.ident, false);
        visit::visit_item_enum(self, item);
    }
    fn visit_item_union(&mut self, item: &'ast syn::ItemUnion) {
        self.binding(&item.ident, false);
        visit::visit_item_union(self, item);
    }
    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        self.binding(&item.ident, false);
        visit::visit_item_type(self, item);
    }
    fn visit_item_const(&mut self, item: &'ast syn::ItemConst) {
        self.binding(&item.ident, false);
        visit::visit_item_const(self, item);
    }
    fn visit_item_static(&mut self, item: &'ast syn::ItemStatic) {
        self.binding(&item.ident, false);
        visit::visit_item_static(self, item);
    }
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        self.binding(&item.ident, false);
        if self.role != "root" {
            self.bad = true;
        }
        visit::visit_item_mod(self, item);
    }
    fn visit_signature(&mut self, item: &'ast syn::Signature) {
        self.binding(&item.ident, false);
        visit::visit_signature(self, item);
    }
    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        self.binding(&item.ident, false);
        visit::visit_item_trait(self, item);
    }
    fn visit_type_param(&mut self, item: &'ast syn::TypeParam) {
        self.binding(&item.ident, false);
        visit::visit_type_param(self, item);
    }
    fn visit_const_param(&mut self, item: &'ast syn::ConstParam) {
        self.binding(&item.ident, false);
        visit::visit_const_param(self, item);
    }
    fn visit_pat_ident(&mut self, item: &'ast syn::PatIdent) {
        self.binding(&item.ident, false);
        visit::visit_pat_ident(self, item);
    }
}
fn aggregate_prefixes(
    source: &str,
    ancestor: &str,
    module: &str,
    exports: &std::collections::BTreeMap<String, String>,
) -> BTreeSet<usize> {
    ["Scene", "SphericalCoordinates", "Gui"]
        .into_iter()
        .flat_map(|constructor| {
            aggregate_prefixes_for(source, ancestor, module, exports, constructor)
        })
        .collect()
}

fn aggregate_prefixes_for(
    source: &str,
    ancestor: &str,
    module: &str,
    exports: &std::collections::BTreeMap<String, String>,
    constructor: &str,
) -> BTreeSet<usize> {
    let empty = BTreeSet::new();
    if module != "parse"
        || exports.keys().map(String::as_str).collect::<Vec<_>>() != ["camera", "types"]
    {
        return empty;
    }
    let Ok(root) = syn::parse_file(ancestor) else {
        return empty;
    };
    let Ok(target) = syn::parse_file(source) else {
        return empty;
    };
    let Ok(types) = syn::parse_file(&exports["types"]) else {
        return empty;
    };
    let Ok(camera) = syn::parse_file(&exports["camera"]) else {
        return empty;
    };
    for (role, file) in [
        ("root", &root),
        ("parse", &target),
        ("types", &types),
        ("camera", &camera),
    ] {
        if !docs_only(&file.attrs) {
            return empty;
        }
        let mut audit = ConstructorBoundary {
            role,
            bad: false,
            root_globs: BTreeSet::new(),
            target_globs: 0,
        };
        audit.visit_file(file);
        if audit.bad
            || role == "root"
                && audit.root_globs != BTreeSet::from(["types".into(), "camera".into()])
            || role == "parse" && audit.target_globs != 1
        {
            return empty;
        }
    }
    for module in ["parse", "types", "camera"] {
        let modules: Vec<_> = root
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Mod(item) if item.ident == module => Some(item),
                _ => None,
            })
            .collect();
        if modules.len() != 1 || modules[0].content.is_some() || !docs_only(&modules[0].attrs) {
            return empty;
        }
    }
    let scenes: Vec<_> = types
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Struct(item) if item.ident == constructor => Some(item),
            _ => None,
        })
        .collect();
    if scenes.len() != 1 {
        return empty;
    }
    let scene = scenes[0];
    let syn::Fields::Named(fields) = &scene.fields else {
        return empty;
    };
    if !matches!(scene.vis, Visibility::Public(_))
        || !string_field_derives_or_docs(&scene.attrs)
        || !scene.generics.params.is_empty()
        || scene.generics.where_clause.is_some()
        || fields.named.iter().any(|field| {
            !matches!(field.vis, Visibility::Public(_))
                || !docs_only(&field.attrs)
                || !concrete_type(&field.ty)
        })
    {
        return empty;
    }
    let names: BTreeSet<_> = fields
        .named
        .iter()
        .map(|field| field.ident.as_ref().unwrap().to_string())
        .collect();
    if names.len() != fields.named.len() {
        return empty;
    }
    fn ordinary_signature(sig: &syn::Signature) -> bool {
        sig.generics.params.is_empty()
            && sig.generics.where_clause.is_none()
            && sig.asyncness.is_none()
            && sig.constness.is_none()
            && matches!(sig.safety, syn::Safety::Default)
            && sig.abi.is_none()
    }
    struct Prefixes<'a> {
        source: &'a str,
        fields: &'a BTreeSet<String>,
        constructor: &'a str,
        in_function: bool,
        lines: BTreeSet<usize>,
    }
    impl<'ast> Visit<'ast> for Prefixes<'_> {
        fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
            if self.in_function || !docs_only(&item.attrs) || !ordinary_signature(&item.sig) {
                return;
            }
            self.in_function = true;
            self.visit_block(&item.block);
            self.in_function = false;
        }
        fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
            if !docs_only(&item.attrs)
                || item.trait_.is_some()
                || item
                    .generics
                    .params
                    .iter()
                    .any(|p| !matches!(p, syn::GenericParam::Lifetime(_)))
            {
                return;
            }
            visit::visit_item_impl(self, item);
        }
        fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
            if !docs_only(&item.attrs) || !ordinary_signature(&item.sig) {
                return;
            }
            self.in_function = true;
            self.visit_block(&item.block);
            self.in_function = false;
        }
        fn visit_type(&mut self, _: &'ast Type) {}
        fn visit_item_const(&mut self, _: &'ast syn::ItemConst) {}
        fn visit_item_static(&mut self, _: &'ast syn::ItemStatic) {}
        fn visit_impl_item_const(&mut self, _: &'ast syn::ImplItemConst) {}
        fn visit_trait_item_const(&mut self, _: &'ast syn::TraitItemConst) {}
        fn visit_generic_argument(&mut self, argument: &'ast GenericArgument) {
            if !matches!(argument, GenericArgument::Const(_)) {
                visit::visit_generic_argument(self, argument);
            }
        }
        fn visit_expr_const(&mut self, _: &'ast syn::ExprConst) {}
        fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {}
        fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
            if self.in_function
                && call.attrs.is_empty()
                && call.args.len() == 1
                && let syn::Expr::Path(callee) = &*call.func
                && callee.attrs.is_empty()
                && callee.qself.is_none()
                && callee.path.is_ident("Ok")
                && let syn::Expr::Struct(value) = &call.args[0]
                && value.attrs.is_empty()
                && value.qself.is_none()
                && value.path.is_ident(self.constructor)
                && value.rest.is_none()
                && value.fields.iter().all(|f| f.attrs.is_empty())
            {
                let names: BTreeSet<_> = value
                    .fields
                    .iter()
                    .filter_map(|f| match &f.member {
                        syn::Member::Named(n) => Some(n.to_string()),
                        _ => None,
                    })
                    .collect();
                let start = call.func.span().start();
                let end = value.brace_token.span.open().end();
                if names == *self.fields
                    && names.len() == value.fields.len()
                    && start.line == end.line
                {
                    let line = self.source.split('\n').nth(start.line - 1).unwrap();
                    if line.chars().take(start.column).all(char::is_whitespace)
                        && line.chars().skip(end.column).all(char::is_whitespace)
                    {
                        self.lines.insert(start.line);
                    }
                }
            }
            visit::visit_expr_call(self, call);
        }
    }
    let mut visitor = Prefixes {
        source,
        fields: &names,
        constructor,
        in_function: false,
        lines: BTreeSet::new(),
    };
    visitor.visit_file(&target);
    visitor.lines
}

struct Evidence {
    fields: BTreeSet<usize>,
    closings: BTreeSet<usize>,
    else_delimiters: BTreeSet<usize>,
    opaque: BTreeSet<usize>,
    declarations: BTreeSet<usize>,
    items: BTreeSet<usize>,
}
fn classify(source: &str, ancestor: &str, module: &str) -> Result<Evidence, syn::Error> {
    let syntax = syn::parse_file(source)?;
    let ancestor_syntax = syn::parse_file(ancestor)?;
    let mut fields = BTreeSet::new();
    let standard_scope = standard_debug_scope(&syntax, &ancestor_syntax, module);
    if standard_scope {
        for item in &syntax.items {
            if let Item::Struct(item) = item {
                let legacy_vec_scope = debug_or_docs(&item.attrs);
                let string_scope = string_field_derives_or_docs(&item.attrs);
                for field in &item.fields {
                    if docs_only(&field.attrs)
                        && ((legacy_vec_scope && public_vec_ident(field))
                            || (string_scope && public_string_field(field)))
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
        else_delimiters: BTreeSet::new(),
        wildcard_prefixes: BTreeSet::new(),
    };
    calls.visit_file(&syntax);
    let lines: Vec<_> = source.split('\n').collect();
    calls.else_delimiters.retain(|line| {
        lines
            .get(line - 1)
            .is_some_and(|text| text.trim() == "} else {")
    });
    calls.wildcard_prefixes.retain(|line| {
        lines
            .get(line - 1)
            .is_some_and(|text| text.trim() == "_ => {")
    });
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
        calls.else_delimiters.clear();
        calls.wildcard_prefixes.clear();
    }
    if audit.opaque || audit.item_macro {
        fields.clear();
    }
    if audit.opaque {
        audit.closings.clear();
        audit.declarations.clear();
    }
    let declarations = if audit.opaque || audit.item_macro {
        BTreeSet::new()
    } else {
        plain_declarations(source, &syntax, &ancestor_syntax, module)
    };
    let mut items = context_lines(source, ancestor, module);
    items.extend(declarations.iter().copied());
    items.extend(calls.wildcard_prefixes);
    // Only the exact exclusive, syntax-proved declaration lines may remove
    // literal/derive opacity. No expression or enclosing item is unmasked.
    audit
        .opaque_lines
        .retain(|line| !declarations.contains(line));
    Ok(Evidence {
        fields,
        closings: calls.closings,
        else_delimiters: calls.else_delimiters,
        opaque: audit.opaque_lines,
        declarations: audit.declarations,
        items,
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
    let exports: std::collections::BTreeMap<String, String> = request
        .get("exports")
        .map(|value| serde_json::from_value(value.clone()).unwrap())
        .unwrap_or_default();
    let aggregate_prefixes = aggregate_prefixes(source, ancestor, module, &exports);
    println!(
        "SDF_COVERAGE_SYNTAX_V1={}",
        serde_json::json!({"source": source, "ancestor": ancestor, "module": module, "helper_source": include_str!("coverage_syntax.rs"), "fields": evidence.fields, "closings": evidence.closings, "else_delimiters": evidence.else_delimiters, "opaque": evidence.opaque, "declarations": evidence.declarations, "items": evidence.items, "exports": exports, "aggregate_prefixes": aggregate_prefixes})
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

fn compiles(source: &str) {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("probe.rs");
    std::fs::write(&path, source).unwrap();
    let output = std::process::Command::new("rustc")
        .args([
            "--edition=2024",
            "--crate-type=lib",
            "--emit=metadata",
            "-o",
        ])
        .arg(tmp.path().join("probe.rmeta"))
        .arg(path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn compiler_accepted_positive_and_negative_declarations() {
    let positive = "use std::{\n    borrow::Cow,\n    collections::BTreeSet,\n    sync::{Arc, OnceLock},\n};\nstatic NAMES: OnceLock<BTreeSet<&str>> = OnceLock::new();\n";
    compiles(positive);
    assert_eq!(candidate_lines(positive), (1..=6).collect());
    let probes = [
        "static FN: fn() = || { panic!(\"uncovered\") };\n",
        "static FN: fn() = { fn hidden() { panic!(\"uncovered\") } hidden };\n",
        "macro_rules! make { () => { || panic!(\"uncovered\") } }\nstatic FN: fn() = make!();\n",
        "#[cfg(any())]\nstatic VALUE: usize = 0;\n",
        "#[allow(unused_imports)]\nuse std::{\n  borrow::Cow,\n  sync::Arc,\n};\n",
        "use std::sync::OnceLock; static VALUE: OnceLock<u8> = OnceLock::new();\n",
        "static VALUE: u32 = 1 + 2;\n",
        "struct T<const N: usize>; const fn f() -> T<1> { T }\nstatic X: T<{ #[unsafe(no_mangle)] pub extern \"C\" fn hidden2() { panic!(\"uncovered\") } 1 }> = f();\n",
        "const fn f() -> [u8;1] { [0] }\nstatic X: [u8; { #[unsafe(no_mangle)] pub extern \"C\" fn hidden() { panic!(\"uncovered\") } 1 }] = f();\n",
        "macro_rules! ty { () => { u32 } }\nconst fn f() -> u32 { 0 }\nstatic X: ty!() = f();\n",
    ];
    for source in probes {
        compiles(source);
        assert!(candidate_lines(source).is_empty(), "{source}");
    }
}
#[test]
fn ignores_shared_lines_and_syntax_expressions() {
    for source in [
        "use std::sync::Arc; fn hidden() {}\n",
        "static X: u8 = f::<u8>();\n",
        "static X: u8 = f(|| 1);\n",
        "#[custom] use std::{sync::Arc};\n",
        "static X: u8 = { f() };\n",
    ] {
        assert!(candidate_lines(source).is_empty(), "{source}");
    }
}
#[test]
fn compiler_accepts_custom_attribute_but_proof_refuses_it() {
    let tmp = tempfile::tempdir().unwrap();
    let macro_src = tmp.path().join("attr.rs");
    let macro_lib = tmp.path().join("libattr.so");
    std::fs::write(&macro_src, "extern crate proc_macro; use proc_macro::TokenStream; #[proc_macro_attribute] pub fn custom(_: TokenStream, item: TokenStream) -> TokenStream { item }").unwrap();
    let result = std::process::Command::new("rustc")
        .args(["--edition=2024", "--crate-type=proc-macro", "-o"])
        .arg(&macro_lib)
        .arg(&macro_src)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let source = "use attr::custom;\n#[custom]\nstatic VALUE: std::sync::OnceLock<u8> = std::sync::OnceLock::new();\n";
    let path = tmp.path().join("probe.rs");
    std::fs::write(&path, source).unwrap();
    let result = std::process::Command::new("rustc")
        .args([
            "--edition=2024",
            "--crate-type=lib",
            "--emit=metadata",
            "--extern",
        ])
        .arg(format!("attr={}", macro_lib.display()))
        .arg("-o")
        .arg(tmp.path().join("probe.rmeta"))
        .arg(path)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let lines = candidate_lines(source);
    assert!(!lines.contains(&2) && !lines.contains(&3));
}

#[test]
fn source_and_ancestor_context_fail_closed() {
    let text = "use std::{\n sync::OnceLock,\n};\nstatic X: OnceLock<u8> = OnceLock::new();\n";
    assert_eq!(
        context_lines(text, "//! docs\nmod parse;", "parse"),
        (1..=4).collect()
    );
    for parent in [
        "#[cfg(any())] mod parse;",
        "#[custom] mod parse;",
        "mod parse {}",
        "mod parse; mod parse;",
        "macro_rules! m { () => {} } mod parse;",
        "#![allow(dead_code)] mod parse;",
    ] {
        assert!(context_lines(text, parent, "parse").is_empty(), "{parent}");
    }
    assert!(
        context_lines(
            "#![allow(dead_code)]\nuse std::sync::Arc;",
            "mod parse;",
            "parse"
        )
        .is_empty()
    );
    assert!(
        context_lines(
            "mod nested {\nuse std::sync::Arc;\n}",
            "mod parse;",
            "parse"
        )
        .is_empty()
    );
    assert!(context_lines("#[cfg(any())] fn f() {\nstatic X: std::sync::OnceLock<u8> = std::sync::OnceLock::new();\n}", "mod parse;", "parse").is_empty());
    assert!(context_lines("fn f() { if true {\nstatic X: std::sync::OnceLock<u8> = std::sync::OnceLock::new();\n} }", "mod parse;", "parse").is_empty());
    let unicode =
        "use std::{\r\n sync::OnceLock,\r\n};\r\nstatic X: OnceLock<u8> = OnceLock::new(); // π\n";
    assert_eq!(
        context_lines(unicode, "mod parse;", "parse"),
        (1..=3).collect()
    );
}

#[test]
fn string_field_shapes_are_bounded_without_widening_legacy_vec_proof() {
    let source = "#[derive(Debug, Default, Clone, PartialEq)]\npub struct Metadata {\n pub raw: String,\n pub subtype: Option<String>,\n}\n";
    assert_eq!(
        classify(source, "mod types;", "types").unwrap().fields,
        BTreeSet::from([3, 4])
    );
    // The existing Vec predicate and its old multi-derive refusal stay intact.
    assert!(
        classify(
            "#[derive(Debug, Clone)] pub struct World { pub extensions: Vec<Thing>, }",
            "mod types;",
            "types"
        )
        .unwrap()
        .fields
        .is_empty()
    );
    for ty in [
        "Option<u8>",
        "Option<String,>",
        "Option::<String>",
        "Option<Vec<String>>",
        "Option<String, String>",
        "String<Thing>",
        "&'static str",
        "[String; 1]",
        "ty!()",
        "::String",
        "std::string::String",
    ] {
        let source = format!("pub struct Metadata {{\n pub raw: {ty},\n}}\n");
        assert!(
            classify(&source, "mod types;", "types")
                .unwrap()
                .fields
                .is_empty(),
            "{ty}"
        );
    }
    for changed in [
        source.replace("pub raw", "#[cfg(any())] pub raw"),
        source.replace(
            "#[derive(Debug, Default, Clone, PartialEq)]",
            "#[custom::transform]",
        ),
    ] {
        assert!(
            !classify(&changed, "mod types;", "types")
                .unwrap()
                .fields
                .contains(&3)
        );
    }
    for ancestor in [
        "#[custom::transform] mod types;",
        "mod types; #[macro_use] extern crate custom;",
        "mod types; macro_rules! later { () => {} }",
    ] {
        assert!(
            classify(source, ancestor, "types")
                .unwrap()
                .fields
                .is_empty()
        );
    }
    assert!(
        classify(
            &format!("use custom::Debug;\n{source}"),
            "mod types;",
            "types"
        )
        .unwrap()
        .fields
        .is_empty()
    );
}

#[test]
fn else_delimiter_requires_exclusive_real_tokens_and_untransformed_context() {
    let source = "fn f(x: bool) -> u8 {\n if x {\n  1\n } else {\n  2\n }\n}\n";
    assert_eq!(
        classify(source, "mod parse;", "parse")
            .unwrap()
            .else_delimiters,
        BTreeSet::from([4])
    );
    assert_eq!(
        classify(&source.replace('\n', "\r\n"), "mod parse;", "parse")
            .unwrap()
            .else_delimiters,
        BTreeSet::from([4])
    );
    for changed in [
        source.replace(" } else {", " } else { 7;"),
        source.replace(" } else {", " } else if !x { 7 } else {"),
        source.replace("fn f", "#[cfg(any())] fn f"),
        source.replace("fn f", "#[custom::transform] fn f"),
        format!("macro_rules! make {{ () => {{ {source} }} }}"),
        "fn f() { let raw = r#\"\n} else {\n\"#; }".to_owned(),
    ] {
        assert!(
            classify(&changed, "mod parse;", "parse")
                .unwrap()
                .else_delimiters
                .is_empty(),
            "{changed}"
        );
    }
    for ancestor in [
        "#[cfg(any())] mod parse;",
        "#[custom::transform] mod parse;",
        "mod parse; #[custom::transform] fn later() {}",
    ] {
        assert!(
            classify(source, ancestor, "parse")
                .unwrap()
                .else_delimiters
                .is_empty()
        );
    }
}

#[test]
fn compiler_accepted_field_bodies_and_macros_receive_no_string_field_evidence() {
    for source in [
        "pub struct Metadata { pub raw: [u8; { #[unsafe(no_mangle)] pub extern \"C\" fn hidden() { std::hint::black_box(42); } 1 }], }",
        "macro_rules! ty { () => { String } } pub struct Metadata { pub raw: ty!(), }",
        "pub struct Metadata { pub raw: Option<Vec<String>>, }",
        "pub struct Metadata { #[cfg(any())] pub raw: String, }",
    ] {
        compiles(source);
        assert!(
            classify(source, "mod types;", "types")
                .unwrap()
                .fields
                .is_empty(),
            "{source}"
        );
    }
}

#[test]
fn plain_concrete_declarations_and_headers_are_exclusive_and_context_bound() {
    let source = "#[derive(Debug, Clone, PartialEq)]\npub struct Scene {\n pub ambient: [f32; 4],\n pub grid: bool,\n /// origin\n pub source: Option<Source>,\n pub other: Other,\n}\nimpl Default for Scene {\n fn default() -> Self { todo!() }\n}\n";
    let evidence = classify(source, "mod types;", "types").unwrap();
    assert_eq!(evidence.items, BTreeSet::from([1, 2, 3, 4, 6, 7, 9]));
    assert!(evidence.items.is_disjoint(&evidence.opaque));
    for replacement in [
        "#[cfg(any())]",
        "#[custom::transform]",
        "#[derive(custom::Debug)]",
    ] {
        let source = source.replace("#[derive(Debug, Clone, PartialEq)]", replacement);
        assert!(
            classify(&source, "mod types;", "types")
                .unwrap()
                .items
                .is_empty()
        );
    }
    assert!(
        classify(source, "#[custom::transform] mod types;", "types")
            .unwrap()
            .items
            .is_empty()
    );
    assert_eq!(
        classify(
            &format!("use shadow::Debug;\n{source}"),
            "mod types;",
            "types"
        )
        .unwrap()
        .items,
        BTreeSet::from([1])
    );
    let shared = "pub struct S { pub value: bool, }\nimpl S { fn side_effect() { panic!() } }\n";
    assert!(
        classify(shared, "mod types;", "types")
            .unwrap()
            .items
            .is_empty()
    );
    for ty in [
        "[u8; { #[unsafe(no_mangle)] pub extern \"C\" fn hidden() { panic!() } 4 }]",
        "Wrapper<{ #[unsafe(no_mangle)] pub extern \"C\" fn hidden() { panic!() } 4 }>",
        "type_macro!()",
        "[u8; size()]",
        "[u8; const { 4 }]",
    ] {
        let source = format!("pub struct S {{\n pub field: {ty},\n}}\n");
        assert!(
            classify(&source, "mod types;", "types")
                .unwrap()
                .items
                .is_empty(),
            "{ty}"
        );
    }
}

#[test]
fn aggregate_delimiters_prove_only_tokens_and_never_the_constructor_prefix() {
    let source = "fn f() {\n Ok(Scene {\n  value: if ready() {\n   run()\n  } else {\n   fallback()\n  },\n })\n}\n";
    let evidence = classify(source, "mod parse;", "parse").unwrap();
    assert!(evidence.closings.contains(&7));
    assert!(evidence.closings.contains(&8));
    assert!(!evidence.items.contains(&2));
    assert!(!evidence.closings.contains(&2));
    for source in [
        format!("#[custom::transform]\n{source}"),
        format!("#[cfg(any())]\n{source}"),
        format!("macro_rules! transform {{ () => {{ {source} }} }}"),
    ] {
        assert!(
            classify(&source, "mod parse;", "parse")
                .unwrap()
                .closings
                .is_empty()
        );
    }
}

#[test]
fn compiler_accepts_plain_fields_and_hidden_type_body_control_but_only_plain_is_proved() {
    use std::{fs, process::Command};
    let temp = tempfile::tempdir().unwrap();
    let plain = "#[derive(Debug, Clone, PartialEq)]\npub struct Scene {\n pub ambient: [f32; 4],\n pub grid: bool,\n pub source: Option<String>,\n}\nimpl Default for Scene {\n fn default() -> Self { Self { ambient: [0.; 4], grid: true, source: None } }\n}\n";
    let hidden = "pub struct Hidden {\n pub field: [u8; { #[unsafe(no_mangle)] pub extern \"C\" fn hidden_type_body() { std::hint::black_box(42); } 4 }],\n}\n";
    for (name, source) in [("plain", plain), ("hidden", hidden)] {
        let path = temp.path().join(format!("{name}.rs"));
        fs::write(&path, source).unwrap();
        let result = Command::new("rustc")
            .args(["--crate-type", "lib", "--edition", "2024"])
            .arg(&path)
            .arg("-o")
            .arg(temp.path().join(format!("{name}.rlib")))
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    let evidence = classify(plain, "mod types;", "types").unwrap();
    assert!(BTreeSet::from([1, 2, 3, 4, 5, 7]).is_subset(&evidence.items));
    assert!(
        classify(hidden, "mod types;", "types")
            .unwrap()
            .items
            .is_empty()
    );
}

fn constructor_fixture() -> (String, String, std::collections::BTreeMap<String, String>) {
    (
        "use crate::*;\npub fn value() -> Result<Scene, ()> {\n Ok(Scene {\n  value: 1,\n })\n}\n"
            .into(),
        "mod types; mod camera; mod parse; pub use types::*; pub use camera::*;".into(),
        std::collections::BTreeMap::from([
            (
                "types".into(),
                "#[derive(Debug, Clone, PartialEq)]\npub struct Scene {\n pub value: u8,\n}\n"
                    .into(),
            ),
            ("camera".into(), "pub struct Camera;".into()),
        ]),
    )
}
#[test]
fn intrinsic_constructor_requires_exact_closed_exports_and_plain_scene() {
    let (source, root, exports) = constructor_fixture();
    assert_eq!(
        aggregate_prefixes(&source, &root, "parse", &exports),
        BTreeSet::from([3])
    );
    assert!(aggregate_prefixes(&source, &root, "other", &exports).is_empty());
    assert!(
        aggregate_prefixes(&source, &root, "parse", &std::collections::BTreeMap::new()).is_empty()
    );
    for root in [
        root.replace("pub use camera::*;", ""),
        format!("{root} pub use unknown::*;"),
        root.replace("mod types;", "#[path=\"elsewhere.rs\"] mod types;"),
        root.replace("pub use types::*;", "pub use ::types::*;"),
    ] {
        assert!(aggregate_prefixes(&source, &root, "parse", &exports).is_empty());
    }
    for ty in [
        "pub type Scene = Other;",
        "pub enum Scene { Value { value: u8 } }",
        "pub struct Scene<const N: usize> { pub value: u8 }",
        "pub struct Scene { pub value: [u8; { fn hidden() {} 1 }] }",
        "#[custom::transform] pub struct Scene { pub value: u8 }",
        "pub use unknown::Scene;",
        "pub struct Scene { pub value: type_macro!() }",
    ] {
        let mut changed = exports.clone();
        changed.insert("types".into(), ty.into());
        assert!(
            aggregate_prefixes(&source, &root, "parse", &changed).is_empty(),
            "{ty}"
        );
    }
    let mut changed = exports.clone();
    changed.insert("camera".into(), "pub fn Ok() {}".into());
    assert!(aggregate_prefixes(&source, &root, "parse", &changed).is_empty());
}
#[test]
fn constructor_refuses_callable_and_raw_identifier_shadowing_and_opaque_context() {
    let (source, root, exports) = constructor_fixture();
    for source in [
        source.replace("use crate::*;", "use crate::*; use other::Ok;"),
        source.replace("use crate::*;", "use crate::*; use other::value as r#Ok;"),
        source.replace("use crate::*;", "use crate::*; use other::*;"),
        source.replace(" Ok(Scene {", " let Ok = |v| v;\n Ok(Scene {"),
        source.replace(" Ok(Scene {", " let r#Ok = |v| v;\n Ok(Scene {"),
        source.replace("value()", "value(Ok: fn(Scene) -> Result<Scene, ()>)"),
        source.replace("value()", "value<Scene>()"),
        source.replace(" Ok(Scene {", " struct Scene { value: u8 }\n Ok(Scene {"),
        format!("{source}\nfn Ok(v: Scene) -> Result<Scene, ()> {{ std::result::Result::Ok(v) }}"),
        source.replace("pub fn", "#[cfg(any())] pub fn"),
        source.replace("pub fn", "#[custom::transform] pub fn"),
        source.replace(
            " Ok(Scene {",
            " format!(\"{{}}\", { let Ok = |x| x; 1 });\n Ok(Scene {",
        ),
        source.replace(" Ok(Scene {", " format!(\"{{}}\", nested!());\n Ok(Scene {"),
        source.replace(
            " Ok(Scene {",
            " format!(\"{{}}\", { #[custom::transform] fn injected() {} 1 });\n Ok(Scene {",
        ),
        source.replace(" Ok(Scene {", " Ok::<Scene, ()>(Scene {"),
        source.replace(" Ok(Scene {", " runtime(Scene {"),
        source.replace("  value: 1,", "  value: 1, ..base"),
        source.replace(" Ok(Scene {\n  value: 1,", " Ok(Scene { value: runtime(),"),
    ] {
        assert!(
            aggregate_prefixes(&source, &root, "parse", &exports).is_empty(),
            "{source}"
        );
    }
}
#[test]
fn compiler_confirms_intrinsic_and_callable_constructor_distinction_and_glob_macro_ambiguity() {
    use std::{fs, process::Command};
    let temp = tempfile::tempdir().unwrap();
    let (source, root, exports) = constructor_fixture();
    for name in ["types", "camera"] {
        fs::write(temp.path().join(format!("{name}.rs")), &exports[name]).unwrap();
    }
    let root = format!(
        "{root}\npub static HITS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);\npub fn run() {{ let _ = parse::value(); }}\n"
    );
    fs::write(temp.path().join("lib.rs"), &root).unwrap();
    for (name, source, expected) in [
        ("intrinsic", source.clone(), 0),
        ("callable", format!("{source}\nfn Ok(v: Scene) -> Result<Scene, ()> {{ crate::HITS.fetch_add(1, std::sync::atomic::Ordering::Relaxed); std::result::Result::Ok(v) }}"), 1),
        ("raw_callable", source.replace(" Ok(Scene {", " fn r#Ok(v: Scene) -> Result<Scene, ()> { crate::HITS.fetch_add(1, std::sync::atomic::Ordering::Relaxed); std::result::Result::Ok(v) }\n Ok(Scene {"), 1),
    ] {
        fs::write(temp.path().join("parse.rs"), &source).unwrap();
        let lib = temp.path().join("libcase.rlib");
        let result = Command::new("rustc").args(["--crate-name", "case", "--crate-type", "lib", "--edition=2024"])
            .arg(temp.path().join("lib.rs")).arg("-o").arg(&lib).output().unwrap();
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        fs::write(temp.path().join("run.rs"), format!("fn main() {{ case::run(); assert_eq!(case::HITS.load(std::sync::atomic::Ordering::Relaxed), {expected}); }}")).unwrap();
        let runner = temp.path().join("run");
        assert!(Command::new("rustc").args(["--edition=2024", "--extern"]).arg(format!("case={}", lib.display()))
            .arg(temp.path().join("run.rs")).arg("-o").arg(&runner).status().unwrap().success());
        assert!(Command::new(&runner).status().unwrap().success());
        assert_eq!(aggregate_prefixes(&source, &root, "parse", &exports).contains(&3), expected == 0, "{name}");
    }
    // Implicit sibling-module macro exports cannot silently replace a known
    // prelude macro through crate::*. The already-required compilation fails.
    fs::write(temp.path().join("lib.rs"), format!("{root} mod hidden;")).unwrap();
    fs::write(temp.path().join("hidden.rs"), "#[macro_export] macro_rules! format { ($($x:tt)*) => { fn Ok(v:Scene)->Result<Scene,()> { std::result::Result::Ok(v) } }; }").unwrap();
    fs::write(
        temp.path().join("parse.rs"),
        source.replace(" Ok(Scene {", " format!(\"unused\");\n Ok(Scene {"),
    )
    .unwrap();
    let output = Command::new("rustc")
        .args([
            "--crate-type",
            "lib",
            "--edition=2024",
            "--error-format=json",
        ])
        .arg(temp.path().join("lib.rs"))
        .arg("-o")
        .arg(temp.path().join("bad.rlib"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    let codes: Vec<_> = String::from_utf8(output.stderr)
        .unwrap()
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter_map(|value| value["code"]["code"].as_str().map(str::to_owned))
        .collect();
    assert!(codes.contains(&"E0659".to_owned()));
}

#[test]
fn compiled_self_import_bindings_and_const_bodies_never_prove_constructor_prefixes() {
    use std::{fs, process::Command};
    let temp = tempfile::tempdir().unwrap();
    let (source, root, exports) = constructor_fixture();
    for name in ["types", "camera"] {
        fs::write(temp.path().join(format!("{name}.rs")), &exports[name]).unwrap();
    }
    fs::write(
        temp.path().join("hidden.rs"),
        "pub struct Scene { pub value: u8 }",
    )
    .unwrap();
    let compile = |source: &str, root: &str| {
        fs::write(temp.path().join("parse.rs"), source).unwrap();
        fs::write(temp.path().join("lib.rs"), root).unwrap();
        let result = Command::new("rustc")
            .args(["--crate-type=lib", "--edition=2024"])
            .arg(temp.path().join("lib.rs"))
            .arg("-o")
            .arg(temp.path().join("case.rlib"))
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(
            aggregate_prefixes(source, root, "parse", &exports).is_empty(),
            "{source}\n{root}"
        );
    };
    for import in [
        "pub use hidden::Scene::{self};",
        "pub use hidden::{Scene::{self}};",
        "pub use hidden::r#Scene::{self};",
    ] {
        compile(&source, &format!("{root} mod hidden; {import}"));
    }
    for body in [
        " const X: Result<Scene, ()> =\n Ok(Scene {\n value:1,\n });\n X\n",
        " static X: Result<Scene, ()> =\n Ok(Scene {\n value:1,\n });\n std::result::Result::Ok(Scene {value:1})\n",
        " struct Holder;\n impl Holder { const X: Result<Scene, ()> =\n Ok(Scene {\n value:1,\n });\n }\n Holder::X\n",
        " trait Holder { const X: Result<Scene, ()> =\n Ok(Scene {\n value:1,\n });\n }\n struct Value; impl Holder for Value {}\n Value::X\n",
        " fn scalar<const N:u8>() -> u8 { N }\n let value = scalar::<{\n let result: Result<Scene, ()> =\n Ok(Scene {\n value:1,\n });\n match result { Ok(v) => v.value, Err(()) => 0 }\n }>();\n std::result::Result::Ok(Scene {value})\n",
    ] {
        compile(
            &format!("use crate::*;\npub fn value() -> Result<Scene, ()> {{\n{body}}}\n"),
            &root,
        );
    }
}

#[test]
fn plain_unit_enums_prove_only_exclusive_declarations() {
    let source =
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum Surface {\n Earth,\n Moon,\n}\n";
    compiles(source);
    let proof = classify(source, "mod types;", "types").unwrap();
    assert_eq!(proof.items, BTreeSet::from([1, 2, 3, 4]));
    assert!(proof.items.is_disjoint(&proof.opaque));
    for source in [
        source.replace(
            "Earth,",
            "Earth = { const fn code() -> isize { 1 } code() },",
        ),
        source
            .replace(
                "Earth,",
                "Earth([u8; { const fn size() -> usize { 1 } size() }]),",
            )
            .replace("#[derive(Debug, Clone, Copy, PartialEq, Eq)]", ""),
        source.replace("Earth,", "#[cfg(any())] Earth,"),
        format!("macro_rules! hidden {{ () => {{}} }}\n{source}"),
        source.replace(
            "#[derive(Debug, Clone, Copy, PartialEq, Eq)]",
            "#[derive(custom::Debug)]",
        ),
    ] {
        assert!(
            classify(&source, "mod types;", "types")
                .unwrap()
                .items
                .is_empty(),
            "{source}"
        );
    }
    for declaration in [
        "pub enum Surface { Earth = { const fn value() -> isize { 1 } value() }, Moon, }",
        "pub enum Surface { Earth([u8; { const fn size() -> usize { 1 } size() }]), Moon, }",
    ] {
        compiles(declaration);
        assert!(
            classify(declaration, "mod types;", "types")
                .unwrap()
                .items
                .is_empty()
        );
    }
    let shared = source
        .replace("Earth,", "Earth, Moon,")
        .replace(" Moon,\n}", "}");
    compiles(&shared);
    assert!(
        !classify(&shared, "mod types;", "types")
            .unwrap()
            .items
            .contains(&3)
    );
    assert!(
        classify(source, "#[custom::attribute] mod types;", "types")
            .unwrap()
            .items
            .is_empty()
    );
    let shadowed = format!("use hidden::Debug;\n{source}");
    assert!(
        !classify(&shadowed, "mod types;", "types")
            .unwrap()
            .items
            .contains(&2)
    );
}

#[test]
fn wildcard_block_prefix_is_exclusive_and_refuses_guards_attributes_and_macros() {
    let source =
        "pub fn value(x: u8) -> u8 {\n match x {\n _ => {\n  std::hint::black_box(1)\n }\n }\n}\n";
    compiles(source);
    assert_eq!(
        classify(source, "mod parse;", "parse").unwrap().items,
        BTreeSet::from([3])
    );
    for source in [
        source.replace("_ => {", "_ if std::hint::black_box(true) => {"),
        source.replace("_ => {", "0 => {"),
        source.replace("_ => {", "_ => { std::hint::black_box(2);"),
        source.replace("_ => {", "#[custom::attribute] _ => {"),
        source.replace("pub fn", "#[custom::attribute] pub fn"),
        format!("macro_rules! hidden {{ () => {{}} }}\n{source}"),
    ] {
        assert!(
            classify(&source, "mod parse;", "parse")
                .unwrap()
                .items
                .is_empty(),
            "{source}"
        );
    }
    for ancestor in [
        "#[custom::attribute] mod parse;",
        "macro_rules! hidden { () => {} } mod parse;",
        "mod parse {}",
    ] {
        assert!(
            classify(source, ancestor, "parse")
                .unwrap()
                .items
                .is_empty()
        );
    }
}

#[test]
fn spherical_constructor_uses_the_same_closed_boundary_as_scene() {
    let (source, root, exports) = constructor_fixture();
    let source = source.replace("Scene", "SphericalCoordinates");
    let exports = exports
        .into_iter()
        .map(|(name, value)| (name, value.replace("Scene", "SphericalCoordinates")))
        .collect();
    assert_eq!(
        aggregate_prefixes(&source, &root, "parse", &exports),
        BTreeSet::from([3])
    );
    for source in [
        source.replace(
            " Ok(SphericalCoordinates {",
            " let Ok = |v| v;\n Ok(SphericalCoordinates {",
        ),
        source.replace(
            " Ok(SphericalCoordinates {",
            " struct SphericalCoordinates { value: u8 }\n Ok(SphericalCoordinates {",
        ),
        source.replace(
            "use crate::*;",
            "use crate::*; use hidden::SphericalCoordinates::{self};",
        ),
        source.replace(
            "use crate::*;",
            "use crate::*; use hidden::r#SphericalCoordinates::{self};",
        ),
        source.replace(
            " Ok(SphericalCoordinates {",
            " format!(\"{}\", { struct SphericalCoordinates; 1 });\n Ok(SphericalCoordinates {",
        ),
        source.replace(
            " Ok(SphericalCoordinates {",
            " custom!();\n Ok(SphericalCoordinates {",
        ),
        source.replace(
            " Ok(SphericalCoordinates {",
            " Ok::<SphericalCoordinates, ()>(SphericalCoordinates {",
        ),
        source.replace(
            " Ok(SphericalCoordinates {",
            " runtime(SphericalCoordinates {",
        ),
    ] {
        assert!(
            aggregate_prefixes(&source, &root, "parse", &exports).is_empty(),
            "{source}"
        );
    }
}

#[test]
fn gui_constructor_requires_the_same_closed_declaration_and_callable_boundary() {
    let (source, root, exports) = constructor_fixture();
    let source = source.replace("Scene", "Gui");
    let exports = exports
        .into_iter()
        .map(|(name, value)| (name, value.replace("Scene", "Gui")))
        .collect();
    assert_eq!(
        aggregate_prefixes(&source, &root, "parse", &exports),
        BTreeSet::from([3])
    );
    for source in [
        source.replace(" Ok(Gui {", " let Ok = |v| v;\n Ok(Gui {"),
        source.replace(
            " Ok(Gui {",
            " fn r#Ok(v: Gui) -> Result<Gui, ()> { std::result::Result::Ok(v) }\n Ok(Gui {",
        ),
        source.replace(" Ok(Gui {", " struct Gui { value: u8 }\n Ok(Gui {"),
        source.replace("use crate::*;", "use crate::*; use hidden::Gui::{self};"),
        source.replace("use crate::*;", "use crate::*; use hidden::r#Gui::{self};"),
        source.replace(
            " Ok(Gui {",
            " format!(\"{}\", { struct Gui; 1 });\n Ok(Gui {",
        ),
        source.replace(" Ok(Gui {", " custom!();\n Ok(Gui {"),
        source.replace(" Ok(Gui {", " runtime(Gui {"),
        source.replace(" Ok(Gui {\n  value: 1,", " Ok(Gui { value: runtime(),"),
        source.replace("pub fn", "#[custom::attribute] pub fn"),
    ] {
        assert!(
            aggregate_prefixes(&source, &root, "parse", &exports).is_empty(),
            "{source}"
        );
    }
    for definition in [
        "pub type Gui = Other;",
        "pub struct Gui<const N: usize> { pub value: u8 }",
        "pub struct Gui { pub value: [u8; { fn hidden() {} 1 }] }",
    ] {
        let mut changed = exports.clone();
        changed.insert("types".into(), definition.into());
        assert!(aggregate_prefixes(&source, &root, "parse", &changed).is_empty());
    }
}
