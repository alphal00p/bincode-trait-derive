use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Fields, GenericParam, Ident, Lifetime, LifetimeParam, Lit, Type, TypeParam,
    TypePath, WherePredicate, parse_macro_input,
};

#[proc_macro_derive(TraitDecode, attributes(trait_decode))]
pub fn trait_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    let mut trait_name: Option<String> = None;
    for attr in input
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("trait_decode"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("trait") {
                let value = meta.value()?.parse::<Lit>()?;
                if let Lit::Str(lit_str) = value {
                    trait_name = Some(lit_str.value());
                    return Ok(());
                } else {
                    return Err(meta.error("expected a string literal"));
                }
            }
            Err(meta.error("unsupported attribute argument"))
        })
        .expect("Failed to parse attribute");
    }

    let trait_ident = syn::Ident::new(
        trait_name
            .as_ref()
            .expect("expected #[trait_decode(trait = \"TraitName\")] attribute"),
        proc_macro2::Span::call_site(),
    );

    let mut generics = input.generics.clone();
    let mut where_clause = generics.make_where_clause().clone();

    for param in generics.params.iter() {
        if let GenericParam::Type(type_param) = param {
            let type_ident = &type_param.ident;

            let type_path = Type::Path(TypePath {
                qself: None,
                path: type_ident.clone().into(),
            });

            let predicate: WherePredicate = syn::parse_quote! {
                #type_path: ::bincode::Decode<__Context>
            };

            where_clause.predicates.push(predicate);
        }
    }

    let extra_where = syn::parse_quote! {
        __Context: #trait_ident
    };
    where_clause.predicates.push(extra_where);

    let extra_generic = GenericParam::Type(TypeParam::from(Ident::new(
        "__Context",
        proc_macro2::Span::call_site(),
    )));
    generics.params.push(extra_generic);

    let (impl_generics, _, _) = generics.split_for_impl();
    let (_, ty_generics, _) = input.generics.split_for_impl();

    let decode_body = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => {
                let decode_fields = fields_named.named.iter().map(|f| {
                    let ident = &f.ident;
                    quote! { #ident: Decode::decode(decoder)? }
                });
                quote! { Ok(Self { #(#decode_fields),* }) }
            }
            Fields::Unnamed(fields_unnamed) => {
                let decode_fields = fields_unnamed.unnamed.iter().map(|_| {
                    quote! { Decode::decode(decoder)? }
                });
                quote! { Ok(Self(#(#decode_fields),*)) }
            }
            Fields::Unit => quote! { Ok(Self) },
        },
        Data::Enum(data_enum) => {
            // Enums: decode a discriminant to match variants
            let variants = data_enum.variants.iter().enumerate().map(|(idx, variant)| {
                let variant_ident = &variant.ident;
                match &variant.fields {
                    Fields::Named(fields_named) => {
                        let decode_fields = fields_named.named.iter().map(|f| {
                            let ident = &f.ident;
                            quote! { #ident: Decode::decode(decoder)? }
                        });
                        quote! {
                            #idx => Ok(Self::#variant_ident { #(#decode_fields),* }),
                        }
                    }
                    Fields::Unnamed(fields_unnamed) => {
                        let decode_fields = fields_unnamed.unnamed.iter().map(|_| {
                            quote! { Decode::decode(decoder)? }
                        });
                        quote! {
                            #idx => Ok(Self::#variant_ident(#(#decode_fields),*)),
                        }
                    }
                    Fields::Unit => quote! {
                        #idx => Ok(Self::#variant_ident),
                    },
                }
            });

            quote! {
                let discriminant: usize = Decode::decode(decoder)?;
                match discriminant {
                    #(#variants)*
                    other => Err(::bincode::error::DecodeError::OtherString(format!("unexpected enum variant"))),
                }
            }
        }
        Data::Union(_) => unimplemented!("Unions are not supported"),
    };

    // Generate final implementation
    let expanded = quote! {
        impl #impl_generics ::bincode::Decode<__Context> for #struct_name #ty_generics #where_clause

         {
            fn decode<D: ::bincode::de::Decoder<Context = __Context>>(decoder: &mut D) -> core::result::Result<Self, ::bincode::error::DecodeError> {
                #decode_body
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(BorrowDecodeFromTraitDecode, attributes(trait_decode))]
pub fn borrow_decode_from_trait_decode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    let mut trait_name: Option<String> = None;
    for attr in input
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("trait_decode"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("trait") {
                let value = meta.value()?.parse::<Lit>()?;
                if let Lit::Str(lit_str) = value {
                    trait_name = Some(lit_str.value());
                    return Ok(());
                } else {
                    return Err(meta.error("expected a string literal"));
                }
            }
            Err(meta.error("unsupported attribute argument"))
        })
        .expect("Failed to parse attribute");
    }

    let trait_ident = syn::Ident::new(
        trait_name
            .as_ref()
            .expect("expected #[trait_decode(trait = \"TraitName\")] attribute"),
        proc_macro2::Span::call_site(),
    );

    let mut generics = input.generics.clone();
    let mut where_clause = generics.make_where_clause().clone();

    for param in generics.params.iter() {
        if let GenericParam::Type(type_param) = param {
            let type_ident = &type_param.ident;

            let type_path = Type::Path(TypePath {
                qself: None,
                path: type_ident.clone().into(),
            });

            let predicate: WherePredicate = syn::parse_quote! {
                #type_path: ::bincode::Decode<__Context>
            };

            where_clause.predicates.push(predicate);
        }
    }

    let extra_where = syn::parse_quote! {
        __Context: #trait_ident
    };
    where_clause.predicates.push(extra_where);

    let extra_generic = GenericParam::Type(TypeParam::from(Ident::new(
        "__Context",
        proc_macro2::Span::call_site(),
    )));
    generics.params.push(extra_generic);

    let life_time = GenericParam::Lifetime(LifetimeParam::new(Lifetime::new(
        "'_de",
        proc_macro2::Span::call_site(),
    )));
    generics.params.push(life_time);

    let (impl_generics, _, _) = generics.split_for_impl();
    let (_, ty_generics, _) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics ::bincode::BorrowDecode<'_de, __Context> for #struct_name #ty_generics #where_clause

         {
            fn borrow_decode<D: ::bincode::de::BorrowDecoder<'_de, Context = __Context>>(decoder: &mut D) -> core::result::Result<Self, ::bincode::error::DecodeError> {
                ::bincode::Decode::decode(decoder)
            }
        }
    };

    TokenStream::from(expanded)
}
