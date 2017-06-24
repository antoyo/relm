#[macro_use]
extern crate quote;
extern crate relm_gen_widget;
extern crate syn;

use quote::Tokens;
use relm_gen_widget::gen_where_clause;
use syn::{
    Body,
    Generics,
    Ident,
    MacroInput,
    Variant,
    VariantData,
};
use syn::Body::Enum;
use syn::VariantData::Unit;

pub fn impl_msg(ast: &MacroInput, krate: Ident) -> Tokens {
    let display = derive_display_variant(ast, &krate);
    let into_option = derive_into_option(ast, &krate);

    quote! {
        #display
        #into_option
    }
}

pub fn impl_simple_msg(ast: &MacroInput, krate: Ident) -> Tokens {
    let name = &ast.ident;

    let display = derive_display_variant(ast, &krate);
    let into_option = derive_into_option(ast, &krate);
    let match_clone = derive_partial_clone(ast);

    let generics = &ast.generics;
    let generics_without_bound = remove_generic_bounds(generics);
    let typ = quote! {
        #name #generics_without_bound
    };
    let where_clause = gen_where_clause(&generics);

    quote! {
        #display
        #into_option

        impl #generics FnOnce<((),)> for #typ #where_clause {
            type Output = #typ;

            extern "rust-call" fn call_once(self, args: ((),)) -> Self::Output {
                self.call(args)
            }
        }

        impl #generics FnMut<((),)> for #typ #where_clause {
            extern "rust-call" fn call_mut(&mut self, args: ((),)) -> Self::Output {
                self.call(args)
            }
        }

        impl #generics Fn<((),)> for #typ #where_clause {
            extern "rust-call" fn call(&self, _: ((),)) -> Self::Output {
                #match_clone
            }
        }
    }
}

fn derive_partial_clone(ast: &MacroInput) -> Tokens {
    if let Enum(ref variants) = ast.body {
        let name = &ast.ident;
        let mut patterns = vec![];
        let mut values = vec![];
        for variant in variants {
            if variant.data == Unit {
                let ident = &variant.ident;
                patterns.push(quote! {
                    #name::#ident
                });
                values.push(&variant.ident);
            }
        }
        quote! {
            #[allow(unreachable_patterns)]
            match *self {
                #(#patterns => #values,)*
                _ => panic!("Expected a variant without parameter"),
            }
        }
    }
    else {
        panic!("Expected enum but found {:?}", ast.body);
    }
}

fn derive_display_variant(ast: &MacroInput, krate: &Ident) -> Tokens {
    let generics = &ast.generics;
    let name = &ast.ident;
    let generics_without_bound = remove_generic_bounds(generics);
    let typ = quote! {
        #name #generics_without_bound
    };

    if let Body::Enum(ref variants) = ast.body {
        let variant_idents_values = gen_idents_count(variants);
        let variant_patterns = variant_idents_values.iter().map(|&(ref ident, value_count)| {
            let value_idents = gen_ignored_idents(value_count);
            if value_count > 0 {
                quote! {
                    #name::#ident(#(#value_idents),*)
                }
            }
            else {
                quote! {
                    #name::#ident
                }
            }
        });
        let variant_names = variant_idents_values.iter().map(|&(ref ident, _)| {
            ident.to_string()
        });
        let where_clause = gen_where_clause(generics);

        quote! {
            impl #generics ::#krate::DisplayVariant for #typ #where_clause {
                #[allow(unused_qualifications)]
                fn display_variant(&self) -> &'static str {
                    match *self {
                        #(#variant_patterns => #variant_names,)*
                    }
                }
            }
        }
    }
    else {
        panic!("Expected enum");
    }
}

fn derive_into_option(ast: &MacroInput, krate: &Ident) -> Tokens {
    let generics = &ast.generics;
    let name = &ast.ident;
    let generics_without_bound = remove_generic_bounds(generics);
    let typ = quote! {
        #name #generics_without_bound
    };
    let where_clause = gen_where_clause(&generics);

    quote! {
        impl #generics ::#krate::IntoOption<#typ> for #typ #where_clause {
            fn into_option(self) -> Option<#typ> {
                Some(self)
            }
        }
    }
}

fn gen_idents_count(variants: &[Variant]) -> Vec<(&Ident, usize)> {
    variants.iter().map(|variant| {
        let value_count =
            if let VariantData::Tuple(ref tuple) = variant.data {
                tuple.len()
            }
            else {
                0
            };
        (&variant.ident, value_count)
    }).collect()
}

fn gen_ignored_idents(count: usize) -> Vec<Ident> {
    (0..count)
        .map(|_| Ident::new("_"))
        .collect()
}

fn remove_generic_bounds(generics: &Generics) -> Generics {
    let mut generics = generics.clone();
    for param in &mut generics.ty_params {
        param.bounds = vec![];
    }
    generics.clone()
}
