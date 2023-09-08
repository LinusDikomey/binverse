//! binverse_derive provides macros for the `binverse` crate. 
//! `#\[serializable\]` automatically implements Serialize and Deserialize
//! and can parse some attributes related to versioning and data structure sizes.

#![warn(missing_docs)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Expr, Fields, FieldsNamed, FieldsUnnamed, GenericArgument, Item, ItemStruct, punctuated::{Pair, Punctuated}};

#[derive(Clone, Copy)]
enum SizeBytes {
    One,
    Two,
    Four,
    Eight
}
impl TryFrom<u32> for SizeBytes {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        use SizeBytes::*;
        Ok(match value {
            1 => One,
            2 => Two,
            4 => Four,
            8 => Eight,
            _ => return Err(())
        })
    }
}
impl SizeBytes {
    fn to_ident(&self) -> proc_macro2::Ident {
        use SizeBytes::*;
        proc_macro2::Ident::new(match self {
            One => "One",
            Two => "Two",
            Four => "Four",
            Eight => "Eight"
        }, proc_macro2::Span::call_site())
    }
}

#[proc_macro_attribute]
/// Implements `Serialize` and `Deserialize` for a struct.
/// All members also have to implement Serialize and Deserialize.
/// Members can be annotated with the following attributes:
/// - SizeBytes<N, T> to set the size bytes of a data structure. N can be 1, 2, 4 or 8.
/// - Added<N, T> states that the member was added in revision N.
/// - Removed<N, T> states that the member was removed in revision N. 
/// This also removes the member from the struct, it will only be used in the
///  Deserialize implementation to skip it in old data.
/// 
/// The attributes can be chained in any meaningful order. 
/// SizeBytes always has to be the innermost attribute if present.
/// 
/// # Example
/// ```ignore
/// #[binverse_derive::serializable]
/// struct Example {
///     a: i32,                                 // Always present 
///     b: Added<2, f32>,                       // Added in revision 2.
///     c: Removed<4, Added<2, u16>>,           // Also added in revision 2 and removed in revision 4 again. The macro will remove this field.
///     d: Added<6, SizeBytes<2, Vec<i32>>>,    // Added in revision 6. The Vec can't serialize with more than 65536 elements.
///     e: SizeBytes<1, String> ,               // A string with a maximum length of 255 bytes when serialized.
/// }
/// ```
pub fn serializable(attr: TokenStream, input: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        panic!("No attributes expected")
    }
    let ast = syn::parse_macro_input!(input);

    impl_serializable(ast)
}

enum AttributedField {
    Added(Box<AttributedField>, u32),
    Removed(Box<AttributedField>, u32),
    Normal(syn::Type, Option<SizeBytes>)
}
impl AttributedField {
    fn inner_ty(&self) -> &syn::Type {
        match self {
            AttributedField::Added(inner, _) | AttributedField::Removed(inner, _) => inner.inner_ty(),
            AttributedField::Normal(normal, _) => normal
        }
    }
    fn size_bytes(&self) -> Option<SizeBytes> {
        match self {
            AttributedField::Added(inner, _) | AttributedField::Removed(inner, _) => inner.size_bytes(),
            AttributedField::Normal(_, sb) => *sb
        }
    }

    fn is_removed(&self) -> bool {
        match self {
            AttributedField::Removed(_, _) => true,
            _ => false
        }
    }

    fn deserialize_patterns(&self) -> Vec<proc_macro2::TokenStream> {
        enum AddPatResult {
            Added(u32),
            Removed(u32),
            Normal
        }
        
        let mut patterns: Vec<proc_macro2::TokenStream> = Vec::new();

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
                AttributedField::Normal(_, _) => AddPatResult::Normal
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
            _ => return AttributedField::Normal(ty, None)
        };
        let mut segments = path_ty.path.segments.clone().into_iter();
        let first = segments.next().expect("Internal Error: path segment expected");
        let second = segments.next();
        let modifier = match (&first, &second) {
            (x, None) if x.ident == "Added" => TypeModifier::Added,
            (x, None) if x.ident == "Removed" => TypeModifier::Removed,
            (x, None) if x.ident == "SizeBytes" => {
                const SIZE_BYTES_ERR: &str = "Arguments expected for SizeBytes attribute: <{bytes}, {Type}>";
                if let syn::PathArguments::AngleBracketed(args) = &x.arguments {
                    let mut args = args.args.clone().into_iter();
                    let size_bytes = if let Some(GenericArgument::Const(Expr::Lit(syn::ExprLit {lit: syn::Lit::Int(int), .. }))) = args.next() {
                        assert!(int.suffix().is_empty(), "{}", SIZE_BYTES_ERR);
                        int.base10_parse::<u32>().expect(SIZE_BYTES_ERR)
                    } else {
                        panic!("{}", SIZE_BYTES_ERR);
                    };
                    
                    let normal_ty = match args.next() {
                        Some(GenericArgument::Type(inner_ty)) => match inner_ty.into() {
                            AttributedField::Normal(normal_ty, None) => normal_ty,
                            _ => panic!("SizeBytes attributes are not allowed to have more attributes inside")
                        },
                        _ => panic!("{}", SIZE_BYTES_ERR)
                    };
                    
                    if let Some(_) = args.next() {
                        panic!("{}", SIZE_BYTES_ERR);
                    }
                    let size_bytes = match size_bytes {
                        1 => SizeBytes::One,
                        2 => SizeBytes::Two,
                        4 => SizeBytes::Four,
                        8 => SizeBytes::Eight,
                        _ => panic!("Invalid SizeBytes: {}", size_bytes)
                    };
                    return AttributedField::Normal(normal_ty, Some(size_bytes));
                } else {
                    panic!("{}", SIZE_BYTES_ERR);
                }
            },
            _ => return AttributedField::Normal(ty, None)
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

            (inner_ty.into(), revision)
        } else {
            panic!("{}", ARG_ERROR)
        };

        match modifier {
            TypeModifier::Added => AttributedField::Added(Box::new(inner_ty), revision),
            TypeModifier::Removed => AttributedField::Removed(Box::new(inner_ty), revision)
        }
    }
}

fn impl_serializable(ast: Item) -> TokenStream {
    // returns the new fields, the serialize TokenStream and the deserialize TokenStream
    fn struct_impl(fields: Punctuated<syn::Field, syn::token::Comma>, named: bool) -> (Fields, proc_macro2::TokenStream, proc_macro2::TokenStream) {
        let attr_fields: Vec<(syn::Field, Option<syn::token::Comma>, AttributedField)> =
            fields.into_pairs()
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
        
        let serializers = attr_fields.iter()
            // only serialize the field if it wasn't marked as 'Removed'
            .filter(|(_, _, attr_field)| !attr_field.is_removed())
            // enumerate for tuple struct indices
            .enumerate()
            .map(|(i, (field, _, attr_field))| -> proc_macro2::TokenStream {
                let name = if named {
                    let ident = &field.ident;
                    quote! { #ident }
                } else {
                    let index = syn::Index::from(i);
                    quote! { #index }
                };
                match attr_field.size_bytes() {
                    Some(sb) => {
                        let sb_ident = sb.to_ident();
                        quote! { binverse::streams::Serializer::serialize_sized(s, binverse::serialize::SizeBytes::#sb_ident, &self.#name)?; }
                    },
                    None => quote! { binverse::serialize::Serialize::serialize(&self.#name, s)?; }.into()
                }
            });

        let mut next_deserialize_index: usize = 0;
        let mut next_deserialize_binding_name = |field: &syn::Field| {
            if named {
                let ident = &field.ident;
                quote! { #ident }
            } else {
                let field_name = syn::Ident::new(&format!("f{}", next_deserialize_index), proc_macro2::Span::call_site());
                next_deserialize_index += 1;
                quote! { #field_name }
            }
        };

        let deserializers = attr_fields.iter().map(|(field, _, attr_field)| {
            let ty = attr_field.inner_ty();
            let patterns = attr_field.deserialize_patterns();

            let deserialize_expr = match attr_field.size_bytes() {
                Some(sb) => {
                    let sb_ident = sb.to_ident();
                    quote! { binverse::streams::Deserializer::deserialize_sized(d, binverse::serialize::SizeBytes::#sb_ident)? }
                },
                _ => quote! { binverse::serialize::Deserialize::deserialize(d)? }
            };

            if patterns.is_empty() {
                assert!(!attr_field.is_removed(), "Internal error: No-pattern deserialize attribute was removed");
                let name = next_deserialize_binding_name(field);
                quote! { let #name = #deserialize_expr; }
            } else {
                let patterns = patterns.iter();
                if attr_field.is_removed() {
                    quote! {
                        match d.revision() {
                            #(#patterns)|* => { let _: #ty = #deserialize_expr; },
                            _ => ()
                        }
                    }
                } else {
                    let name = next_deserialize_binding_name(field);
                    quote! {
                        let #name = match d.revision() {
                            #(#patterns)|* => #deserialize_expr,
                            _ => std::default::Default::default()
                        };
                    }
                }
            }
        });
        let deserialize_inits = attr_fields.iter()
            // filter out fields that have been removed
            .filter(|(_, _, attr_field)| !attr_field.is_removed())
            .enumerate()
            .map(|(i, (field, _, _))| if named {
                let ident = &field.ident;
                quote! { #ident }
            } else {
                let ident = &syn::Ident::new(&format!("f{}", i), proc_macro2::Span::call_site());
                quote! { #ident }
            });

        let mut new_fields = Punctuated::new();
        attr_fields.iter()
            .filter(|(_, _, attr_field)| !attr_field.is_removed())
            .for_each(|(field, comma_opt, attr_field)| {
            new_fields.push_value(syn::Field {
                ty: attr_field.inner_ty().clone(),
                ..field.clone()
            });
            if let Some(comma) = comma_opt {
                new_fields.push_punct(*comma);
            }
        });
        
        (
            if named {
                Fields::Named(FieldsNamed {
                    named: new_fields,
                    brace_token: syn::token::Brace { span: proc_macro2::Span::call_site() }
                })
            } else {
                Fields::Unnamed(FieldsUnnamed {
                    unnamed: new_fields,
                    paren_token: syn::token::Paren { span: proc_macro2::Span::call_site() }
                })
            },
            quote! { #(#serializers)* },
            if named {
                quote! {
                    #(#deserializers)*
                    Ok(Self {
                        #(#deserialize_inits),*
                    })
                }
            } else {
                quote! {
                    #(#deserializers)*
                    Ok(Self(#(#deserialize_inits),*))
                }
            }
        )
    }


    match ast {
        Item::Struct(s) => {
            let (new_fields, serialize, deserialize) = match s.fields {
                Fields::Unit => (Fields::Unit, quote! { }, quote! { Ok(Self) }),
                Fields::Named(fields) => struct_impl(fields.named, true),
                Fields::Unnamed(fields) => struct_impl(fields.unnamed, false)
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
                impl ::binverse::serialize::Serialize for #ident {
                    #[inline]
                    fn serialize<W: ::std::io::Write>(&self, s: &mut ::binverse::streams::Serializer<W>) -> ::binverse::error::BinverseResult<()> {
                        #serialize
                        Ok(())
                    }
                }
                #[automatically_derived]
                #[allow(unused_qualifications)]
                impl ::binverse::serialize::Deserialize for #ident {
                    #[inline]
                    fn deserialize<R: ::std::io::Read>(d: &mut ::binverse::streams::Deserializer<R>) -> ::binverse::error::BinverseResult<Self> {
                        #deserialize
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
