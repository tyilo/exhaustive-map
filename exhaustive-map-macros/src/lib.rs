use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    bracketed, parse::Parse, parse_macro_input, punctuated::Punctuated, Ident, Index, LitInt, Path,
    Token,
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
pub fn impl_enum(input: TokenStream) -> TokenStream {
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
pub fn impl_tuples(input: TokenStream) -> TokenStream {
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
