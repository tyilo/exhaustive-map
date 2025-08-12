use std::borrow::Borrow;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote, quote_spanned};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput, Field, Fields,
    GenericParam, Generics, Ident, Index, LitInt, Path, Variant,
};

struct Output {
    value: proc_macro2::TokenStream,
    bounds: Vec<proc_macro2::TokenStream>,
}

fn sum<T: Borrow<proc_macro2::TokenStream>>(iter: impl IntoIterator<Item = T>) -> Output {
    let mut iter = iter.into_iter();
    let first = iter.next();
    let mut output = match first {
        None => {
            return Output {
                value: quote!(::exhaustive_map::typenum::consts::U0),
                bounds: vec![],
            };
        }
        Some(first) => Output {
            value: first.borrow().clone(),
            bounds: vec![],
        },
    };
    for v in iter {
        let value = output.value;
        let v = v.borrow();
        output.value = quote! {
            <#value as ::core::ops::Add::<#v>>::Output
        };
        output.bounds.push(quote! {
            #value: ::core::ops::Add::<#v>
        });
    }
    output
}

fn prod<T: Borrow<proc_macro2::TokenStream>>(iter: impl IntoIterator<Item = T>) -> Output {
    let mut iter = iter.into_iter();
    let first = iter.next();
    let mut output = match first {
        None => {
            return Output {
                value: quote!(::exhaustive_map::typenum::consts::U1),
                bounds: vec![],
            };
        }
        Some(first) => Output {
            value: first.borrow().clone(),
            bounds: vec![],
        },
    };
    for v in iter {
        let value = output.value;
        let v = v.borrow();
        output.value = quote! {
            <#value as ::core::ops::Mul::<#v>>::Output
        };
        output.bounds.push(quote! {
            #value: ::core::ops::Mul::<#v>
        });
    }
    output
}

#[proc_macro]
pub fn __impl_tuples(input: TokenStream) -> TokenStream {
    let v = parse_macro_input!(input as LitInt);
    let n: usize = v.base10_parse().unwrap();

    let mut res = Vec::<TokenStream>::new();
    for k in 0..=n {
        let indices = 0..k;
        let idents: Vec<_> = indices
            .clone()
            .map(|i| Ident::new(&format!("T{i}"), Span::call_site()))
            .collect();

        let rev_indices = indices.clone().map(Index::from).rev();
        let rev_idents = idents.iter().rev();

        let inhabitants = prod(idents.iter().map(|i| quote!(#i::INHABITANTS)));
        let inhabitants_value = inhabitants.value;
        let mut bounds = inhabitants.bounds;
        bounds.push(quote!(#inhabitants_value: ::exhaustive_map::generic_array::ArrayLength + ::exhaustive_map::FitsInUsize));

        res.push(
            quote! {
                #[automatically_derived]
                impl <#( #idents: ::exhaustive_map::Finite ),*> ::exhaustive_map::Finite for (#( #idents, )*) where #(#bounds,)* {
                    type INHABITANTS = #inhabitants_value;

                    fn to_usize(&self) -> usize {
                        let mut res = 0;
                        #(
                            res *= <#rev_idents::INHABITANTS as ::exhaustive_map::typenum::Unsigned>::USIZE;
                            res += self.#rev_indices.to_usize();
                        )*
                        res
                    }

                    fn from_usize(mut i: usize) -> Option<Self> {
                        if i >= <Self::INHABITANTS as ::exhaustive_map::typenum::Unsigned>::USIZE {
                            return None;
                        }
                        Some((#(
                            {
                                let v = #idents::from_usize(i % <#idents::INHABITANTS as ::exhaustive_map::typenum::Unsigned>::USIZE).unwrap();
                                i /= <#idents::INHABITANTS as ::exhaustive_map::typenum::Unsigned>::USIZE;
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
    let generics = add_trait_bounds(generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let FiniteImpl {
        mut bounds,
        inhabitants,
        to_usize,
        from_usize,
    } = finite_impl(data);

    bounds.push(quote!(#inhabitants: ::exhaustive_map::generic_array::ArrayLength + ::exhaustive_map::FitsInUsize));

    let where_clause = match where_clause {
        None => Some(quote!(where #(#bounds,)*)),
        Some(w) => Some(quote!(#w, #(#bounds,)*)),
    };

    quote! {
        #[automatically_derived]
        impl #impl_generics ::exhaustive_map::Finite for #path #ty_generics #where_clause {
            type INHABITANTS = #inhabitants;

            #[allow(non_snake_case)]
            fn to_usize(&self) -> usize {
                let v = self;
                #to_usize
            }

            #[allow(clippy::let_unit_value)]
            #[allow(clippy::modulo_one)]
            fn from_usize(mut i: usize) -> Option<Self> {
                if i >= <Self::INHABITANTS as ::exhaustive_map::typenum::Unsigned>::USIZE {
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
    bounds: Vec<proc_macro2::TokenStream>,
    inhabitants: proc_macro2::TokenStream,
    to_usize: proc_macro2::TokenStream,
    from_usize: proc_macro2::TokenStream,
}

struct FiniteImpls {
    bounds: Vec<Vec<proc_macro2::TokenStream>>,
    inhabitants: Vec<proc_macro2::TokenStream>,
    to_usize: Vec<proc_macro2::TokenStream>,
    from_usize: Vec<proc_macro2::TokenStream>,
}

impl FromIterator<FiniteImpl> for FiniteImpls {
    fn from_iter<T: IntoIterator<Item = FiniteImpl>>(iter: T) -> Self {
        let mut bounds = vec![];
        let mut inhabitants = vec![];
        let mut to_usize = vec![];
        let mut from_usize = vec![];
        for imp in iter {
            bounds.push(imp.bounds);
            inhabitants.push(imp.inhabitants);
            to_usize.push(imp.to_usize);
            from_usize.push(imp.from_usize);
        }
        FiniteImpls {
            bounds,
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
                bounds,
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
                bounds,
                inhabitants,
                to_usize,
                from_usize,
            }
        }
        Data::Enum(ref data) => {
            let mut partial_inhabitants = vec![];
            let FiniteImpls {
                bounds,
                inhabitants,
                to_usize,
                from_usize,
            } = data
                .variants
                .iter()
                .map(|v| {
                    let finite_impl =
                        finite_impl_for_variant(v, quote!(0 #(+ <#partial_inhabitants as ::exhaustive_map::typenum::Unsigned>::USIZE)*));
                    partial_inhabitants.push(finite_impl.inhabitants.clone());
                    finite_impl
                })
                .collect();

            let mut bounds: Vec<_> = bounds.into_iter().flatten().collect();
            let Output {
                value: new_inhabitants,
                bounds: mut new_bounds,
            } = sum(&inhabitants);
            bounds.append(&mut new_bounds);
            FiniteImpl {
                to_usize: quote! {
                    match *v {
                        #(#to_usize,)*
                    }
                },
                from_usize: quote! {
                    #(
                        if i < <#inhabitants as ::exhaustive_map::typenum::Unsigned>::USIZE {
                            return #from_usize;
                        }
                        i -= <#inhabitants as ::exhaustive_map::typenum::Unsigned>::USIZE;
                    )*
                    unreachable!()
                },
                bounds,
                inhabitants: new_inhabitants,
            }
        }
        Data::Union(_) => panic!("Finite can't be derived for unions"),
    }
}

fn finite_impl_for_variant(variant: &Variant, offset: proc_macro2::TokenStream) -> FiniteImpl {
    let name = &variant.ident;
    let FiniteImpl {
        bounds,
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
        bounds,
        inhabitants,
        to_usize,
        from_usize,
    }
}

fn finite_impl_for_fields(fields: &Fields, constructor: proc_macro2::TokenStream) -> FiniteImpl {
    let FiniteImpls {
        bounds,
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

    let mut bounds: Vec<_> = bounds.into_iter().flatten().collect();
    let Output {
        value,
        bounds: mut new_bounds,
    } = prod(&inhabitants);
    bounds.append(&mut new_bounds);
    FiniteImpl {
        bounds,
        inhabitants: value,
        to_usize: quote! {
            {
                let mut res = 0;
                #(
                    res *= <#inhabitants as ::exhaustive_map::typenum::Unsigned>::USIZE;
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
                let v = <#ty as ::exhaustive_map::Finite>::from_usize(i % <#inhabitants as ::exhaustive_map::typenum::Unsigned>::USIZE).unwrap();
                i /= <#inhabitants as ::exhaustive_map::typenum::Unsigned>::USIZE;
                v
            }
        },
        bounds: vec![quote!(#ty: ::exhaustive_map::Finite)],
        inhabitants,
    }
}

fn mapped_field_name(ident: &Ident) -> proc_macro2::TokenStream {
    let ident = format_ident!("field_{}", ident);
    quote!(#ident)
}

#[cfg(test)]
mod test {
    use syn::{
        parse_quote,
        token::{Brace, Enum},
        DataEnum, ItemEnum,
    };

    use super::*;

    #[test]
    fn test_derive() {
        let item_enum: ItemEnum = parse_quote! {
            enum Color {
                Red,
                Green,
                Blue,
            }
        };

        let data = Data::Enum(DataEnum {
            enum_token: item_enum.enum_token,
            brace_token: item_enum.brace_token,
            variants: item_enum.variants,
        });

        let x = impl_finite(&item_enum.ident.into(), item_enum.generics, &data);
        panic!("{x}");
    }
}

// From https://github.com/paholg/typenum/pull/136/files
use proc_macro2::TokenStream as TokenStream2;
use syn::parse::{Parse, ParseStream, Result as ParseResult};

struct UnsignedInteger {
    value: u128,
}

impl Parse for UnsignedInteger {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let literal = input.parse::<LitInt>()?;
        let value = literal.base10_parse::<u128>()?;

        let output = UnsignedInteger { value };

        Ok(output)
    }
}

#[proc_macro]
pub fn uint(input: TokenStream) -> TokenStream {
    let UnsignedInteger { value } = parse_macro_input!(input as UnsignedInteger);

    let tokens = recursive_value_to_typeuint(value);
    TokenStream::from(tokens)
}

fn recursive_value_to_typeuint(value: u128) -> TokenStream2 {
    if value == 0 {
        quote! {
            ::exhaustive_map::typenum::uint::UTerm
        }
    } else if value & 1 == 1 {
        let sub_tokens = recursive_value_to_typeuint(value >> 1);
        quote! {
            ::exhaustive_map::typenum::uint::UInt<#sub_tokens, ::exhaustive_map::typenum::bit::B1>
        }
    } else {
        let sub_tokens = recursive_value_to_typeuint(value >> 1);
        quote! {
            ::exhaustive_map::typenum::uint::UInt<#sub_tokens, ::exhaustive_map::typenum::bit::B0>
        }
    }
}
