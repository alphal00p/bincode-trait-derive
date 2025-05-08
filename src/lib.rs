use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Fields, GenericParam, Ident, Lifetime, LifetimeParam, Path, Type, TypeParam,
    TypePath, WherePredicate, parse_macro_input, spanned::Spanned,
};

#[proc_macro_derive(Encode)]
pub fn encode_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    let mut generics_for_impl = input.generics.clone();
    let mut where_clause_for_impl = generics_for_impl.make_where_clause().clone();

    // Add bounds for generic type parameters
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

    // Extract field types to add additional bounds for associated types
    let field_types = match &input.data {
        Data::Struct(data_struct) => data_struct.fields.iter().map(|f| &f.ty).collect::<Vec<_>>(),
        Data::Enum(data_enum) => data_enum
            .variants
            .iter()
            .flat_map(|variant| variant.fields.iter().map(|f| &f.ty))
            .collect::<Vec<_>>(),
        Data::Union(_) => vec![],
    };

    // Check for associated types in field types and add bounds for them
    for ty in field_types {
        if let Type::Path(type_path) = ty {
            if type_path.path.segments.len() > 1 {
                // This might be an associated type like F::Element
                let predicate: WherePredicate = syn::parse_quote! {
                    #ty: ::bincode::Encode
                };
                where_clause_for_impl.predicates.push(predicate);
            }
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

    let expanded = quote! {
        impl #impl_generics ::bincode::Encode for #struct_name #ty_generics #where_clause_for_impl {
            fn encode<__E: ::bincode::enc::Encoder>(&self, encoder: &mut __E) -> std::result::Result<(), ::bincode::error::EncodeError> {
                #encode_body
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Decode, attributes(trait_decode))]
pub fn trait_derive(input: TokenStream) -> TokenStream {
    let mut input_ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &input_ast.ident;

    let mut option_trait_name: Option<Path> = None;
    let mut option_context_type_name: Option<Path> = None;

    for attr in input_ast
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("trait_decode"))
    {
        if let Err(e) = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("trait") {
                if option_context_type_name.is_some() {
                    return Err(meta.error("cannot specify both `trait` and `context_type` in #[trait_decode]"));
                }
                option_trait_name = Some(meta.value()?.parse::<Path>()?);
                Ok(())
            } else if meta.path.is_ident("context_type") {
                if option_trait_name.is_some() {
                    return Err(meta.error("cannot specify both `trait` and `context_type` in #[trait_decode]"));
                }
                option_context_type_name = Some(meta.value()?.parse::<Path>()?);
                Ok(())
            } else {
                Err(meta.error("unrecognized key for #[trait_decode] attribute, supported keys are `trait` and `context_type`"))
            }
        }) {
            return e.to_compile_error().into();
        }
    }

    let mut generics_for_impl = input_ast.generics.clone();
    let mut where_clause_for_impl = input_ast.generics.make_where_clause().clone();

    // Only add a generic parameter if we don't have a concrete context type
    let context_generic_ident = Ident::new("__Context", proc_macro2::Span::call_site());

    // Only add the generic parameter if not using a concrete context type
    if option_context_type_name.is_none() {
        let context_generic_param_for_impl =
            GenericParam::Type(TypeParam::from(context_generic_ident.clone()));

        generics_for_impl
            .params
            .push(context_generic_param_for_impl);
    }

    if let Some(ref trait_ident_path) = option_trait_name {
        let pred: WherePredicate = syn::parse_quote! { #context_generic_ident: #trait_ident_path };
        where_clause_for_impl.predicates.push(pred);
    } else if let Some(ref concrete_type_path) = option_context_type_name {
        // Instead of using the context_type directly in the where clause, we'll use it in the impl
        // Replace the generic context parameter with the concrete type
        // Just don't add any where predicates for the context
    } else {
        // No attribute specifying trait or context_type. __Context remains generic for this impl.
        // It will be constrained by field requirements, e.g., `usize: Decode<__Context>` implies `__Context = ()`.
    }

    // Add `TypeParameter: Decode<__Context>` bounds for the struct's own type parameters.
    for param in input_ast.generics.params.iter() {
        if let GenericParam::Type(type_param) = param {
            let type_ident = &type_param.ident;
            let type_path = Type::Path(TypePath {
                qself: None,
                path: type_ident.clone().into(),
            });
            let predicate: WherePredicate = syn::parse_quote! {
                #type_path: ::bincode::Decode<#context_generic_ident>
            };
            where_clause_for_impl.predicates.push(predicate);
        }
    }

    // Create the context type based on whether it's generic or concrete
    let context_type = if let Some(ref concrete_type_path) = option_context_type_name {
        // If we're using a concrete type, use it directly
        quote! { #concrete_type_path }
    } else {
        // Otherwise use the generic parameter
        quote! { #context_generic_ident }
    };

    let (impl_generics, _, _) = generics_for_impl.split_for_impl(); // Contains original generics + __Context
    let (_, ty_generics_for_struct, _) = input_ast.generics.split_for_impl(); // Original generics for struct type

    let decode_body = match &input_ast.data {
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
        Data::Union(_) => unimplemented!("Unions are not supported by Decode derive"),
    };

    let expanded = quote! {
        impl #impl_generics ::bincode::Decode<#context_type> for #struct_name #ty_generics_for_struct #where_clause_for_impl {
            fn decode<D: ::bincode::de::Decoder<Context = #context_type>>(decoder: &mut D) -> std::result::Result<Self, ::bincode::error::DecodeError> {
                #decode_body
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(BorrowDecodeFromDecode, attributes(trait_decode))]
pub fn borrow_decode_from_trait_decode(input: TokenStream) -> TokenStream {
    let mut input_ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &input_ast.ident;

    let mut option_trait_name: Option<Path> = None;
    let mut option_context_type_name: Option<Path> = None;

    for attr in input_ast
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("trait_decode"))
    {
        if let Err(e) = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("trait") {
                if option_context_type_name.is_some() {
                    return Err(meta.error("cannot specify both `trait` and `context_type` in #[trait_decode]"));
                }
                option_trait_name = Some(meta.value()?.parse::<Path>()?);
                Ok(())
            } else if meta.path.is_ident("context_type") {
                if option_trait_name.is_some() {
                    return Err(meta.error("cannot specify both `trait` and `context_type` in #[trait_decode]"));
                }
                option_context_type_name = Some(meta.value()?.parse::<Path>()?);
                Ok(())
            } else {
                Err(meta.error("unrecognized key for #[trait_decode] attribute, supported keys are `trait` and `context_type`"))
            }
        }) {
            return e.to_compile_error().into();
        }
    }

    let mut generics_for_impl = input_ast.generics.clone();
    let mut where_clause_for_impl = input_ast.generics.make_where_clause().clone();

    let lifetime_de_ident = Lifetime::new("'_de", proc_macro2::Span::call_site());
    let lifetime_de_param = GenericParam::Lifetime(LifetimeParam::new(lifetime_de_ident.clone()));
    generics_for_impl.params.push(lifetime_de_param);

    // Only add a generic parameter if we don't have a concrete context type
    let context_ident = Ident::new("__Context", proc_macro2::Span::call_site());

    // Only add the generic parameter if not using a concrete context type
    if option_context_type_name.is_none() {
        let context_generic_param_for_impl =
            GenericParam::Type(TypeParam::from(context_ident.clone()));
        generics_for_impl
            .params
            .push(context_generic_param_for_impl);
    }

    if let Some(ref trait_ident_path) = option_trait_name {
        let pred: WherePredicate = syn::parse_quote! { #context_ident: #trait_ident_path };
        where_clause_for_impl.predicates.push(pred);
    } else if let Some(ref concrete_type_path) = option_context_type_name {
        // Instead of using the context_type directly in the where clause, we'll use it in the impl
        // Replace the generic context parameter with the concrete type
        // Just don't add any where predicates for the context
    } else {
        // __Context remains generic for this impl.
    }

    // Add `TypeParameter: Decode<__Context>` bounds for the struct's own type parameters,
    // as BorrowDecode calls Decode.
    for param in input_ast.generics.params.iter() {
        if let GenericParam::Type(type_param) = param {
            let type_ident = &type_param.ident;
            let type_path = Type::Path(TypePath {
                qself: None,
                path: type_ident.clone().into(),
            });
            let predicate: WherePredicate = syn::parse_quote! {
                #type_path: ::bincode::Decode<#context_ident>
            };
            where_clause_for_impl.predicates.push(predicate);
        }
    }

    // This struct itself must implement Decode<__Context> for BorrowDecode to call it.
    // This should be implicitly handled if the Decode derive is also present and correct.
    // If Decode is not derived, this might lead to issues, but that's outside this macro's scope.

    // Create the context type based on whether it's generic or concrete
    let context_type = if let Some(ref concrete_type_path) = option_context_type_name {
        // If we're using a concrete type, use it directly
        quote! { #concrete_type_path }
    } else {
        // Otherwise use the generic parameter
        quote! { #context_ident }
    };

    let (impl_generics, _, _) = generics_for_impl.split_for_impl(); // Contains original generics + '_de + __Context
    let (_, ty_generics_for_struct, _) = input_ast.generics.split_for_impl(); // Original generics for struct type

    let expanded = quote! {
        impl #impl_generics ::bincode::BorrowDecode<#lifetime_de_ident, #context_type> for #struct_name #ty_generics_for_struct #where_clause_for_impl {
            fn borrow_decode<D: ::bincode::de::BorrowDecoder<#lifetime_de_ident, Context = #context_type>>(decoder: &mut D) -> std::result::Result<Self, ::bincode::error::DecodeError> {
                <Self as ::bincode::Decode<#context_type>>::decode(decoder)
            }
        }
    };

    TokenStream::from(expanded)
}
