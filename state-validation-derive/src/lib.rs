use std::collections::{BTreeSet, HashMap};

use heck::ToSnakeCase;
use itertools::Itertools;
use proc_macro::TokenStream;
use quote::TokenStreamExt;
use syn::{
    Expr, GenericArgument, GenericParam, Generics, Ident, Lifetime, Type, TypePath, ext,
    parse_macro_input, parse_quote,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ConversionType {
    Type(syn::Type),
    Generic {
        generic_ident: Vec<syn::Ident>,
        ty: syn::Type,
    },
}
impl syn::parse::Parse for ConversionType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Ident) && input.peek2(syn::Token![=]) {
            let generic_ident = input.parse()?;
            let generic_ident = vec![generic_ident];
            let _: syn::Token![=] = input.parse()?;
            let ty = input.parse()?;
            Ok(ConversionType::Generic { generic_ident, ty })
        } else if input.peek(syn::Ident) && input.peek2(syn::Token![,]) {
            let mut generic_ident = Vec::with_capacity(2);
            loop {
                let generic = input.parse::<syn::Ident>()?;
                generic_ident.push(generic);
                if input.parse::<syn::Token![,]>().is_err() {
                    break;
                }
            }
            let _: syn::Token![=] = input.parse()?;
            let ty = input.parse()?;
            Ok(ConversionType::Generic { generic_ident, ty })
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
            ConversionType::Generic { ty, .. } => {
                tokens.append_all(quote::quote!(#ty));
            }
        }
    }
}

/// Implements `StateFilterInputConversion` and `StateFilterInputCombination`,
/// so that this struct can be split into every individual field and later combined together.
///
/// The types each field can be converted to must be specified.
///
/// Use `conversion` on the struct fields to convert them to a different type:
/// ```ignore
/// #[derive(StateFilterConversion)]
/// struct ExampleStruct {
///     #[conversion(AdminUser)]
///     some_value: UnknownUser,
/// }
/// # struct UnknownUser;
/// # struct AdminUser;
/// ```
/// The above code will let a `StateFilter` take `ExampleStruct` as an input,
/// and output a new struct whose `some_value` is of type `AdminUser` or the original `UnknownUser`.
///
/// In some cases, your filters may result in more data output than what was given.
/// In those cases, you can use the `conversion` attribute on the struct itself for extra fields:
/// ```ignore
/// #[derive(StateFilterConversion)]
/// #[conversion(Age)]
/// struct ExampleStruct {
///     some_value: UnknownUser,
/// }
/// # struct UnknownUser;
/// # struct Age;
/// ```
/// The above code will allow `ExampleStruct` to be deconstructed and
/// then reconstructed into a new struct which also contains the field `age: Age`.
///
/// If you use generics, use this syntax:
/// ```ignore
/// #[derive(StateFilterConversion)]
/// struct ExampleStruct {
///     #[conversion(T = UserWithData<T>)]
///     some_value: UnknownUser,
/// }
/// # struct UnknownUser;
/// # struct UserWithData<T>(T);
/// ```
/// And if you want more than one generic, be sure to use a different generic name:
/// ```ignore
/// #[derive(StateFilterConversion)]
/// #[conversion(T0 = SomeMoreData<T0>)]
/// struct ExampleStruct {
///     #[conversion(T1, T2 = UserWithData<T1, T2>)]
///     some_value: UnknownUser,
/// }
/// # struct UnknownUser;
/// # struct UserWithData<T0, T1>(T0, T1);
/// # struct SomeMoreData<T>(T);
/// ```
///
/// The `conversion` attribute can be used multiple times on a single field for different conversion types:
/// ```
/// #[derive(StateFilterConversion)]
/// struct ExampleStruct {
///     #[conversion(AdminUser)]
///     #[conversion(UserWithData)]
///     some_value: UnknownUser,
/// }
/// # struct UnknownUser;
/// # struct AdminUser;
/// # struct UserWithData;
/// ```
#[proc_macro_derive(StateFilterConversion, attributes(conversion))]
pub fn state_filter_conversion(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    let name = &ast.ident;
    let state_conversions = match &ast.data {
        syn::Data::Struct(s) => {
            let fields_count = s.fields.len();
            let mut state_conversions = Vec::with_capacity(fields_count);
            let (iter, extra_fields_count) = {
                let mut iter: Vec<_> = s
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(i, field)| {
                        let field_name = field.ident.as_ref().expect("expected a named field");
                        let mut all_conversion_fields = Vec::new();
                        all_conversion_fields.push((
                            field_name.clone(),
                            ConversionSort {
                                sort_number: i,
                                ty: ConversionType::Type(field.ty.clone()),
                            },
                            extract_generics_from_type(&field.ty, &ast.generics),
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
                                ConversionType::Type(ty) => {
                                    extract_generics_from_type(ty, &ast.generics)
                                }
                                ConversionType::Generic { generic_ident, .. } => {
                                    parse_quote!(<#(#generic_ident),*>)
                                }
                            };
                            all_conversion_fields.push((
                                field_name.clone(),
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
                let extra_struct_fields: Vec<_> = ast
                    .attrs
                    .into_iter()
                    .filter(|attr| attr.path().is_ident("conversion"))
                    .enumerate()
                    .map(|(i, attr)| {
                        let f = attr
                            .parse_args::<ConversionType>()
                            .expect("expected a conversion type");
                        let (field_name, generics) = match &f {
                            ConversionType::Type(ty) => {
                                let ident = type_to_ident(ty);
                                (
                                    quote::format_ident!("{}", ident.to_string().to_snake_case()),
                                    extract_generics_from_type(ty, &ast.generics),
                                )
                            }
                            ConversionType::Generic { generic_ident, ty } => {
                                let ident = type_to_ident(ty);
                                (
                                    quote::format_ident!("{}", ident.to_string().to_snake_case()),
                                    parse_quote!(<#(#generic_ident),*>),
                                )
                            }
                        };
                        // TODO: for now, the extra fields can be of only 1 type
                        vec![(
                            field_name,
                            ConversionSort {
                                sort_number: i + iter.len(),
                                ty: f,
                            },
                            generics,
                        )]
                    })
                    .collect();
                let extra_fields_count = extra_struct_fields.len();
                iter.extend(extra_struct_fields);
                (iter, extra_fields_count)
            };
            let mut combination_names = HashMap::new();
            let mut remainder_names = HashMap::new();
            let mut i = 0;
            for powerset in iter.iter().powerset() {
                for (field_names, mut field_types, field_generics) in
                    powerset.into_iter().multi_cartesian_product().map(|f| {
                        let mut field_names = Vec::with_capacity(f.len());
                        let mut field_types = Vec::with_capacity(f.len());
                        let mut generics = Vec::with_capacity(f.len());
                        for (field_name, field_type, field_generics) in f {
                            field_names.push(field_name);
                            field_types.push(field_type.clone());
                            generics.push(field_generics);
                        }
                        (field_names, field_types, generics)
                    })
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
                    i += 1;
                }
            }
            let mut i = 0;
            for powerset in iter.iter().powerset() {
                for (field_names, mut field_types, field_generics) in
                    powerset.into_iter().multi_cartesian_product().map(|f| {
                        let mut field_names = Vec::with_capacity(f.len());
                        let mut field_types = Vec::with_capacity(f.len());
                        let mut generics = Vec::with_capacity(f.len());
                        for (field_name, field_type, field_generics) in f {
                            field_names.push(field_name);
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
                &ast.generics,
                &combination_names,
                &remainder_names,
                name,
                &s.fields,
                ast.generics.clone(),
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
                for count in 0..=(fields_count + extra_fields_count) {
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
                                current_field_names.push((*field_name).clone());
                                current_field_types.push((*field_type).clone());
                                current_field_generics.push((*generics).clone());
                            }
                            let mut other_field_names = Vec::with_capacity(remainder.len());
                            let mut other_field_types = Vec::with_capacity(remainder.len());
                            let mut other_field_generics = Vec::with_capacity(remainder.len());
                            for ((field_name, field_type), generics) in remainder {
                                other_field_names.push((*field_name).clone());
                                other_field_types.push((*field_type).clone());
                                other_field_generics.push((*generics).clone());
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
                            let r = current_field_types
                                .iter()
                                .chain(other_field_types.iter())
                                .cloned()
                                .sorted()
                                .collect::<Vec<_>>();
                            let combined_struct_name = combination_names
                                .get(&r)
                                .expect(&format!("0: expected a combined struct: {:?}", r));
                            let remainder_struct_name = {
                                let mut other_field_types = other_field_types.clone();
                                other_field_types.sort();
                                remainder_names.get(&other_field_types).unwrap()
                            };
                            let mut o = Generics::default();
                            for other_field_generics in other_field_generics {
                                o = merge_generics(o, &other_field_generics);
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
        #(#state_conversions)*
    }
    .into()
}

fn create_original_conversion_combinations(
    state_conversions: &mut Vec<proc_macro2::TokenStream>,
    original_generics: &Generics,
    combination_names: &HashMap<Vec<ConversionSort>, Ident>,
    remainder_names: &HashMap<Vec<ConversionSort>, Ident>,
    name: &Ident,
    fields: &syn::Fields,
    mut all_field_generics: Generics,
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
                extract_generics_from_type(&field.ty, original_generics),
            )
        })
        .collect();
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
                    current_field_names.push((*field_name).clone());
                    current_field_types.push(field_type.clone());
                    current_field_generics.push(generics.clone());
                }
                let mut other_field_names = Vec::with_capacity(remainder.len());
                let mut other_field_types = Vec::with_capacity(remainder.len());
                let mut other_field_generics = Vec::new();
                for (field_name, field_type, generics) in remainder {
                    other_field_names.push((*field_name).clone());
                    other_field_types.push(field_type.clone());
                    other_field_generics.push(generics.clone());
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
                let r = current_field_types
                    .iter()
                    .chain(other_field_types.iter())
                    .cloned()
                    .sorted()
                    .collect::<Vec<_>>();
                let combined_struct_name = combination_names.get(&r).expect(&format!(
                    "1: expected a combined struct: {:#?}\nCOMBINATION NAMES: {:#?}",
                    r, combination_names,
                ));
                let remainder_struct_name = {
                    let mut other_field_types = other_field_types.clone();
                    other_field_types.sort();
                    remainder_names
                        .get(&other_field_types)
                        .expect("expected a remainder struct")
                };
                let mut current_field_generic = Generics::default();
                for current_generics in current_field_generics {
                    current_field_generic =
                        merge_generics(current_field_generic, &current_generics);
                }
                let current_field_generics = current_field_generic;
                let mut other_field_generic = Generics::default();
                for other_generics in other_field_generics {
                    other_field_generic = merge_generics(other_field_generic, &other_generics);
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

fn extract_generics_from_type(ty: &Type, original_generics: &Generics) -> Generics {
    let mut type_params = BTreeSet::new();
    let mut lifetime_params = BTreeSet::new();
    let mut const_params = BTreeSet::new();

    collect_generics(
        ty,
        original_generics,
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
    original_generics: &Generics,
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
                                if let Type::Path(p) = inner_ty
                                    && let Some(ident) = p.path.get_ident()
                                    && original_generics.type_params().any(|ty| ty.ident == *ident)
                                {
                                    type_params.insert(ident.clone());
                                }
                                collect_generics(
                                    inner_ty,
                                    original_generics,
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
            collect_generics(
                &r.elem,
                original_generics,
                type_params,
                lifetime_params,
                const_params,
            );
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

fn type_to_ident(ty: &Type) -> &Ident {
    match ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|seg| &seg.ident)
            .unwrap(),
        _ => unimplemented!(),
    }
}
