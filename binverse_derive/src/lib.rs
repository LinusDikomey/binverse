use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Expr, Fields, FieldsNamed, GenericArgument, Item, ItemStruct, PathArguments, punctuated::{Pair, Punctuated}};

#[proc_macro_attribute]
pub fn serializable(attr: TokenStream, input: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        panic!("No attributes expected")
    }
    let ast = syn::parse(input).unwrap();

    impl_serializable(ast)
}

enum AttributedField {
    Added(Box<AttributedField>, u32),
    Removed(Box<AttributedField>, u32),
    Normal(syn::Type)
}
impl From<syn::Type> for AttributedField {
    fn from(ty: syn::Type) -> Self {
        if let syn::Type::Path(path) = &ty {
            enum CompareValue {
                Added,
                Removed,
                Other
            }
            let mut iter = path.path.segments.iter();
            let first = iter.next().unwrap();
            let second = iter.next();
            let mut is_first = true;
            let detected_type = if first.ident == "binverse" && second.is_some() {
                is_first = false;
                match second.as_ref().unwrap() {
                    x if x.ident == "Added" => CompareValue::Added,
                    x if x.ident == "Removed" => CompareValue::Added,
                    _ => CompareValue::Other                
                }
            } else if second.is_none() {
                match () {
                    _ if first.ident == "Added" => CompareValue::Added,
                    _ if first.ident == "Removed" => CompareValue::Removed,
                    _ => CompareValue::Other
                }
            } else {
                CompareValue::Other
            };
            match detected_type {
                val@(CompareValue::Added | CompareValue::Removed) => {
                    let arguments = &if is_first { first } else { second.unwrap() }.arguments;
                    if let PathArguments::AngleBracketed(generics) = arguments {
                        let mut args_iter = generics.args.iter();
                        let inner_ty = if let GenericArgument::Type(inner_ty) = args_iter.next().unwrap() {
                            inner_ty
                        } else {
                            panic!("Invalid arguments on Added or Removed type")
                        };
                        let revision = if let GenericArgument::Const(Expr::Lit(syn::ExprLit {lit: syn::Lit::Int(int), .. })) = args_iter.next().unwrap() {
                            assert!(int.suffix().is_empty(), "Invalid const argument on Added or Removed type");
                            int.base10_parse::<u32>().expect("Invalid const argument on Added or Removed type")
                        } else {
                            panic!("Invalid arguments on Added or Removed type")
                        };
                        match val {
                            CompareValue::Added => AttributedField::Added(Box::new(inner_ty.clone().into()), revision),
                            CompareValue::Removed => AttributedField::Removed(Box::new(inner_ty.clone().into()), revision),
                            _ => unreachable!()
                        }
                    } else {
                        panic!("Invalid arguments on Added or Removed type")
                    }
                },
                _ => AttributedField::Normal(ty.clone())
            }
        } else {
            AttributedField::Normal(ty)
        }
    }
}

fn impl_serializable(ast: Item) -> TokenStream {
    println!("{:#?}", ast);
    match ast {
        Item::Struct(s) => {
            let (new_fields, serializers, deserializers) = match s.fields {
                Fields::Unit => (Fields::Unit, quote! { }, quote! { Self }),
                Fields::Named(fields) => {
                    let attr_fields: Vec<(syn::Field, Option<syn::token::Comma>, AttributedField)> = 
                        fields.named.into_pairs()
                        .map(|field| match field {
                            Pair::Punctuated(field, punct) => {
                                let ty = field.ty.clone();
                                (field, Some(punct), ty.into())
                            },
                            Pair::End(field) => {
                                let ty = field.ty.clone();
                                (field, None, ty.into())
                            }
                        })
                        .collect();
                    
                    let serializers = attr_fields.iter().map(|(field, _, attr_ty)| {
                        let ident = &field.ident;
                        // only serialize the field if it wasn't marked as 'Removed'
                        match attr_ty {
                            AttributedField::Removed(_, _) => quote! { },
                            _ => quote! { binverse::serialize::Serialize::serialize(&self.#ident, s)?; } 
                        } 
                    });
                    let deserializers = attr_fields.iter().map(|(field, _, attr_ty)| {
                        let ident = &field.ident;
                        match attr_ty {
                            AttributedField::Removed(_, _) => quote! { },
                        _ => quote! { #ident: binverse::serialize::Deserialize::deserialize(d)?, } 
                        }
                    });
                    let mut named = Punctuated::new();
                    attr_fields.iter().for_each(|(field, comma_opt, attr_ty)| {
                        match attr_ty {
                            AttributedField::Removed(_, _) => (),
                            _ => {
                                named.push_value(field.clone());
                                if let Some(comma) = comma_opt {
                                    named.push_punct(*comma);
                                }
                            }
                        }
                    });
                    let new_fields = FieldsNamed {
                        brace_token: fields.brace_token.clone(),
                        named
                    };
                    
                    (
                        Fields::Named(new_fields),
                        quote! { #(#serializers)* },
                        quote! { 
                            Self {
                                #(#deserializers)*
                            }
                        }
                    )
                },
                Fields::Unnamed(fields) => {
                    let serializers = fields.unnamed.iter().enumerate().map(|(i, _)| {
                        let field_index = syn::Index::from(i);
                        quote! { binverse::serialize::Serialize::serialize(&self.#field_index, s)?; }
                    });
                    let deserializers = fields.unnamed.iter().map(|_| {
                        quote! { binverse::serialize::Deserialize::deserialize(d)?, } 
                    });
                    let new_fields = fields.clone();
                    (
                        Fields::Unnamed(new_fields),
                        quote! { #(#serializers)* },
                        quote! { Self(#(#deserializers)*) }
                    )
                }
            };

            let new_struct = ItemStruct {
                fields: new_fields,
                ..s
            };
            
            let ident = new_struct.ident.clone();
            let new_ast = Item::Struct(new_struct);
            quote! {
                #new_ast

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