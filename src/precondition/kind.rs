//! Defines the different kinds of preconditions.

use std::fmt;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    token::Paren,
    Ident, LitStr,
};

/// The custom keywords used by the precondition kinds.
mod custom_keywords {
    use syn::custom_keyword;

    custom_keyword!(valid_ptr);
}

/// The different kinds of preconditions.
#[derive(Clone)]
pub(crate) enum PreconditionKind {
    /// Requires that the given pointer is valid.
    ValidPtr {
        /// The `valid_ptr` keyword.
        _valid_ptr_keyword: custom_keywords::valid_ptr,
        /// The parentheses following the `valid_ptr` keyword.
        _parentheses: Paren,
        /// The identifier of the pointer.
        ident: Ident,
    },
    /// A custom precondition that is spelled out in a string.
    Custom(LitStr),
}

impl fmt::Debug for PreconditionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PreconditionKind::Custom(lit) => write!(f, "{:?}", lit.value()),
            PreconditionKind::ValidPtr {
                _valid_ptr_keyword: _,
                _parentheses: _,
                ident,
            } => write!(f, "valid_ptr({})", ident.to_string()),
        }
    }
}

impl Parse for PreconditionKind {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(custom_keywords::valid_ptr) {
            let content;

            Ok(PreconditionKind::ValidPtr {
                _valid_ptr_keyword: input.parse()?,
                _parentheses: parenthesized!(content in input),
                ident: content.parse()?,
            })
        } else if lookahead.peek(LitStr) {
            Ok(PreconditionKind::Custom(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}
