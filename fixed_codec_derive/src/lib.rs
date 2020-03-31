mod decode;
mod encode;
mod fixed_codec;

extern crate proc_macro;

use crate::fixed_codec::impl_fixed_codec;

#[proc_macro_derive(MutaFixedCodec)]
pub fn muta_fixed_codec(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let ast = syn::parse2(input).unwrap();
    let ret: proc_macro2::TokenStream = impl_fixed_codec(ast).into();
    proc_macro::TokenStream::from(ret)
}
