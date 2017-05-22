/*
 * TODO: does an attribute #[msg] would simplify the implementation instead of #[derive(Msg)]?
 */

#![recursion_limit="256"]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate relm_gen_widget;
extern crate syn;

use proc_macro::TokenStream;

use quote::Tokens;
use relm_gen_widget::{gen_widget, gen_where_clause};
use syn::{
    Body,
    Generics,
    Ident,
    Item,
    MacroInput,
    Variant,
    VariantData,
    parse_item,
    parse_macro_input,
};
use syn::ItemKind::Struct;
use syn::TokenTree::Delimited;
use syn::Ty::Mac;

#[proc_macro_derive(SimpleMsg)]
pub fn simple_msg(input: TokenStream) -> TokenStream {
    let string = input.to_string();
    let ast = parse_macro_input(&string).unwrap();
    let gen = impl_simple_msg(&ast);
    gen.parse().unwrap()
}

fn impl_simple_msg(ast: &MacroInput) -> Tokens {
    let name = &ast.ident;

    let display = derive_display_variant(ast);
    let into_option = derive_into_option(ast);

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
                self.clone()
            }
        }
    }
}

#[proc_macro_derive(Msg)]
pub fn msg(input: TokenStream) -> TokenStream {
    let string = input.to_string();
    let ast = parse_macro_input(&string).unwrap();
    let gen = impl_msg(&ast);
    gen.parse().unwrap()
}

fn impl_msg(ast: &MacroInput) -> Tokens {
    let display = derive_display_variant(ast);
    let into_option = derive_into_option(ast);

    quote! {
        #display
        #into_option
    }
}

fn derive_into_option(ast: &MacroInput) -> Tokens {
    let generics = &ast.generics;
    let name = &ast.ident;
    let generics_without_bound = remove_generic_bounds(generics);
    let typ = quote! {
        #name #generics_without_bound
    };
    let where_clause = gen_where_clause(&generics);

    quote! {
        impl #generics ::relm::IntoOption<#typ> for #typ #where_clause {
            fn into_option(self) -> Option<#typ> {
                Some(self)
            }
        }
    }
}

fn derive_display_variant(ast: &MacroInput) -> Tokens {
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
            impl #generics ::relm::DisplayVariant for #typ #where_clause {
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

#[proc_macro_derive(Widget)]
pub fn widget(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let ast = parse_item(&source).unwrap();
    let expanded = impl_widget(&ast);
    expanded.parse().unwrap()
}

fn impl_widget(ast: &Item) -> Tokens {
    if let Struct(VariantData::Struct(ref fields), _) = ast.node {
        for field in fields {
            if field.ident == Some(Ident::new("widget")) {
                if let Mac(ref mac) = field.ty {
                    if let Delimited(syn::Delimited { ref tts, .. }) = mac.tts[0] {
                        let mut tokens = Tokens::new();
                        tokens.append_all(tts);
                        return gen_widget(tokens);
                    }
                }
            }
        }
    }

    panic!("Expecting `widget` field.");
}

fn remove_generic_bounds(generics: &Generics) -> Generics {
    let mut generics = generics.clone();
    for param in &mut generics.ty_params {
        param.bounds = vec![];
    }
    generics.clone()
}

fn gen_ignored_idents(count: usize) -> Vec<Ident> {
    (0..count)
        .map(|_| Ident::new("_"))
        .collect()
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
