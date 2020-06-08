//! Functionality for parsing and visiting `assert_pre` attributes.

use proc_macro_error::{emit_error, emit_warning};
use std::mem;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    token::Paren,
    visit_mut::VisitMut,
    ExprCall,
};

use crate::{
    precondition::{Precondition, PreconditionList},
    render_assert_pre,
};

/// An `assert_pre` declaration.
pub(crate) struct AssertPreAttr {
    /// The parentheses surrounding the attribute.
    _parentheses: Paren,
    /// The precondition list in the declaration.
    preconditions: PreconditionList<Precondition>,
}

impl Parse for AssertPreAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(AssertPreAttr {
            _parentheses: parenthesized!(content in input),
            preconditions: content.parse()?,
        })
    }
}

/// The reason to display in the hint where to add the reason.
const HINT_REASON: &'static str = "why does this hold?";

/// The name of the macro used to assert that a condition holds.
const ASSERT_CONDITION_HOLDS_ATTR: &'static str = "assert_pre";

/// A visitor for `assert_pre` declarations.
pub(crate) struct AssertPreVisitor;

impl VisitMut for AssertPreVisitor {
    fn visit_expr_call_mut(&mut self, call: &mut ExprCall) {
        let mut i = 0;
        let mut attr_found = false;
        while i < call.attrs.len() {
            if call.attrs[i].path.is_ident(ASSERT_CONDITION_HOLDS_ATTR) {
                let attr = call.attrs.remove(i);

                if attr_found {
                    emit_error!(
                        attr,
                        "duplicate {} attribute found",
                        ASSERT_CONDITION_HOLDS_ATTR;
                        hint = "combine the list of conditions into one attribute"
                    );
                    continue;
                } else {
                    attr_found = true;
                }

                if let Ok(attr) = syn::parse2(attr.tokens.clone()).map_err(|err| emit_error!(err)) {
                    process_attribute(attr, call);
                }
            } else {
                i += 1;
            }
        }

        syn::visit_mut::visit_expr_call_mut(self, call);
    }
}

/// Checks if a warning about an unfinished reason should be given.
fn has_unfinished_reason(precondition: &Precondition) -> bool {
    let reason = precondition.reason().map(|r| r.value());

    if let Some(mut reason) = reason {
        reason.make_ascii_lowercase();
        match &*reason {
            HINT_REASON | "todo" | "?" => true,
            _ => false,
        }
    } else {
        false
    }
}

/// Process a found `assert_pre` attribute.
fn process_attribute(attr: AssertPreAttr, call: &mut ExprCall) {
    for precondition in attr.preconditions.iter() {
        if precondition.reason().is_none() {
            let missing_reason_span = precondition
                .missing_reason_span()
                .expect("the reason is missing");
            emit_error!(
                precondition.span(),
                "you need to specify a reason why this precondition holds";
                help = missing_reason_span => "add `, reason = {:?}`", HINT_REASON
            );
        } else if has_unfinished_reason(precondition) {
            emit_warning!(
                precondition
                .reason()
                .map(|r| r.span())
                .expect("reason exists"),
                "you should specify a more meaningful reason here";
                help = "specifying a meaningful reason here will help you and others understand why this is ok in the future"
            )
        }
    }

    let mut output = render_assert_pre(attr.preconditions, call.clone());
    mem::swap(&mut output, call);
}
