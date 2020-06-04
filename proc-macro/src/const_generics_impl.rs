//! Implements the procedural macros using a zero-sized const generics parameter.
//!
//! # Advantages of this approach
//! - helpful error messages for typos
//! - supports arbitrarily complex strings out of the box
//! - quick to compute
//!
//! # Disadvantages of this approach
//! - error messages for no invariants not very readable

use proc_macro2::{Span, TokenStream};
use proc_macro_crate::crate_name;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse_quote, ExprCall, Ident, ItemFn, LitStr};

use crate::precondition::{Precondition, PreconditionHolds, PreconditionKind, PreconditionList};

/// Returns the name of the main crate.
fn get_crate_name() -> Ident {
    let name = crate_name("pre").expect("crate `pre` must be imported");
    Ident::new(&name, Span::call_site())
}

impl<T: ToTokens + Ord> ToTokens for PreconditionList<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for precondition in self.sorted_iter() {
            tokens.append_all(quote! {
                #precondition,
            });
        }
    }
}

impl ToTokens for Precondition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let pre = get_crate_name();
        match self.kind() {
            PreconditionKind::Custom(string) => {
                tokens.append_all(quote! {
                    ::#pre::CustomConditionHolds::<#string>
                });
            }
            PreconditionKind::ValidPtr { ident, .. } => {
                let ident_lit = LitStr::new(&ident.to_string(), ident.span());
                tokens.append_all(quote! {
                    ::#pre::ValidPtrConditionHolds::<#ident_lit>
                });
            }
        }
    }
}

/// Generates the code for the function with the precondition handling added.
pub(crate) fn render_pre(
    preconditions: PreconditionList<Precondition>,
    mut function: ItemFn,
) -> TokenStream {
    function.sig.inputs.push(parse_quote! {
        _: ::core::marker::PhantomData<(#preconditions)>
    });

    quote! {
        #function
    }
}

impl ToTokens for PreconditionHolds {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let pre = get_crate_name();
        match self.kind() {
            PreconditionKind::Custom(string) => {
                tokens.append_all(quote! {
                    ::#pre::CustomConditionHolds::<#string>
                });
            }
            PreconditionKind::ValidPtr { ident, .. } => {
                let ident_lit = LitStr::new(&ident.to_string(), ident.span());
                tokens.append_all(quote! {
                    ::#pre::ValidPtrConditionHolds::<#ident_lit>
                });
            }
        }
    }
}

/// Generates the code for the call with the precondition handling added.
pub(crate) fn render_assert_precondition(
    preconditions: PreconditionList<PreconditionHolds>,
    mut call: ExprCall,
) -> TokenStream {
    call.args.push(parse_quote! {
        ::core::marker::PhantomData::<(#preconditions)>
    });

    quote! {
        #call
    }
}
