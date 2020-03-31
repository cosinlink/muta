use proc_macro2::TokenStream;
use quote::quote;

pub struct ParseQuotes {
    single:      TokenStream,
    list:        TokenStream,
    takes_index: bool,
}

pub fn decode_parse_quotes() -> ParseQuotes {
    ParseQuotes {
        single:      quote! { rlp.val_at },
        list:        quote! { rlp.list_at },
        takes_index: true,
    }
}

pub fn decode_field(index: usize, field: &syn::Field, quotes: ParseQuotes) -> TokenStream {
    let id = if let Some(ident) = &field.ident {
        quote! { #ident }
    } else {
        let index = syn::Index::from(index);
        quote! { #index }
    };

    let index = quote! { #index };
    let single = quotes.single;
    let list = quotes.list;

    match &field.ty {
        syn::Type::Array(_array) => {
            let len = quote! { #id.len() };
            let temp = quote! {
                let bytes = bytes::Bytes::from(#single(#index)?);
                if bytes.len() != #len {
                    panic!("Length mismatch");
                }
                let mut out = [0u8; #len];
                out.copy_from_slice(&bytes);
                out
            };
            quote! { #id: #temp, }
        }
        syn::Type::Path(path) => {
            let ident = &path
                .path
                .segments
                .first()
                .expect("there must be at least 1 segment")
                .ident;
            let ident_type = ident.to_string();
            if ident_type == "Vec" {
                if quotes.takes_index {
                    quote! { #id: #list(#index)?, }
                } else {
                    quote! { #id: #list()?, }
                }
            } else if ident_type == "Bytes" {
                if quotes.takes_index {
                    let temp = quote! { #single(#index)? };
                    quote! { #id: bytes::Bytes::from(#temp), }
                } else {
                    let temp = quote! { #single()? };
                    quote! { #id: bytes::Bytes::from(#temp), }
                }
            } else if quotes.takes_index {
                quote! { #id: #single(#index)?, }
            } else {
                quote! { #id: #single()?, }
            }
        }
        _ => panic!("fixed_codec_derive not supported"),
    }
}
