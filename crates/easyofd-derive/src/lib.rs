//! Proc-macro shim for `#[derive(OfdModel)]`.
//!
//! This is a thin wrapper that delegates to `easyofd-derive-impl`.
//! All logic lives in `easyofd-derive-impl` so it can be tested without
//! proc-macro infrastructure limitations.

use proc_macro::TokenStream;

/// Derive macro for the `OfdModel` trait. Delegates to `easyofd-derive-impl`.
///
/// See [`easyofd_derive_impl::derive_ofd_model_impl`] for documentation.
#[proc_macro_derive(OfdModel, attributes(ofd))]
pub fn derive_ofd_model(input: TokenStream) -> TokenStream {
    easyofd_derive_impl::derive_ofd_model_impl(input.into()).into()
}
