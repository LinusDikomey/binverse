#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Expr, Fields, FieldsNamed, GenericArgument, Item, ItemStruct, punctuated::{Pair, Punctuated}};

#[proc_macro_attribute]
pub fn serializable(attr: TokenStream, input: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        panic!("No attributes expected")
    }
    let ast = syn::parse(input).expect("Could not parse macro input");

    impl_serializable(ast)
}



enum AttributedField {
    Added(Box<AttributedField>, u32),
    Removed(Box<AttributedField>, u32),
    Normal(syn::Type)
}
impl AttributedField {
    fn inner_ty(&self) -> &syn::Type {
        match self {
            AttributedField::Added(inner, _) | AttributedField::Removed(inner, _) => inner.inner_ty(),
            AttributedField::Normal(normal) => normal
        }
    }

    fn is_removed(&self) -> bool {
        match self {
            AttributedField::Removed(_, _) => true,
            _ => false
        }
    }

    fn deserialize_patterns(&self) -> Vec<proc_macro2::TokenStream> {
        let mut patterns: Vec<proc_macro2::TokenStream> = Vec::new();
        enum AddPatResult {
            Added(u32),
            Removed(u32),
            Normal
        }
        fn add_pat(field: &AttributedField, patterns: &mut Vec<proc_macro2::TokenStream>) -> AddPatResult {
            match field {
                AttributedField::Added(inner, revision) => {
                    match add_pat(inner, patterns) {
                        AddPatResult::Added(_) => panic!("Invalid chained 'Added' attribute"),
                        AddPatResult::Removed(removed_revision) => {
                            if removed_revision >= *revision {
                                panic!("Error: inner revision has to be smaller than outer revision");
                            }
                        },
                        AddPatResult::Normal => ()
                    }
                    AddPatResult::Added(*revision)
                },
                AttributedField::Removed(inner, revision) => {
                    let start_revision = match add_pat(inner, patterns) {
                        AddPatResult::Added(start_revision) => start_revision,
                        AddPatResult::Removed(_) => panic!("Invalid chained 'Removed' attribute"),
                        AddPatResult::Normal => 0
                    };
                    let end_revision = revision - 1;
                    patterns.push(quote! { #start_revision..=#end_revision }.into());
                    AddPatResult::Removed(*revision)
                },
                AttributedField::Normal(_) => AddPatResult::Normal
            }
        }
        
        match add_pat(self, &mut patterns) {
            AddPatResult::Added(revision) => patterns.push(quote! { #revision.. }.into()),
            AddPatResult::Removed(_) | AddPatResult::Normal => ()
        }

        patterns
    }
}
impl From<syn::Type> for AttributedField {
    fn from(ty: syn::Type) -> Self {
        enum TypeModifier {
            Added,
            Removed
        }

        let path_ty = match &ty {
            syn::Type::Path(path_ty) => path_ty,
            _ => return AttributedField::Normal(ty)
        };
        let mut segments = path_ty.path.segments.clone().into_iter();
        let first = segments.next().expect("Internal Error: path segment expected");
        let second = segments.next();
        let modifier = match (&first, &second) {
            (x, None) if x.ident == "Added" => TypeModifier::Added,
            (x, None) if x.ident == "Removed" => TypeModifier::Removed,
            _ => return AttributedField::Normal(ty)
        };
        const ARG_ERROR: &str = "Arguments for attributed field expected: <{revision}, {Type}>";

        let (inner_ty, revision) = if let syn::PathArguments::AngleBracketed(args) = first.arguments {
            let mut args = args.args.into_iter();
            let revision = if let GenericArgument::Const(Expr::Lit(syn::ExprLit {lit: syn::Lit::Int(int), .. })) = args.next().expect("Not enough arguments on Added or Removed attribute, expected: <{revision}, {Type}>") {
                assert!(int.suffix().is_empty(), "Invalid const argument on Added or Removed attribute");
                int.base10_parse::<u32>().expect("Invalid const argument on Added or Removed attribute")
            } else {
                panic!("{}", ARG_ERROR)
            };

            let inner_ty = if let GenericArgument::Type(inner_ty) = args.next().expect("Missing argument on 'Added' or 'Removed' attribute") {
                inner_ty
            } else {
                panic!("{}", ARG_ERROR)
            };

            (inner_ty, revision)
        } else {
            panic!("{}", ARG_ERROR)
        };

        match modifier {
            TypeModifier::Added => AttributedField::Added(Box::new(inner_ty.into()), revision),
            TypeModifier::Removed => AttributedField::Removed(Box::new(inner_ty.into()), revision)
        }
    }
}

fn impl_serializable(ast: Item) -> TokenStream {
    match ast {
        Item::Struct(s) => {
            let (new_fields, serializers, deserializers) = match s.fields {
                Fields::Unit => (Fields::Unit, quote! { }, quote! { Ok(Self) }),
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
                            AttributedField::Removed(_, _) => Option::<proc_macro2::TokenStream>::None,
                            _ => Some(quote! { binverse::serialize::Serialize::serialize(&self.#ident, s)?; }.into())
                        }
                    });

                    let deserializers = attr_fields.iter().map(|(field, _, attr_ty)| {
                        let ident = field.ident.as_ref().expect("Internal error: named field on named struct expected");
                        // if the field was removed, the deserialized value can be ignored,
                        // but the type still has to be specified.
                        let optional_let_binding = if attr_ty.is_removed() {
                            let ty = attr_ty.inner_ty();
                            quote! { let _: #ty = }
                        } else {
                            quote! { let #ident = }
                        };
                        let patterns = attr_ty.deserialize_patterns();
                        if patterns.is_empty() {
                            quote! { #optional_let_binding binverse::serialize::Deserialize::deserialize(d)?; }
                        } else {
                            let patterns = patterns.iter();
                            quote! {
                                #optional_let_binding match d.revision() {
                                    #( #patterns )|* => binverse::serialize::Deserialize::deserialize(d)?,
                                    _ => std::default::Default::default()
                                };
                            }
                        }
                    });
                    let deserialize_inits = attr_fields.iter()
                        // filter out fields that have been removed
                        .filter(|(_, _, attr_field)| !attr_field.is_removed())
                        .map(|(field, _, _)| &field.ident); 
                    let mut named = Punctuated::new();
                    attr_fields.iter().for_each(|(field, comma_opt, attr_field)| {
                        match attr_field {
                            AttributedField::Removed(_, _) => (),
                            _ => {
                                named.push_value(syn::Field {
                                    ty: attr_field.inner_ty().clone(),
                                    ..field.clone()
                                });
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
                            #(#deserializers)*
                            Ok(Self {
                                #(#deserialize_inits),*
                            })
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
                        quote! { Ok(Self(#(#deserializers)*)) }
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

                #[automatically_derived]
                #[allow(unused_qualifications)]
                impl binverse::serialize::Serialize for #ident {
                    fn serialize<W: std::io::Write>(&self, s: &mut binverse::streams::Serializer<W>) -> binverse::error::BinverseResult<()> {
                        #serializers
                        Ok(())
                    }
                }
                #[automatically_derived]
                #[allow(unused_qualifications)]
                impl binverse::serialize::Deserialize for #ident {
                    fn deserialize<R: std::io::Read>(d: &mut binverse::streams::Deserializer<R>) -> binverse::error::BinverseResult<Self> {
                        #deserializers
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