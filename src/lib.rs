use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Fields, GenericParam, Ident, Lifetime, LifetimeParam, Path, Type, TypeParam,
    TypePath, WherePredicate, parse_macro_input, spanned::Spanned,
};

#[proc_macro_derive(Encode)] // Renamed and no attributes()
pub fn encode_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    // No attribute parsing for Encode, as bincode::Encode has no context generic

    let mut generics_for_impl = input.generics.clone();
    let mut where_clause_for_impl = generics_for_impl.make_where_clause().clone();

    // Add ::bincode::Encode bounds for all type parameters
    for param in input.generics.params.iter() {
        if let GenericParam::Type(type_param) = param {
            let type_ident = &type_param.ident;
            let type_path = Type::Path(TypePath {
                qself: None,
                path: type_ident.clone().into(),
            });
            let predicate: WherePredicate = syn::parse_quote! {
                #type_path: ::bincode::Encode
            };
            where_clause_for_impl.predicates.push(predicate);
        }
    }

    let (impl_generics, _, _) = generics_for_impl.split_for_impl();
    let (_, ty_generics, _) = input.generics.split_for_impl();

    let encode_body = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => {
                let encode_fields = fields_named.named.iter().map(|f| {
                    let ident = &f.ident;
                    quote! { ::bincode::Encode::encode(&self.#ident, encoder)?; }
                });
                quote! { #(#encode_fields)* Ok(()) }
            }
            Fields::Unnamed(fields_unnamed) => {
                let encode_fields = fields_unnamed.unnamed.iter().enumerate().map(|(i, _)| {
                    let index = syn::Index::from(i);
                    quote! { ::bincode::Encode::encode(&self.#index, encoder)?; }
                });
                quote! { #(#encode_fields)* Ok(()) }
            }
            Fields::Unit => quote! { Ok(()) },
        },
        Data::Enum(data_enum) => {
            let variant_arms = data_enum.variants.iter().enumerate().map(|(idx, variant)| {
                let variant_ident = &variant.ident;
                let discriminant = proc_macro2::Literal::usize_unsuffixed(idx);
                match &variant.fields {
                    Fields::Named(fields_named) => {
                        let field_pats = fields_named.named.iter().map(|f| &f.ident);
                        let field_encodes = fields_named.named.iter().map(|f| {
                            let ident = &f.ident;
                            quote! { ::bincode::Encode::encode(#ident, encoder)?; }
                        });
                        quote! {
                            Self::#variant_ident { #(#field_pats),* } => {
                                ::bincode::Encode::encode(&#discriminant, encoder)?;
                                #(#field_encodes)*
                                Ok(())
                            }
                        }
                    }
                    Fields::Unnamed(fields_unnamed) => {
                        let field_pats_bindings = fields_unnamed
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, field)| Ident::new(&format!("field{}", i), field.span()));
                        let field_pats = field_pats_bindings.clone();
                        let field_encodes = field_pats_bindings.map(|binding| {
                            quote! { ::bincode::Encode::encode(#binding, encoder)?; }
                        });
                        quote! {
                            Self::#variant_ident ( #(#field_pats),* ) => {
                                ::bincode::Encode::encode(&#discriminant, encoder)?;
                                #(#field_encodes)*
                                Ok(())
                            }
                        }
                    }
                    Fields::Unit => {
                        quote! {
                            Self::#variant_ident => {
                                ::bincode::Encode::encode(&#discriminant, encoder)?;
                                Ok(())
                            }
                        }
                    }
                }
            });
            quote! {
                match self {
                    #(#variant_arms)*
                }
            }
        }
        Data::Union(_) => unimplemented!("Unions are not supported by Encode derive"),
    };

    // Note: ::bincode::Encode is not generic, and Encoder also does not take a Context generic here.
    let expanded = quote! {
        impl #impl_generics ::bincode::Encode for #struct_name #ty_generics #where_clause_for_impl {
            fn encode<E: ::bincode::enc::Encoder>(&self, encoder: &mut E) -> std::result::Result<(), ::bincode::error::EncodeError> {
                #encode_body
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Decode, attributes(trait_decode))]
pub fn trait_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    let mut option_trait_name: Option<Path> = None;
    for attr in input
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("trait_decode"))
    {
        if let Err(e) = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("trait") {
                option_trait_name = Some(meta.value()?.parse::<Path>()?);
                Ok(())
            } else {
                Err(meta.error("unrecognized key for #[trait_decode] attribute, only `trait = ...` is supported"))
            }
        }) {
            return e.to_compile_error().into();
        }
    }

    let mut generics_for_impl = input.generics.clone();
    let mut where_clause_for_impl = generics_for_impl.make_where_clause().clone();

    let context_generic = GenericParam::Type(TypeParam::from(Ident::new(
        "__Context",
        proc_macro2::Span::call_site(),
    )));
    generics_for_impl.params.push(context_generic);
    let (decode_trait_generic, decoder_context_generic, field_bound_context_generic) =
        if let Some(ref trait_ident_path) = option_trait_name {
            let context_where_predicate: WherePredicate = syn::parse_quote! {
                __Context: #trait_ident_path
            };
            where_clause_for_impl
                .predicates
                .push(context_where_predicate);

            (
                quote! { <__Context> },
                quote! { <Context = __Context> },
                quote! { <__Context> },
            )
        } else {
            (
                quote! { <__Context> },
                quote! { <Context = __Context> },
                quote! { <__Context> },
            )
        };

    for param in input.generics.params.iter() {
        if let GenericParam::Type(type_param) = param {
            let type_ident = &type_param.ident;
            let type_path = Type::Path(TypePath {
                qself: None,
                path: type_ident.clone().into(),
            });
            let predicate: WherePredicate = syn::parse_quote! {
                #type_path: ::bincode::Decode #field_bound_context_generic
            };
            where_clause_for_impl.predicates.push(predicate);
        }
    }

    let (impl_generics, _, _) = generics_for_impl.split_for_impl();
    let (_, ty_generics, _) = input.generics.split_for_impl();

    let decode_body = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => {
                let decode_fields = fields_named.named.iter().map(|f| {
                    let ident = &f.ident;
                    quote! { #ident: ::bincode::Decode::decode(decoder)? }
                });
                quote! { Ok(Self { #(#decode_fields),* }) }
            }
            Fields::Unnamed(fields_unnamed) => {
                let decode_fields = fields_unnamed.unnamed.iter().map(|_| {
                    quote! { ::bincode::Decode::decode(decoder)? }
                });
                quote! { Ok(Self(#(#decode_fields),*)) }
            }
            Fields::Unit => quote! { Ok(Self) },
        },
        Data::Enum(data_enum) => {
            let variants = data_enum.variants.iter().enumerate().map(|(idx, variant)| {
                let variant_ident = &variant.ident;
                match &variant.fields {
                    Fields::Named(fields_named) => {
                        let decode_fields = fields_named.named.iter().map(|f| {
                            let ident = &f.ident;
                            quote! { #ident: ::bincode::Decode::decode(decoder)? }
                        });
                        quote! { #idx => Ok(Self::#variant_ident { #(#decode_fields),* }), }
                    }
                    Fields::Unnamed(fields_unnamed) => {
                        let decode_fields = fields_unnamed.unnamed.iter().map(|_| {
                            quote! { ::bincode::Decode::decode(decoder)? }
                        });
                        quote! { #idx => Ok(Self::#variant_ident(#(#decode_fields),*)), }
                    }
                    Fields::Unit => quote! { #idx => Ok(Self::#variant_ident), },
                }
            });
            quote! {
                let discriminant: usize = ::bincode::Decode::decode(decoder)?;
                match discriminant {
                    #(#variants)*
                    _other => Err(::bincode::error::DecodeError::OtherString(format!("unexpected enum variant discriminant"))),
                }
            }
        }
        Data::Union(_) => unimplemented!("Unions are not supported by TraitDecode"),
    };

    let expanded = quote! {
        impl #impl_generics ::bincode::Decode #decode_trait_generic for #struct_name #ty_generics #where_clause_for_impl {
            fn decode<D: ::bincode::de::Decoder #decoder_context_generic>(decoder: &mut D) -> std::result::Result<Self, ::bincode::error::DecodeError> {
                #decode_body
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(BorrowDecodeFromDecode, attributes(trait_decode))]
pub fn borrow_decode_from_trait_decode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    let mut option_trait_name: Option<Path> = None;
    for attr in input
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("trait_decode"))
    // Still uses trait_decode attribute
    {
        if let Err(e) = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("trait") {
                option_trait_name = Some(meta.value()?.parse::<Path>()?);
                Ok(())
            } else {
                Err(meta.error("unrecognized key for #[trait_decode] attribute, only `trait = ...` is supported"))
            }
        }) {
            return e.to_compile_error().into();
        }
    }

    let mut generics_for_impl = input.generics.clone();
    let mut where_clause_for_impl = generics_for_impl.make_where_clause().clone();

    let lifetime_de = GenericParam::Lifetime(LifetimeParam::new(Lifetime::new(
        "'_de",
        proc_macro2::Span::call_site(),
    )));
    generics_for_impl.params.push(lifetime_de.clone()); // Push the lifetime param

    let (borrow_decode_trait_generic, borrow_decoder_context_generic, field_bound_context_generic) =
        if let Some(ref trait_ident_path) = option_trait_name {
            let context_generic = GenericParam::Type(TypeParam::from(Ident::new(
                "__Context",
                proc_macro2::Span::call_site(),
            )));
            generics_for_impl.params.push(context_generic);

            let context_where_predicate: WherePredicate = syn::parse_quote! {
                __Context: #trait_ident_path
            };
            where_clause_for_impl
                .predicates
                .push(context_where_predicate);

            (
                quote! { <'_de, __Context> },
                quote! { <'_de, Context = __Context> },
                quote! { <__Context> },
            )
        } else {
            // If no trait is specified, BorrowDecode still needs '_de, but no __Context
            (quote! { <'_de> }, quote! { <'_de> }, quote! {})
        };

    // Fields still need to implement Decode (potentially with __Context)
    // because BorrowDecode's default implementation calls Decode::decode.
    for param in input.generics.params.iter() {
        if let GenericParam::Type(type_param) = param {
            let type_ident = &type_param.ident;
            let type_path = Type::Path(TypePath {
                qself: None,
                path: type_ident.clone().into(),
            });
            let predicate: WherePredicate = syn::parse_quote! {
                #type_path: ::bincode::Decode #field_bound_context_generic
            };
            where_clause_for_impl.predicates.push(predicate);
        }
    }

    let (impl_generics, _, _) = generics_for_impl.split_for_impl();
    let (_, ty_generics, _) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics ::bincode::BorrowDecode #borrow_decode_trait_generic for #struct_name #ty_generics #where_clause_for_impl {
            fn borrow_decode<D: ::bincode::de::BorrowDecoder #borrow_decoder_context_generic>(decoder: &mut D) -> std::result::Result<Self, ::bincode::error::DecodeError> {
                ::bincode::Decode::decode(decoder)
            }
        }
    };

    TokenStream::from(expanded)
}
