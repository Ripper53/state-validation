use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::TokenStreamExt;
use syn::{Expr, Ident, Type, parse_macro_input};

#[derive(darling::FromDeriveInput)]
#[darling(attributes(state_filter_input))]
struct StateFilterInputData {
    remainder_type: Option<Type>,
    remainder: Option<Expr>,
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

#[proc_macro_derive(StateFilterInput, attributes(state_filter_input, conversion))]
pub fn state_filter_input(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let generics: Vec<_> = ast
        .generics
        .type_params()
        .into_iter()
        .map(|ty| ty.ident.clone())
        .collect();
    let data = StateFilterInputData::from_derive_input(&ast).unwrap();
    let state_conversions = match &ast.data {
        syn::Data::Struct(s) => {
            let fields_count = s.fields.len();
            let mut state_conversions = Vec::with_capacity(fields_count);
            let iter: Vec<_> = s
                .fields
                .iter()
                .filter_map(|field| {
                    /*if !field.attrs.iter().any(|f| f.path().is_ident("conversion")) {
                        return None;
                    }*/
                    let field_name = field.ident.as_ref().expect("expected a named field");
                    let mut all_conversion_fields = Vec::new();
                    all_conversion_fields.push(ConversionType::Type(field.ty.clone()));
                    for attr in field
                        .attrs
                        .iter()
                        .filter(|attr| attr.path().is_ident("conversion"))
                    {
                        let f = attr
                            .parse_args::<ConversionType>()
                            .expect("expected a conversion type");
                        all_conversion_fields.push(f);
                    }
                    Some((field_name, all_conversion_fields))
                })
                .collect();
            for (i, m) in FieldCombinationIter::new(iter).enumerate() {
                let mut a = |combined_struct_name: &Ident,
                             remainder_struct_name: &Ident,
                             Meow {
                                 current_field: (field_name, field_type, field_generics),
                                 other_fields:
                                     (other_field_names, other_field_types, other_field_generics),
                             }: &Meow<'_>| {
                    let q = quote::quote! {
                        pub struct #combined_struct_name <#(#field_generics),* #(#(#other_field_generics),*)*> {
                            #field_name: #field_type,
                            #(#other_field_names: #other_field_types),*
                        }
                        pub struct #remainder_struct_name <#(#(#other_field_generics),*)*> {
                            #(#other_field_names: #other_field_types),*
                        }
                        impl <#(#field_generics),* #(#(#other_field_generics),*)*> state_validation::StateFilterInputCombination<#field_type> for #remainder_struct_name <#(#(#other_field_generics),*)*> {
                            type Combined = #combined_struct_name <#(#field_generics),* #(#(#other_field_generics),*)*>;
                            fn combine(self, value: #field_type) -> Self::Combined {
                                #combined_struct_name {
                                    #field_name: value,
                                    #(#other_field_names: self.#other_field_names),*
                                }
                            }
                        }
                        impl <#(#field_generics),* #(#(#other_field_generics),*)*> state_validation::StateFilterInputConversion<#field_type> for #combined_struct_name <#(#field_generics),* #(#(#other_field_generics),*)*> {
                            type Remainder = #remainder_struct_name <#(#(#other_field_generics),*)*>;
                            fn split_take(self) -> (#field_type, Self::Remainder) {
                                (
                                    self.#field_name,
                                    #remainder_struct_name {
                                        #(#other_field_names: self.#other_field_names),*
                                    },
                                )
                            }
                        }
                    };
                    state_conversions.push(q);
                };
                match m {
                    M::Next { field_number, meow } => {
                        let combined_struct_name =
                            quote::format_ident!("__StateValidationGeneration_{name}Combined_{i}");
                        let remainder_struct_name =
                            quote::format_ident!("__StateValidationGeneration_{name}Remainder_{i}");
                        a(&combined_struct_name, &remainder_struct_name, &meow);
                    }
                    M::NewType {
                        field_number,
                        type_number,
                        meow,
                    } => {
                        let combined_struct_name =
                            quote::format_ident!("__StateValidationGeneration_{name}Combined_{i}");
                        let remainder_struct_name =
                            quote::format_ident!("__StateValidationGeneration_{name}Remainder_{i}");
                        a(&combined_struct_name, &remainder_struct_name, &meow);
                        let (field_name, field_type, field_generics) = meow.current_field;
                        let (other_field_names, other_field_types, other_field_generics) =
                            meow.other_fields;
                        let remainder_struct_name = quote::format_ident!(
                            "__StateValidationGeneration_{name}RemainderField_{field_number}"
                        );
                        let q = quote::quote! {
                            impl <#(#field_generics),* #(#(#other_field_generics),*)*> state_validation::StateFilterInputCombination<#field_type> for #remainder_struct_name <#(#(#other_field_generics),*)*> {
                                type Combined = #combined_struct_name <#(#field_generics),* #(#(#other_field_generics),*)*>;
                                fn combine(self, value: #field_type) -> Self::Combined {
                                    #combined_struct_name {
                                        #field_name: value,
                                        #(#other_field_names: self.#other_field_names),*
                                    }
                                }
                            }
                        };
                        state_conversions.push(q);
                    }
                    M::NewField { field_number, meow } => {
                        let combined_struct_name = quote::format_ident!(
                            "__StateValidationGeneration_{name}CombinedField_{field_number}"
                        );
                        let remainder_struct_name = quote::format_ident!(
                            "__StateValidationGeneration_{name}RemainderField_{field_number}"
                        );
                        a(&combined_struct_name, &remainder_struct_name, &meow);
                        let (field_name, field_type, field_generics) = meow.current_field;
                        let (other_field_names, other_field_types, other_field_generics) =
                            meow.other_fields;
                        let q = quote::quote! {
                            impl <#(#generics),* #(#field_generics),*> state_validation::StateFilterInputConversion<#field_type> for #name #ty_generics #where_clause {
                                type Remainder = #remainder_struct_name;
                                fn split_take(self) -> (#field_type, Self::Remainder) {
                                    (
                                        self.#field_name,
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
            state_conversions
        }
        _ => todo!(),
    };
    quote::quote! {
        impl #impl_generics state_validation::StateFilterInput for #name #ty_generics #where_clause {}
        #(#state_conversions)*
    }.into()
}

struct FieldCombinationIter<'a> {
    current_field: Option<CurrentFieldCursor<'a>>,
    current_other_fields: Vec<CurrentFieldCursor<'a>>,
    current_iteration: usize,
    max_iteration: usize,
    first_iteration: bool,
    type_iteration: usize,
}
impl<'a> FieldCombinationIter<'a> {
    fn new(mut fields: Vec<(&'a Ident, Vec<ConversionType>)>) -> Self {
        let max_iteration = fields.len();
        let (current_field, current_field_types) =
            fields.pop().expect("expected at least one field");
        let current_other_fields = fields
            .into_iter()
            .map(|(field_name, conversion_types)| {
                CurrentFieldCursor::new(field_name, conversion_types)
            })
            .collect();
        FieldCombinationIter {
            current_field: Some(CurrentFieldCursor::new(current_field, current_field_types)),
            current_other_fields,
            current_iteration: 0,
            max_iteration,
            first_iteration: true,
            type_iteration: 0,
        }
    }
}
struct CurrentFieldCursor<'a> {
    field_name: &'a Ident,
    last_current_type: ConversionType,
    iter: std::vec::IntoIter<ConversionType>,
    original_vec: Vec<ConversionType>,
}
impl<'a> CurrentFieldCursor<'a> {
    fn new(field_name: &'a Ident, original_vec: Vec<ConversionType>) -> Self {
        let mut iter = original_vec.clone().into_iter();
        let current_type = iter.next().expect("expected at least one element");
        CurrentFieldCursor {
            field_name,
            last_current_type: current_type,
            iter,
            original_vec,
        }
    }
    fn reset(&mut self) {
        self.iter = self.original_vec.clone().into_iter();
        self.last_current_type = self.iter.next().unwrap();
    }
}
impl<'a> Iterator for CurrentFieldCursor<'a> {
    type Item = ();
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current_type) = self.iter.next() {
            self.last_current_type = current_type;
            Some(())
        } else {
            None
        }
    }
}
enum M<'a> {
    NewField {
        field_number: usize,
        meow: Meow<'a>,
    },
    NewType {
        field_number: usize,
        type_number: usize,
        meow: Meow<'a>,
    },
    Next {
        field_number: usize,
        meow: Meow<'a>,
    },
}
struct Meow<'a> {
    // (field_names, field_types, TODO: field_generics)
    current_field: (&'a Ident, ConversionType, Vec<Ident>),
    other_fields: (Vec<&'a Ident>, Vec<ConversionType>, Vec<Vec<Ident>>),
}
impl<'a> Iterator for FieldCombinationIter<'a> {
    type Item = M<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        enum N {
            Next,
            NewType,
            NewField,
        }
        let mut n = if self.first_iteration {
            N::NewField
        } else {
            N::Next
        };
        loop {
            if let Some(ref mut current_cursor) = self.current_field {
                if self.first_iteration
                    || self
                        .current_other_fields
                        .iter_mut()
                        .any(|other_cursor| other_cursor.next().is_some())
                {
                    self.first_iteration = false;
                    let current_field_generics = if let ConversionType::Generic {
                        generic_ident,
                        path,
                    } = &current_cursor.last_current_type
                    {
                        generic_ident.clone()
                    } else {
                        Vec::new()
                    };
                    let (other_field_names, (other_field_types, other_field_generics)): (
                        Vec<_>,
                        (Vec<_>, Vec<_>),
                    ) = self
                        .current_other_fields
                        .iter()
                        .map(|other_cursor| {
                            let other_generics = if let ConversionType::Generic {
                                generic_ident,
                                path,
                            } = &other_cursor.last_current_type
                            {
                                generic_ident.clone()
                            } else {
                                Vec::new()
                            };
                            (
                                other_cursor.field_name,
                                (other_cursor.last_current_type.clone(), other_generics),
                            )
                        })
                        .unzip();
                    let m = match n {
                        N::Next => M::Next {
                            field_number: self.current_iteration,
                            meow: Meow {
                                current_field: (
                                    current_cursor.field_name,
                                    current_cursor.last_current_type.clone(),
                                    current_field_generics,
                                ),
                                other_fields: (
                                    other_field_names,
                                    other_field_types,
                                    other_field_generics,
                                ),
                            },
                        },
                        N::NewType => M::NewType {
                            field_number: self.current_iteration,
                            type_number: self.type_iteration,
                            meow: Meow {
                                current_field: (
                                    current_cursor.field_name,
                                    current_cursor.last_current_type.clone(),
                                    current_field_generics,
                                ),
                                other_fields: (
                                    other_field_names,
                                    other_field_types,
                                    other_field_generics,
                                ),
                            },
                        },
                        N::NewField => M::NewField {
                            field_number: self.current_iteration,
                            meow: Meow {
                                current_field: (
                                    current_cursor.field_name,
                                    current_cursor.last_current_type.clone(),
                                    current_field_generics,
                                ),
                                other_fields: (
                                    other_field_names,
                                    other_field_types,
                                    other_field_generics,
                                ),
                            },
                        },
                    };
                    break Some(m);
                } else if current_cursor.next().is_some() {
                    n = N::NewType;
                    self.type_iteration += 1;
                    self.first_iteration = true;
                    for other_cursor in self.current_other_fields.iter_mut() {
                        other_cursor.reset();
                    }
                } else {
                    n = N::NewField;
                    self.current_iteration += 1;
                    self.type_iteration = 0;
                    if self.current_iteration < self.max_iteration
                        && let Some(mut new_current_field) = self.current_other_fields.pop()
                    {
                        self.first_iteration = true;
                        self.current_other_fields
                            .insert(0, self.current_field.take().unwrap());
                        new_current_field.reset();
                        self.current_field = Some(new_current_field);
                        for other_field_cursor in self.current_other_fields.iter_mut() {
                            other_field_cursor.reset();
                        }
                    } else {
                        self.current_field = None;
                    }
                }
            } else {
                break None;
            }
        }
    }
}
