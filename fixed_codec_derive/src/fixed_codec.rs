use proc_macro2::TokenStream;
use quote::quote;

use crate::decode::{decode_field, decode_parse_quotes};
use crate::encode::encode_field;

pub fn impl_fixed_codec(ast: syn::DeriveInput) -> TokenStream {
    let name = ast.ident;
    let body = if let syn::Data::Struct(s) = &ast.data {
        s
    } else {
        panic!("#[derive(FixedCodec)] is only defined for structs.");
    };

    let impl_encode = impl_encode(&name, body);
    let impl_decode = impl_decode(&name, body);

    quote! {
        const _: () = {
            extern crate rlp;
            extern crate protocol;

            use protocol::{fixed_codec::{FixedCodec, FixedCodecError}, ProtocolResult, Bytes};

            #impl_encode
            #impl_decode

            impl FixedCodec for #name {
                fn encode_fixed(&self) -> ProtocolResult<bytes::Bytes> {
                    Ok(bytes::Bytes::from(rlp::encode(&self)))
                }

                fn decode_fixed(bytes: bytes::Bytes) -> ProtocolResult<Self> {
                    Ok(rlp::decode(bytes.as_ref()).map_err(FixedCodecError::from)?)
                }
            }
        };
    }
}

fn impl_encode(name: &syn::Ident, body: &syn::DataStruct) -> TokenStream {
    let stmts = body
        .fields
        .iter()
        .enumerate()
        .map(|(i, field)| encode_field(i, field))
        .collect::<Vec<_>>();
    let stmts_len = stmts.len();
    let stmts_len = quote! { #stmts_len };

    quote! {
        impl rlp::Encodable for #name {
            fn rlp_append(&self, stream: &mut rlp::RlpStream) {
                stream.begin_list(#stmts_len);
                #(#stmts)*
            }
        }
    }
}

pub fn impl_decode(name: &syn::Ident, body: &syn::DataStruct) -> TokenStream {
    let stmts = body
        .fields
        .iter()
        .enumerate()
        .map(|(i, field)| decode_field(i, field, decode_parse_quotes()))
        .collect::<Vec<_>>();

    quote! {
        impl rlp::Decodable for #name {
            fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
                let result = #name {
                    #(#stmts)*
                };
                Ok(result)
            }
        }
    }
}
