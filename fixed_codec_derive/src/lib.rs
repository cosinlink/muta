mod decode;
mod encode;
mod fixed_codec;

extern crate proc_macro;

use crate::fixed_codec::impl_fixed_codec;

#[proc_macro_derive(FixedCodec)]
pub fn fixed_codec(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let ast = syn::parse2(input).unwrap();
    proc_macro::TokenStream::from(impl_fixed_codec(ast))
}
