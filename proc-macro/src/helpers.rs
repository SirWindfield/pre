//! Allows retrieving the name of the main crate.

use lazy_static::lazy_static;
use proc_macro2::Span;
use proc_macro_error::{abort_call_site, emit_error};
use std::env;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Paren,
    Attribute, Expr, Signature,
};

/// The reason to display in examples on how to use reasons.
pub(crate) const HINT_REASON: &str = "<specify the reason why you can assure this here>";

lazy_static! {
    /// Returns the name of the main `pre` crate.
    pub(crate) static ref CRATE_NAME: String = {
        match proc_macro_crate::crate_name("pre") {
            Ok(name) => name,
            Err(err) => match env::var("CARGO_PKG_NAME") {
                // This allows for writing documentation tests on the functions themselves.
                //
                // This *may* lead to false positives, if someone also names their crate `pre`, however
                // it will very likely fail to compile at a later stage then.
                Ok(val) if val == "pre" => "pre".into(),
                _ => abort_call_site!("crate `pre` must be imported: {}", err),
            },
        }
    };
}

/// Checks if the given attribute is an `attr_to_check` attribute of the main crate.
pub(crate) fn is_attr(attr_to_check: &str, attr: &Attribute) -> bool {
    let path = &attr.path;

    if path.is_ident(attr_to_check) {
        true
    } else if path.segments.len() == 2 {
        // Note that `Path::leading_colon` is not checked here, so paths both with and without a
        // leading colon are accepted here
        path.segments[0].ident == *CRATE_NAME && path.segments[1].ident == attr_to_check
    } else {
        false
    }
}

/// Removes matching attributes, parses them, and then allows visiting them.
///
/// This returns the most appropriate span to reference the original attributes.
pub(crate) fn visit_matching_attrs_parsed<ParsedAttr: Parse>(
    attributes: &mut Vec<Attribute>,
    mut filter: impl FnMut(&mut Attribute) -> bool,
    mut visit: impl FnMut(ParsedAttr, Span),
) -> Option<Span> {
    let mut span_of_all: Option<Span> = None;
    let mut i = 0;

    // TODO: use `drain_filter` once it's stabilized (see
    // https://github.com/rust-lang/rust/issues/43244).
    while i < attributes.len() {
        if filter(&mut attributes[i]) {
            let attr = attributes.remove(i);
            // This should never fail on nightly, where joining is supported.
            // On stable, it'll use the better `bracket_token` span instead of the default `#` span
            // returned by `attr.span()`.
            let span = attr
                .span()
                .join(attr.bracket_token.span)
                .unwrap_or_else(|| attr.bracket_token.span);

            span_of_all = Some(match span_of_all.take() {
                Some(old_span) => old_span.join(span).unwrap_or_else(|| span),
                None => span,
            });

            match syn::parse2::<ParsedAttr>(attr.tokens) {
                Ok(parsed_attr) => visit(parsed_attr, span),
                Err(err) => emit_error!(err),
            }
        } else {
            i += 1;
        }
    }

    span_of_all
}

/// A parsable thing surrounded by parentheses.
pub(crate) struct Parenthesized<T> {
    /// The parentheses surrounding the object.
    _parentheses: Paren,
    /// The content that was surrounded by the parentheses.
    pub(crate) content: T,
}

impl<T: Parse> Parse for Parenthesized<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let parentheses = parenthesized!(content in input);
        let content = content.parse()?;

        Ok(Parenthesized {
            _parentheses: parentheses,
            content,
        })
    }
}

/// Returns the attributes of the given expression.
pub(crate) fn attributes_of_expression(expr: &mut Expr) -> Option<&mut Vec<Attribute>> {
    macro_rules! extract_attributes_from {
        ($expr:expr => $($variant:ident),*) => {
            match $expr {
                $(
                    Expr::$variant(e) => Some(&mut e.attrs),
                )*
                    _ => None,
            }
        }
    }

    extract_attributes_from!(expr =>
        Array, Assign, AssignOp, Async, Await, Binary, Block, Box, Break, Call, Cast,
        Closure, Continue, Field, ForLoop, Group, If, Index, Let, Lit, Loop, Macro, Match,
        MethodCall, Paren, Path, Range, Reference, Repeat, Return, Struct, Try, TryBlock, Tuple,
        Type, Unary, Unsafe, While, Yield
    )
}

/// Incorporates the given span into the signature.
///
/// Ideally both are shown, when the function definition is shown.
pub(crate) fn add_span_to_signature(span: Span, signature: &mut Signature) {
    signature.fn_token.span = signature.fn_token.span.join(span).unwrap_or_else(|| span);

    if let Some(token) = &mut signature.constness {
        token.span = token.span.join(span).unwrap_or_else(|| span);
    }

    if let Some(token) = &mut signature.asyncness {
        token.span = token.span.join(span).unwrap_or_else(|| span);
    }

    if let Some(token) = &mut signature.unsafety {
        token.span = token.span.join(span).unwrap_or_else(|| span);
    }

    if let Some(abi) = &mut signature.abi {
        abi.extern_token.span = abi.extern_token.span.join(span).unwrap_or_else(|| span);
    }
}
