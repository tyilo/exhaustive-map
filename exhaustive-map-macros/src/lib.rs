use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote, quote_spanned};
use syn::{
    bracketed, parse::Parse, parse_macro_input, parse_quote, punctuated::Punctuated,
    spanned::Spanned, Data, DeriveInput, Field, Fields, GenericParam, Generics, Ident, Index,
    LitInt, Path, Token, TypeParamBound, Variant,
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

    impl_finite(
        &typ,
        Default::default(),
        &Data::Enum(syn::DataEnum {
            enum_token: Default::default(),
            brace_token: Default::default(),
            variants: inhabitants
                .into_iter()
                .map(|ident| Variant {
                    attrs: Default::default(),
                    ident,
                    fields: Fields::Unit,
                    discriminant: Default::default(),
                })
                .collect(),
        }),
    )
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

        let rev_indices = indices.clone().map(Index::from).rev();
        let rev_idents = idents.iter().rev();

        res.push(
            quote! {
                #[automatically_derived]
                impl <#( #idents: ::exhaustive_map::Finite ),*> ::exhaustive_map::Finite for (#( #idents, )*) {
                    const INHABITANTS: usize = 1 #( * #idents::INHABITANTS )*;

                    fn to_usize(&self) -> usize {
                        let mut res = 0;
                        #(
                            res *= #rev_idents::INHABITANTS;
                            res += self.#rev_indices.to_usize();
                        )*
                        res
                    }
                }

                #[automatically_derived]
                impl <#( #idents: ::exhaustive_map::FiniteExt ),*> ::exhaustive_map::FiniteExt for (#( #idents, )*) {
                    fn from_usize(mut i: usize) -> Option<Self> {
                        if i >= Self::INHABITANTS {
                            return None;
                        }
                        Some((#(
                            {
                                let v = #idents::from_usize(i % #idents::INHABITANTS).unwrap();
                                i /= #idents::INHABITANTS;
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

#[proc_macro_derive(Finite, attributes(__finite_foreign))]
pub fn finite_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let foreign_attrs: Vec<_> = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("__finite_foreign"))
        .collect();

    let path = match foreign_attrs[..] {
        [] => input.ident.into(),
        [attr] => match attr.parse_args() {
            Ok(path) => path,
            Err(e) => return e.to_compile_error().into(),
        },
        _ => panic!("Only one `finite_foreign` attribute allowed"),
    };

    impl_finite(&path, input.generics, &input.data).into()
}

fn impl_finite(path: &Path, generics: Generics, data: &Data) -> proc_macro2::TokenStream {
    let FiniteImpl {
        inhabitants,
        to_usize,
        from_usize,
    } = finite_impl(data);

    let finite_impl = {
        let generics = add_trait_bounds(generics.clone(), parse_quote!(::exhaustive_map::Finite));
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        quote! {
            #[automatically_derived]
            impl #impl_generics ::exhaustive_map::Finite for #path #ty_generics #where_clause {
                const INHABITANTS: usize = #inhabitants;

                #[allow(non_snake_case)]
                fn to_usize(&self) -> usize {
                    let v = self;
                    #to_usize
                }
            }
        }
    };
    let finite_ext_impl = {
        let generics = add_trait_bounds(generics, parse_quote!(::exhaustive_map::FiniteExt));
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        quote! {
            #[automatically_derived]
            impl #impl_generics ::exhaustive_map::FiniteExt for #path #ty_generics #where_clause {
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
    };

    [finite_impl, finite_ext_impl].into_iter().collect()
}

fn add_trait_bounds(mut generics: Generics, bound: TypeParamBound) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(bound.clone());
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

impl FromIterator<FiniteImpl> for FiniteImpls {
    fn from_iter<T: IntoIterator<Item = FiniteImpl>>(iter: T) -> Self {
        let mut inhabitants = vec![];
        let mut to_usize = vec![];
        let mut from_usize = vec![];
        for imp in iter {
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
            let FiniteImpl {
                inhabitants,
                to_usize,
                from_usize,
            } = finite_impl_for_fields(&data.fields, quote!(Self));

            let to_usize = match data.fields {
                Fields::Named(_) => {
                    let names: Vec<_> = data
                        .fields
                        .iter()
                        .map(|f| f.ident.as_ref().unwrap())
                        .collect();
                    let mapped_names = names.iter().map(|name| mapped_field_name(name));

                    quote! {
                        {
                            let Self { #(#names: #mapped_names,)* } = v;
                            #to_usize
                        }
                    }
                }
                Fields::Unnamed(_) => to_usize,
                Fields::Unit => to_usize,
            };

            FiniteImpl {
                inhabitants,
                to_usize,
                from_usize,
            }
        }
        Data::Enum(ref data) => {
            let mut inhabitants = vec![];
            let FiniteImpls {
                inhabitants,
                to_usize,
                from_usize,
            } = data
                .variants
                .iter()
                .map(|v| {
                    let finite_impl = finite_impl_for_variant(v, quote!(0 #(+ #inhabitants)*));
                    inhabitants.push(finite_impl.inhabitants.clone());
                    finite_impl
                })
                .collect();

            FiniteImpl {
                inhabitants: quote!(0 #(+ #inhabitants)*),
                to_usize: quote! {
                    match *v {
                        #(#to_usize,)*
                    }
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
        Data::Union(_) => panic!("Finite can't be derived for unions"),
    }
}

fn finite_impl_for_variant(variant: &Variant, offset: proc_macro2::TokenStream) -> FiniteImpl {
    let name = &variant.ident;
    let FiniteImpl {
        inhabitants,
        to_usize,
        from_usize,
    } = finite_impl_for_fields(&variant.fields, quote!(Self::#name));

    let to_usize = match variant.fields {
        Fields::Named(_) => {
            let names: Vec<_> = variant
                .fields
                .iter()
                .map(|f| f.ident.as_ref().unwrap())
                .collect();
            let mapped_names = names.iter().map(|name| mapped_field_name(name));

            quote! {
                Self::#name { #(#names: ref #mapped_names,)* } => {
                    (#to_usize + #offset)
                }
            }
        }
        Fields::Unnamed(_) => {
            let field_names: Vec<_> = (0..variant.fields.len())
                .map(|i| Ident::new(&format!("v_{i}"), Span::call_site()))
                .collect();
            quote! {
                Self::#name(#(ref #field_names,)*) => {
                    let v = (#(#field_names,)*);
                    (#to_usize + #offset)
                }
            }
        }
        Fields::Unit => quote! {
            Self::#name => #offset
        },
    };

    FiniteImpl {
        inhabitants,
        to_usize,
        from_usize,
    }
}

fn finite_impl_for_fields(fields: &Fields, constructor: proc_macro2::TokenStream) -> FiniteImpl {
    let FiniteImpls {
        mut inhabitants,
        mut to_usize,
        from_usize,
    } = finite_impls_for_fields(fields.iter());

    inhabitants.reverse();
    to_usize.reverse();

    let from_usize = match fields {
        Fields::Named(_) => {
            let names: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();
            quote! {
                Some(#constructor { #(#names: #from_usize,)* })
            }
        }
        Fields::Unnamed(_) => {
            quote! {
                Some(#constructor(#(#from_usize,)*))
            }
        }
        Fields::Unit => quote! {
            Some(#constructor)
        },
    };

    FiniteImpl {
        inhabitants: quote!(1 #(* #inhabitants)*),
        to_usize: quote! {
            {
                let mut res = 0;
                #(
                    res *= #inhabitants;
                    res += #to_usize;
                )*
                res
            }
        },
        from_usize,
    }
}

fn finite_impls_for_fields<'a>(fields: impl Iterator<Item = &'a Field>) -> FiniteImpls {
    fields
        .enumerate()
        .map(|(i, f)| finite_impl_for_field(f, i))
        .collect()
}

fn finite_impl_for_field(field: &Field, i: usize) -> FiniteImpl {
    let ty = &field.ty;
    let access = match &field.ident {
        Some(name) => mapped_field_name(name),
        None => {
            let index = Index::from(i);
            quote!(&v.#index)
        }
    };
    let inhabitants = quote_spanned! { field.span() =>
        <#ty as ::exhaustive_map::Finite>::INHABITANTS
    };
    FiniteImpl {
        to_usize: quote_spanned! { field.span() =>
            <#ty as ::exhaustive_map::Finite>::to_usize(#access)
        },
        from_usize: quote_spanned! { field.span() =>
            {
                let v = <#ty as ::exhaustive_map::FiniteExt>::from_usize(i % #inhabitants).unwrap();
                i /= #inhabitants;
                v
            }
        },
        inhabitants,
    }
}

fn mapped_field_name(ident: &Ident) -> proc_macro2::TokenStream {
    let ident = format_ident!("field_{}", ident);
    quote!(#ident)
}
