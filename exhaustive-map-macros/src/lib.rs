use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{
    bracketed, parse::Parse, parse_macro_input, parse_quote, punctuated::Punctuated,
    spanned::Spanned, Data, DeriveInput, Field, Fields, GenericParam, Generics, Ident, Index,
    LitInt, Path, Token, Variant,
};

struct EnumInput {
    typ: Path,
    inhabitants: Punctuated<Ident, Token![,]>,
}

impl Parse for EnumInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let typ = input.parse()?;
        let _: Token![,] = input.parse()?;
        let content;
        bracketed!(content in input);
        let inhabitants = content.parse_terminated(Ident::parse, Token![,])?;
        Ok(Self { typ, inhabitants })
    }
}

#[proc_macro]
pub fn __impl_enum(input: TokenStream) -> TokenStream {
    let EnumInput { typ, inhabitants } = parse_macro_input!(input as EnumInput);
    let inhabitants = inhabitants.into_iter().map(|ident| {
        let mut typ = typ.clone();
        typ.segments.push(syn::PathSegment {
            ident,
            arguments: syn::PathArguments::None,
        });
        typ
    });

    let elements = inhabitants.len();
    let inhabitants1 = inhabitants.clone();
    let i1 = 0..elements;
    let inhabitants2 = inhabitants;
    let i2 = 0..elements;

    quote! {
        impl exhaustive_map::Finite for #typ {
            const INHABITANTS: usize = #elements;

            fn to_usize(&self) -> usize {
                match self {
                    #( #inhabitants1 => #i1 ),*
                }
            }

            fn from_usize(i: usize) -> Option<Self> {
                match i {
                    #( #i2 => Some(#inhabitants2) ),* ,
                    _ => None,
                }
            }
        }
    }
    .into()
}

#[proc_macro]
pub fn __impl_tuples(input: TokenStream) -> TokenStream {
    let v = parse_macro_input!(input as LitInt);
    let n: usize = v.base10_parse().unwrap();

    let mut res = Vec::<TokenStream>::new();
    for k in 1..=n {
        let indices = 0..k;
        let idents: Vec<_> = indices
            .clone()
            .map(|i| Ident::new(&format!("T{i}"), Span::call_site()))
            .collect();
        let idents2 = idents.clone();
        let idents3 = idents.clone();
        let idents4 = idents.clone();

        let rev_indices = indices.clone().map(Index::from).rev();
        let rev_idents = idents.iter().rev();

        res.push(
            quote! {
                impl <#( #idents: exhaustive_map::Finite ),*> exhaustive_map::Finite for (#( #idents2, )*) {
                    const INHABITANTS: usize = 1 #( * #idents3::INHABITANTS )*;

                    fn to_usize(&self) -> usize {
                        let mut res = 0;
                        #(
                            res *= #rev_idents::INHABITANTS;
                            res += self.#rev_indices.to_usize();
                        )*
                        res
                    }

                    fn from_usize(mut i: usize) -> Option<Self> {
                        if i >= Self::INHABITANTS {
                            return None;
                        }
                        Some((#(
                            {
                                let v = #idents4::from_usize(i % #idents4::INHABITANTS).unwrap();
                                i /= #idents4::INHABITANTS;
                                v
                            },
                        )*))
                    }
                }
            }
            .into(),
        );
    }
    res.into_iter().collect()
}

#[proc_macro_derive(Finite)]
pub fn finite_derive(input: TokenStream) -> TokenStream {
    finite_derive_inner(parse_macro_input!(input as DeriveInput)).into()
}

fn finite_derive_inner(input: DeriveInput) -> proc_macro2::TokenStream {
    let name = input.ident;

    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let FiniteImpl {
        inhabitants,
        to_usize,
        from_usize,
    } = finite_impl(&input.data);

    quote! {
        impl #impl_generics exhaustive_map::Finite for #name #ty_generics #where_clause {
            const INHABITANTS: usize = #inhabitants;

            fn to_usize(&self) -> usize {
                let v = self;
                #to_usize
            }

            #[allow(unused_assignments)]
            #[allow(clippy::let_unit_value)]
            #[allow(clippy::modulo_one)]
            fn from_usize(mut i: usize) -> Option<Self> {
                if i >= Self::INHABITANTS {
                    return None;
                }
                #from_usize
            }
        }
    }
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param
                .bounds
                .push(parse_quote!(::exhaustive_map::Finite));
        }
    }
    generics
}

struct FiniteImpl {
    inhabitants: proc_macro2::TokenStream,
    to_usize: proc_macro2::TokenStream,
    from_usize: proc_macro2::TokenStream,
}

struct FiniteImpls {
    inhabitants: Vec<proc_macro2::TokenStream>,
    to_usize: Vec<proc_macro2::TokenStream>,
    from_usize: Vec<proc_macro2::TokenStream>,
}

impl From<Vec<FiniteImpl>> for FiniteImpls {
    fn from(value: Vec<FiniteImpl>) -> Self {
        let mut inhabitants = vec![];
        let mut to_usize = vec![];
        let mut from_usize = vec![];
        for imp in value {
            inhabitants.push(imp.inhabitants);
            to_usize.push(imp.to_usize);
            from_usize.push(imp.from_usize);
        }
        FiniteImpls {
            inhabitants,
            to_usize,
            from_usize,
        }
    }
}

fn finite_impl(data: &Data) -> FiniteImpl {
    match *data {
        Data::Struct(ref data) => {
            let FiniteImpls {
                mut inhabitants,
                mut to_usize,
                from_usize,
            } = finite_impls_for_fields(data.fields.iter());
            let inhabitants_product = quote!(1 #(* #inhabitants)*);

            inhabitants.reverse();
            to_usize.reverse();

            match data.fields {
                Fields::Named(_) => {
                    let names: Vec<_> = data
                        .fields
                        .iter()
                        .map(|f| f.ident.as_ref().unwrap())
                        .collect();
                    FiniteImpl {
                        inhabitants: inhabitants_product,
                        to_usize: quote! {
                            {
                                let Self { #(#names,)* } = v;
                                let mut __res = 0;
                                #(
                                    __res *= #inhabitants;
                                    __res += #to_usize;
                                )*
                                __res
                            }
                        },
                        from_usize: quote! {
                            Some(Self { #(#names: #from_usize,)* })
                        },
                    }
                }
                Fields::Unnamed(_) => FiniteImpl {
                    inhabitants: inhabitants_product,
                    to_usize: quote! {
                        let mut res = 0;
                        #(
                            res *= #inhabitants;
                            res += #to_usize;
                        )*
                        res
                    },
                    from_usize: quote! {
                        Some(Self(#(#from_usize,)*))
                    },
                },
                Fields::Unit => FiniteImpl {
                    inhabitants: inhabitants_product,
                    to_usize: quote!(0),
                    from_usize: quote! {
                        Some(Self)
                    },
                },
            }
        }
        Data::Enum(ref data) => {
            let impls: Vec<_> = data.variants.iter().map(finite_impl_for_variant).collect();
            let FiniteImpls {
                inhabitants,
                to_usize,
                from_usize,
            } = impls.into();

            FiniteImpl {
                inhabitants: quote!(0 #(+ #inhabitants)*),
                to_usize: quote! {
                    let mut offset = 0;
                    #(
                        if let Some(i) = #to_usize {
                            return offset + i;
                        }
                        offset += #inhabitants;
                    )*
                    unreachable!()
                },
                from_usize: quote! {
                    #(
                        if i < #inhabitants {
                            return #from_usize;
                        }
                        i -= #inhabitants;
                    )*
                    unreachable!()
                },
            }
        }
        Data::Union(_) => unimplemented!(),
    }
}

fn finite_impl_for_variant(variant: &Variant) -> FiniteImpl {
    let name = &variant.ident;
    let FiniteImpls {
        mut inhabitants,
        mut to_usize,
        from_usize,
    } = finite_impls_for_fields(variant.fields.iter());
    let inhabitants_product = quote!(1 #(* #inhabitants)*);

    inhabitants.reverse();
    to_usize.reverse();

    let field_names =
        (0..variant.fields.len()).map(|i| Ident::new(&format!("_v{i}"), Span::call_site()));

    match variant.fields {
        Fields::Named(_) => {
            let names: Vec<_> = variant
                .fields
                .iter()
                .map(|f| f.ident.as_ref().unwrap())
                .collect();

            FiniteImpl {
                inhabitants: inhabitants_product,
                to_usize: quote! {
                    if let Self::#name { #(#names,)* } = v {
                        let mut __res = 0;
                        #(
                            __res *= #inhabitants;
                            __res += #to_usize;
                        )*
                        Some(__res)
                    } else {
                        None
                    }
                },
                from_usize: quote! {
                    Some(Self::#name { #(#names: #from_usize,)* })
                },
            }
        }
        Fields::Unnamed(_) => {
            let tuple = quote! {
                (#(#field_names,)*)
            };
            FiniteImpl {
                inhabitants: inhabitants_product,
                to_usize: quote! {
                    if let Self::#name #tuple = v {
                        let v = #tuple;
                        let mut res = 0;
                        #(
                            res *= #inhabitants;
                            res += #to_usize;
                        )*
                        Some(res)
                    } else {
                        None
                    }
                },
                from_usize: quote! {
                    Some(Self::#name(#(#from_usize,)*))
                },
            }
        }
        Fields::Unit => FiniteImpl {
            inhabitants: inhabitants_product,
            to_usize: quote! {
                if let Self::#name = v {
                    Some(0)
                } else {
                    None
                }
            },
            from_usize: quote! {
                if i == 0 {
                    Some(Self::#name)
                } else {
                    None
                }
            },
        },
    }
}

fn finite_impls_for_fields<'a>(fields: impl Iterator<Item = &'a Field>) -> FiniteImpls {
    let impls: Vec<_> = fields
        .enumerate()
        .map(|(i, f)| finite_impl_for_field(f, i))
        .collect();
    impls.into()
}

fn finite_impl_for_field(field: &Field, i: usize) -> FiniteImpl {
    let ty = &field.ty;
    let access = match &field.ident {
        Some(name) => quote!(#name),
        None => {
            let index = Index::from(i);
            quote!(&v.#index)
        }
    };
    let inhabitants = quote_spanned! { field.span() =>
        <#ty as exhaustive_map::Finite>::INHABITANTS
    };
    FiniteImpl {
        to_usize: quote_spanned! { field.span() =>
            <#ty as exhaustive_map::Finite>::to_usize(#access)
        },
        from_usize: quote_spanned! { field.span() =>
            {
                let v = <#ty as exhaustive_map::Finite>::from_usize(i % #inhabitants).unwrap();
                i /= #inhabitants;
                v
            }
        },
        inhabitants,
    }
}
