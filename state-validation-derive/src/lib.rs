use std::collections::{BTreeSet, HashMap};

use itertools::Itertools;
use proc_macro::TokenStream;
use quote::TokenStreamExt;
use syn::{
    Expr, GenericArgument, GenericParam, Generics, Ident, Lifetime, Type, TypePath,
    parse_macro_input, parse_quote,
};

#[derive(Clone, PartialEq, Eq, Hash)]
struct ConversionSort {
    sort_number: usize,
    ty: ConversionType,
}
impl Ord for ConversionSort {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sort_number.cmp(&other.sort_number)
    }
}
impl PartialOrd for ConversionSort {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl quote::ToTokens for ConversionSort {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.ty.to_tokens(tokens)
    }
}
#[derive(Clone, PartialEq, Eq, Hash)]
enum ConversionType {
    Type(syn::Type),
    Generic {
        generic_ident: Vec<syn::Ident>,
        path: syn::Path,
    },
}
impl syn::parse::Parse for ConversionType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Ident) && input.peek2(syn::Token![=]) {
            let generic_ident = input.parse()?;
            let _: syn::Token![=] = input.parse()?;
            let path = input.parse()?;
            Ok(ConversionType::Generic {
                generic_ident: vec![generic_ident],
                path,
            })
        } else {
            input.parse().map(ConversionType::Type)
        }
    }
}
impl quote::ToTokens for ConversionType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ConversionType::Type(ty) => {
                tokens.append_all(ty.to_token_stream());
            }
            ConversionType::Generic {
                generic_ident,
                path,
            } => {
                tokens.append_all(quote::quote!(#path));
            }
        }
    }
}

#[proc_macro_derive(StateFilterConversion, attributes(conversion))]
pub fn state_filter_conversion(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let generics: Vec<_> = ast
        .generics
        .type_params()
        .into_iter()
        .map(|ty| ty.ident.clone())
        .collect();
    let state_conversions = match &ast.data {
        syn::Data::Struct(s) => {
            let fields_count = s.fields.len();
            let mut state_conversions = Vec::with_capacity(fields_count);
            let iter: Vec<_> = s
                .fields
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    let field_name = field.ident.as_ref().expect("expected a named field");
                    let mut all_conversion_fields = Vec::new();
                    all_conversion_fields.push((
                        field_name,
                        ConversionSort {
                            sort_number: i,
                            ty: ConversionType::Type(field.ty.clone()),
                        },
                        extract_generics_from_type(&field.ty),
                    ));
                    for attr in field
                        .attrs
                        .iter()
                        .filter(|attr| attr.path().is_ident("conversion"))
                    {
                        let f = attr
                            .parse_args::<ConversionType>()
                            .expect("expected a conversion type");
                        let generics = match &f {
                            ConversionType::Type(ty) => extract_generics_from_type(ty),
                            ConversionType::Generic { generic_ident, .. } => {
                                parse_quote!(<#(#generic_ident),*>)
                            }
                        };
                        all_conversion_fields.push((
                            field_name,
                            ConversionSort {
                                sort_number: i,
                                ty: f,
                            },
                            generics,
                        ));
                    }
                    all_conversion_fields
                })
                .collect();
            let mut combination_names = HashMap::new();
            let mut remainder_names = HashMap::new();
            for (i, (field_names, mut field_types, field_generics)) in iter
                .iter()
                .multi_cartesian_product()
                .map(|f| {
                    let mut field_names = Vec::with_capacity(f.len());
                    let mut field_types = Vec::with_capacity(f.len());
                    let mut generics = Vec::with_capacity(f.len());
                    for (field_name, field_type, field_generics) in f {
                        field_names.push(*field_name);
                        field_types.push(field_type.clone());
                        generics.push(field_generics);
                    }
                    (field_names, field_types, generics)
                })
                .enumerate()
            {
                let combination_struct_name =
                    quote::format_ident!("__StateValidationGeneration_{name}Combined_{i}");
                let mut generics = Generics::default();
                for g in field_generics {
                    generics = merge_generics(generics, g);
                }
                let q = quote::quote! {
                    pub struct #combination_struct_name #generics {
                        #(pub #field_names: #field_types),*
                    }
                };
                state_conversions.push(q);
                field_types.sort();
                combination_names.insert(field_types, combination_struct_name);
            }
            let mut i = 0;
            for powerset in iter.iter().powerset() {
                for (field_names, mut field_types, field_generics) in
                    powerset.into_iter().multi_cartesian_product().map(|f| {
                        let mut field_names = Vec::with_capacity(f.len());
                        let mut field_types = Vec::with_capacity(f.len());
                        let mut generics = Vec::with_capacity(f.len());
                        for (field_name, field_type, field_generics) in f {
                            field_names.push(*field_name);
                            field_types.push(field_type.clone());
                            generics.push(field_generics);
                        }
                        (field_names, field_types, generics)
                    })
                {
                    let remainder_struct_name =
                        quote::format_ident!("__StateValidationGeneration_{name}Remainder_{i}");
                    let mut generics = Generics::default();
                    for g in field_generics {
                        generics = merge_generics(generics, g);
                    }
                    let q = quote::quote! {
                        pub struct #remainder_struct_name #generics {
                            #(#field_names: #field_types),*
                        }
                    };
                    state_conversions.push(q);
                    field_types.sort();
                    remainder_names.insert(field_types, remainder_struct_name);
                    i += 1;
                }
            }
            create_original_conversion_combinations(
                &mut state_conversions,
                &combination_names,
                &remainder_names,
                name,
                &s.fields,
                generics,
            );
            let cartesian_product = iter.iter().multi_cartesian_product().map(|f| {
                let mut field_names = Vec::with_capacity(f.len());
                let mut field_types = Vec::with_capacity(f.len());
                let mut generics = Vec::with_capacity(f.len());
                for (field_name, field_type, field_generics) in f {
                    field_names.push(field_name);
                    field_types.push(field_type);
                    generics.push(field_generics);
                }
                (field_names, field_types, generics)
            });
            for (k, (field_names, field_types, field_generics)) in cartesian_product.enumerate() {
                let mut all_field_generics = Generics::default();
                for field_generics in field_generics.iter() {
                    all_field_generics = merge_generics(all_field_generics, field_generics);
                }
                let fields_name_type_generics: Vec<_> = field_names
                    .clone()
                    .into_iter()
                    .zip(field_types.clone().into_iter())
                    .zip(field_generics.clone().into_iter())
                    .collect();
                for count in 0..=fields_count {
                    for f in fields_name_type_generics.iter().combinations(count) {
                        for (
                            current_field_names,
                            current_field_types,
                            current_field_generics,
                            other_field_names,
                            other_field_types,
                            other_field_generics,
                        ) in f.into_iter().permutations(count).map(|subset| {
                            let remainder: Vec<_> = fields_name_type_generics
                                .iter()
                                .filter(|((field_name_a, ..), ..)| {
                                    !subset.iter().any(|((field_name_b, ..), ..)| {
                                        field_name_a == field_name_b
                                    })
                                })
                                .collect();
                            let mut current_field_names = Vec::with_capacity(subset.len());
                            let mut current_field_types = Vec::with_capacity(subset.len());
                            let mut current_field_generics = Vec::with_capacity(subset.len());
                            for ((field_name, field_type), generics) in subset {
                                current_field_names.push(**field_name);
                                current_field_types.push((*field_type).clone());
                                current_field_generics.push(generics);
                            }
                            let mut other_field_names = Vec::with_capacity(remainder.len());
                            let mut other_field_types = Vec::with_capacity(remainder.len());
                            let mut other_field_generics = Vec::with_capacity(remainder.len());
                            for ((field_name, field_type), generics) in remainder {
                                other_field_names.push(**field_name);
                                other_field_types.push((*field_type).clone());
                                other_field_generics.push(generics);
                            }
                            (
                                current_field_names,
                                current_field_types,
                                current_field_generics,
                                other_field_names,
                                other_field_types,
                                other_field_generics,
                            )
                        }) {
                            let combined_struct_name = combination_names
                                .get(
                                    &current_field_types
                                        .iter()
                                        .chain(other_field_types.iter())
                                        .cloned()
                                        .sorted()
                                        .collect::<Vec<_>>(),
                                )
                                .expect("0: expected a combined struct");
                            let remainder_struct_name = {
                                let mut other_field_types = other_field_types.clone();
                                other_field_types.sort();
                                remainder_names.get(&other_field_types).unwrap()
                            };
                            let mut o = Generics::default();
                            for other_field_generics in other_field_generics {
                                o = merge_generics(o, other_field_generics);
                            }
                            let other_field_generics = o;
                            let q = quote::quote! {
                                impl #all_field_generics state_validation::StateFilterInputCombination<(#(#current_field_types),*)> for #remainder_struct_name #other_field_generics {
                                    type Combined = #combined_struct_name #all_field_generics;
                                    fn combine(self, (#(#current_field_names),*): (#(#current_field_types),*)) -> Self::Combined {
                                        #combined_struct_name {
                                            #(#current_field_names,)*
                                            #(#other_field_names: self.#other_field_names),*
                                        }
                                    }
                                }
                                impl #all_field_generics state_validation::StateFilterInputConversion<(#(#current_field_types),*)> for #combined_struct_name #all_field_generics {
                                    type Remainder = #remainder_struct_name #other_field_generics;
                                    fn split_take(self) -> ((#(#current_field_types),*), Self::Remainder) {
                                        (
                                            (#(self.#current_field_names),*),
                                            #remainder_struct_name {
                                                #(#other_field_names: self.#other_field_names),*
                                            },
                                        )
                                    }
                                }
                            };
                            state_conversions.push(q);
                        }
                    }
                }
            }
            state_conversions
        }
        _ => todo!(),
    };
    quote::quote! {
        //impl #impl_generics state_validation::StateFilterInput for #name #ty_generics #where_clause {}
        #(#state_conversions)*
    }
    .into()
}

fn create_original_conversion_combinations(
    state_conversions: &mut Vec<proc_macro2::TokenStream>,
    combination_names: &HashMap<Vec<ConversionSort>, Ident>,
    remainder_names: &HashMap<Vec<ConversionSort>, Ident>,
    name: &Ident,
    fields: &syn::Fields,
    all_field_generics: Vec<Ident>,
) {
    let fields: Vec<_> = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let field_name = field.ident.as_ref().expect("expected a named field");
            (
                field_name,
                ConversionSort {
                    sort_number: i,
                    ty: ConversionType::Type(field.ty.clone()),
                },
                extract_generics_from_type(&field.ty),
            )
        })
        .collect();
    let mut all_field_generics: Generics = parse_quote!();
    for (_, _, generics_b) in fields.iter() {
        all_field_generics = merge_generics(all_field_generics, generics_b);
    }
    for k in 0..=fields.len() {
        for combination in fields.iter().combinations(k) {
            for (
                current_field_names,
                current_field_types,
                current_field_generics,
                other_field_names,
                other_field_types,
                other_field_generics,
            ) in combination.into_iter().permutations(k).map(|subset| {
                let remainder: Vec<_> = fields
                    .iter()
                    .filter(|(field_name_a, ..)| {
                        !subset
                            .iter()
                            .any(|(field_name_b, ..)| field_name_a == field_name_b)
                    })
                    .collect();
                let mut current_field_names = Vec::with_capacity(subset.len());
                let mut current_field_types = Vec::with_capacity(subset.len());
                let mut current_field_generics = Vec::new();
                for (field_name, field_type, generics) in subset {
                    current_field_names.push(*field_name);
                    current_field_types.push((*field_type).clone());
                    current_field_generics.push(generics);
                }
                let mut other_field_names = Vec::with_capacity(remainder.len());
                let mut other_field_types = Vec::with_capacity(remainder.len());
                let mut other_field_generics = Vec::new();
                for (field_name, field_type, generics) in remainder {
                    other_field_names.push(*field_name);
                    other_field_types.push((*field_type).clone());
                    other_field_generics.push(generics);
                }
                (
                    current_field_names,
                    current_field_types,
                    current_field_generics,
                    other_field_names,
                    other_field_types,
                    other_field_generics,
                )
            }) {
                let combined_struct_name = combination_names
                    .get(
                        &current_field_types
                            .iter()
                            .chain(other_field_types.iter())
                            .cloned()
                            .sorted()
                            .collect::<Vec<_>>(),
                    )
                    .expect("1: expected a combined struct");
                let remainder_struct_name = {
                    let mut other_field_types = other_field_types.clone();
                    other_field_types.sort();
                    remainder_names
                        .get(&other_field_types)
                        .expect("expected a remainder struct")
                };
                let mut current_field_generic = Generics::default();
                for current_generics in current_field_generics {
                    current_field_generic = merge_generics(current_field_generic, current_generics);
                }
                let current_field_generics = current_field_generic;
                let mut other_field_generic = Generics::default();
                for other_generics in other_field_generics {
                    other_field_generic = merge_generics(other_field_generic, other_generics);
                }
                let other_field_generics = other_field_generic;
                let q = quote::quote! {
                    impl #all_field_generics state_validation::StateFilterInputConversion<(#(#current_field_types),*)> for #name #all_field_generics {
                        type Remainder = #remainder_struct_name #other_field_generics;
                        fn split_take(self) -> ((#(#current_field_types),*), Self::Remainder) {
                            (
                                (#(self.#current_field_names),*),
                                #remainder_struct_name {
                                    #(#other_field_names: self.#other_field_names),*
                                },
                            )
                        }
                    }
                };
                state_conversions.push(q);
            }
        }
    }
}

// UTILITY //

fn extract_generics_from_type(ty: &Type) -> Generics {
    let mut type_params = BTreeSet::new();
    let mut lifetime_params = BTreeSet::new();
    let mut const_params = BTreeSet::new();

    collect_generics(
        ty,
        &mut type_params,
        &mut lifetime_params,
        &mut const_params,
    );

    let mut generics = Generics::default();

    for lt in lifetime_params {
        generics
            .params
            .push(GenericParam::Lifetime(parse_quote!(#lt)));
    }
    for tp in type_params {
        generics.params.push(GenericParam::Type(parse_quote!(#tp)));
    }
    for cp in const_params {
        generics
            .params
            .push(GenericParam::Const(parse_quote!(const #cp: usize)));
    }

    generics
}

fn collect_generics(
    ty: &Type,
    type_params: &mut BTreeSet<syn::Ident>,
    lifetime_params: &mut BTreeSet<Lifetime>,
    const_params: &mut BTreeSet<syn::Ident>,
) {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            for segment in &path.segments {
                // Extract angle bracketed generics
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    for arg in &args.args {
                        match arg {
                            GenericArgument::Type(inner_ty) => {
                                collect_generics(
                                    inner_ty,
                                    type_params,
                                    lifetime_params,
                                    const_params,
                                );
                            }
                            GenericArgument::Lifetime(lt) => {
                                lifetime_params.insert(lt.clone());
                            }
                            GenericArgument::Const(expr) => {
                                if let syn::Expr::Path(expr_path) = expr
                                    && let Some(ident) = expr_path.path.get_ident()
                                {
                                    const_params.insert(ident.clone());
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Type::Reference(r) => {
            if let Some(lt) = &r.lifetime {
                lifetime_params.insert(lt.clone());
            }
            collect_generics(&r.elem, type_params, lifetime_params, const_params);
        }
        _ => {}
    }
}

fn merge_generics(mut generics_a: Generics, generics_b: &Generics) -> Generics {
    let mut existing = BTreeSet::new();
    for param in &generics_a.params {
        match param {
            GenericParam::Type(tp) => {
                existing.insert(tp.ident.to_string());
            }
            GenericParam::Lifetime(lt) => {
                existing.insert(lt.lifetime.ident.to_string());
            }
            GenericParam::Const(cp) => {
                existing.insert(cp.ident.to_string());
            }
        }
    }

    for param in &generics_b.params {
        let name = match param {
            GenericParam::Type(tp) => tp.ident.to_string(),
            GenericParam::Lifetime(lt) => lt.lifetime.ident.to_string(),
            GenericParam::Const(cp) => cp.ident.to_string(),
        };
        if !existing.contains(&name) {
            generics_a.params.push(param.clone());
            existing.insert(name);
        }
    }

    match (&mut generics_a.where_clause, &generics_b.where_clause) {
        (Some(a_wc), Some(b_wc)) => {
            a_wc.predicates.extend(b_wc.predicates.clone());
        }
        (None, Some(b_wc)) => {
            generics_a.where_clause = Some(b_wc.clone());
        }
        _ => {}
    }

    generics_a
}
