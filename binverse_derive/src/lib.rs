use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Fields, Item};

#[proc_macro_attribute]
pub fn serializable(attr: TokenStream, input: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        panic!("No attributes expected")
    }
    let ast = syn::parse(input).unwrap();

    impl_serialize(ast)
}

fn impl_serialize(ast: Item) -> TokenStream {
    match &ast {
        Item::Struct(s) => {            

            let ident = &s.ident;

            let (serializers, deserializers) = match &s.fields {
                Fields::Unit => (quote! { }, quote! { Self }),
                Fields::Named(fields) => {
                    let serializers = fields.named.iter().map(|field| {
                        let fident = field.ident.as_ref().unwrap();
                        quote! { binverse::serialize::Serialize::serialize(&self.#fident, s)?; } 
                    });
                    let deserializers = fields.named.iter().map(|field| {
                        let fident = field.ident.as_ref().unwrap();
                        quote! { #fident: binverse::serialize::Deserialize::deserialize(d)? } 
                    });
                    (
                        quote! { #(#serializers)* },
                        quote! { 
                            Self {
                                #(#deserializers,)*
                            }
                        }
                    )
                },
                Fields::Unnamed(fields) => {
                    let serializers = fields.unnamed.iter().enumerate().map(|(i, _)| {
                        quote! { binverse::serialize::Serialize::serialize(&self.#i, s)?; }
                    });
                    let deserializers = fields.unnamed.iter().map(|_| {
                        quote! { binverse::serialize::Deserialize::deserialize(d)? } 
                    });
                    (
                        quote! { #(#serializers)* },
                        quote! { Self(#(#deserializers,)*) }
                    )
                }
            };

            quote! {
                // expanded by #[serialize]
                #ast

                impl binverse::serialize::Serialize for #ident {
                    fn serialize<W: std::io::Write>(&self, s: &mut binverse::streams::Serializer<W>) -> binverse::error::BinverseResult<()> {
                        #serializers
                        Ok(())
                    }
                }
                impl binverse::serialize::Deserialize for #ident {
                    fn deserialize<R: std::io::Read>(d: &mut binverse::streams::Deserializer<R>) -> binverse::error::BinverseResult<Self> {
                        Ok(#deserializers)
                    }
                }
            }.into()
        },
        Item::Enum(_e) => {
            todo!("Enums are not yet implemented")
        },
        _ => panic!("Only structs and enums are supported by serialize/deserialize attribute!")
    }
}