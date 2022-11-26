use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(SerdeExt)]
pub fn serde_ext_derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl SerdeExt for #ident {}
    };
    output.into()
}

#[proc_macro_derive(WasmMsgExt)]
pub fn wasm_msg_ext_derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl common::cw::WasmMsgExt for #ident {}
    };
    output.into()
}
