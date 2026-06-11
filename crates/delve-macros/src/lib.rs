use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{ItemStruct, parse_macro_input};

#[proc_macro_attribute]
pub fn entity(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut s = parse_macro_input!(item as ItemStruct);

    // s.fields.members()

    s.to_token_stream().into()
}
