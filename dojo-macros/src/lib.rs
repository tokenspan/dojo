use proc_macro::TokenStream;

use crate::expand_embedded_derive::expand_embedded_derive;
use crate::expand_enum_derive::expand_enum_derive;
use crate::expand_model_derive::expand_model_derive;
use crate::expand_update_model_derive::expand_update_model_derive;

mod common;
mod expand_embedded_derive;
mod expand_enum_derive;
mod expand_model_derive;
mod expand_update_model_derive;

#[proc_macro_derive(Model, attributes(dojo))]
pub fn model_derive_macro(input: TokenStream) -> TokenStream {
    expand_model_derive(input.into()).unwrap().into()
}

#[proc_macro_derive(Type, attributes(dojo))]
pub fn enum_derive_macro(input: TokenStream) -> TokenStream {
    expand_enum_derive(input.into()).unwrap().into()
}

#[proc_macro_derive(EmbeddedModel)]
pub fn embedded_derive_macro(input: TokenStream) -> TokenStream {
    expand_embedded_derive(input.into()).unwrap().into()
}

#[proc_macro_derive(UpdateModel, attributes(dojo))]
pub fn update_model_derive_macro(input: TokenStream) -> TokenStream {
    expand_update_model_derive(input.into()).unwrap().into()
}
