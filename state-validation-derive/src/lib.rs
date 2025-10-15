use darling::FromDeriveInput;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{Expr, Ident, Type, parse_macro_input};

#[derive(darling::FromDeriveInput)]
#[darling(attributes(state_filter_input))]
struct StateFilterInputData {
    remainder_type: Option<Type>,
    remainder: Option<Expr>,
}

#[proc_macro_derive(StateFilterInput, attributes(state_filter_input))]
pub fn state_filter_input(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let data = StateFilterInputData::from_derive_input(&ast).unwrap();
    let remainder_code = if let Some(remainder_type) = data.remainder_type {
        let remainder_expr = data.remainder.expect("expected `remainder` expression");
        quote::quote! {
            impl #impl_generics state_validation::StateFilterInputConversion<Self> for #name #ty_generics #where_clause {
                type Remainder = #remainder_type;
                fn split_take(self) -> (Self, Self::Remainder) {
                    (self, #remainder_expr)
                }
            }
        }
    } else {
        quote::quote! {
            impl #impl_generics state_validation::StateFilterInputConversion<Self> for #name #ty_generics #where_clause {
                type Remainder = ();
                fn split_take(self) -> (Self, Self::Remainder) {
                    (self, ())
                }
            }
        }
    };
    let mut generics_1 = ast.generics.clone();
    generics_1
        .params
        .push(syn::GenericParam::Type(syn::TypeParam {
            attrs: Vec::new(),
            ident: Ident::new("T", Span::call_site()),
            colon_token: None,
            bounds: syn::punctuated::Punctuated::new(),
            eq_token: None,
            default: None,
        }));
    let (_impl_generics_1, _ty_generics_1, _where_clause_1) = generics_1.split_for_impl();
    let mut generics_2 = ast.generics.clone();
    generics_2
        .params
        .push(syn::GenericParam::Type(syn::TypeParam {
            attrs: Vec::new(),
            ident: Ident::new("T0", Span::call_site()),
            colon_token: None,
            bounds: syn::punctuated::Punctuated::new(),
            eq_token: None,
            default: None,
        }));
    generics_2
        .params
        .push(syn::GenericParam::Type(syn::TypeParam {
            attrs: Vec::new(),
            ident: Ident::new("T1", Span::call_site()),
            colon_token: None,
            bounds: syn::punctuated::Punctuated::new(),
            eq_token: None,
            default: None,
        }));
    let (_impl_generics_2, _ty_generics_2, _where_clause_2) = generics_2.split_for_impl();
    quote::quote! {
        impl #impl_generics state_validation::StateFilterInput for #name #ty_generics #where_clause {}
        #remainder_code
        /*impl #impl_generics_1 card_game::validation::StateFilterInputConversion<#name #ty_generics> for (#name #ty_generics, T) #where_clause {
            type Remainder = (T,);
            fn combine(input: #name #ty_generics, remainder: Self::Remainder) -> Self {
                (input, remainder.0)
            }
            fn split_take(self) -> (#name #ty_generics, Self::Remainder) {
                (self.0, (self.1,))
            }
        }
        impl #impl_generics_2 card_game::validation::StateFilterInputConversion<#name #ty_generics> for (#name #ty_generics, T0, T1) #where_clause {
            type Remainder = (T0, T1);
            fn combine(input: #name #ty_generics, remainder: Self::Remainder) -> Self {
                (input, remainder.0, remainder.1)
            }
            fn split_take(self) -> (#name #ty_generics, Self::Remainder) {
                (self.0, (self.1, self.2))
            }
        }
        impl #impl_generics_2 card_game::validation::StateFilterInputConversion<(#name #ty_generics, T0)> for (#name #ty_generics, T0, T1) #where_clause {
            type Remainder = (T1,);
            fn combine(input: (#name #ty_generics, T0), remainder: Self::Remainder) -> Self {
                (input.0, input.1, remainder.0)
            }
            fn split_take(self) -> ((#name #ty_generics, T0), Self::Remainder) {
                ((self.0, self.1), (self.2,))
            }
        }*/
        /*impl #impl_generics_1 card_game::validation::StateFilterInputConversion<T> for (#name #ty_generics, T) #where_clause {
            type Remainder = (#name #ty_generics,);
            fn combine(input: T, remainder: Self::Remainder) -> Self {
                (remainder.0, input)
            }
            fn split_take(self) -> (T, Self::Remainder) {
                (self.0, (self.1,))
            }
        }*/
        /*impl #impl_generics_2 card_game::validation::StateFilterInputConversion<(#name #ty_generics, T0)> for (#name #ty_generics, T0, T1) #where_clause {
            type Remainder = (T1,);
            fn combine(input: (#name #ty_generics, T0), remainder: Self::Remainder) -> Self {
                (input.0, input.1, remainder.0)
            }
            fn split_take(self) -> ((#name #ty_generics, T0), Self::Remainder) {
                ((self.0, self.1), (self.2,))
            }
        }*/
        /*impl #impl_generics_2 card_game::validation::StateFilterInputConversion<(T0, T1)> for (T0, #name #ty_generics, T1) #where_clause {
            type Remainder = (#name #ty_generics,);
            fn combine(input: (T0, T1), remainder: Self::Remainder) -> Self {
                (input.0, remainder.0, input.1)
            }
            fn split_take(self) -> ((T0, T1), Self::Remainder) {
                ((self.0, self.2), (self.1,))
            }
        }
        impl #impl_generics_2 card_game::validation::StateFilterInputConversion<(T0, T1)> for (#name #ty_generics, T0, T1) #where_clause {
            type Remainder = (#name #ty_generics,);
            fn combine(input: (T0, T1), remainder: Self::Remainder) -> Self {
                (remainder.0, input.0, input.1)
            }
            fn split_take(self) -> ((T0, T1), Self::Remainder) {
                ((self.1, self.2), (self.0,))
            }
        }*/
    }
    .into()
}
