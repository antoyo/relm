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
    Field,
    Generics,
    Ident,
    Item,
    MacroInput,
    PolyTraitRef,
    TraitBoundModifier,
    Variant,
    VariantData,
    parse_item,
    parse_macro_input,
    parse_path,
};
use syn::ItemKind::Struct;
use syn::TokenTree::Delimited;
use syn::Ty::Mac;
use syn::TyParamBound::Trait;

#[proc_macro_derive(SimpleMsg)]
pub fn simple_msg(input: TokenStream) -> TokenStream {
    let string = input.to_string();
    let ast = parse_macro_input(&string).unwrap();
    let gen = impl_simple_msg(&ast);
    gen.parse().unwrap()
}

fn impl_simple_msg(ast: &MacroInput) -> Tokens {
    let name = &ast.ident;

    let clone = derive_clone(ast);
    let display = derive_display_variant(ast);
    let into_option = derive_into_option(ast);

    let generics = &ast.generics;
    let generics_without_bound = remove_generic_bounds(generics);
    let typ = quote! {
        #name #generics_without_bound
    };
    let where_clause = gen_where_clause(&generics);

    quote! {
        #clone
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
    let clone = derive_clone(ast);
    let display = derive_display_variant(ast);
    let into_option = derive_into_option(ast);

    quote! {
        #clone
        #display
        #into_option
    }
}

fn derive_clone(ast: &MacroInput) -> Tokens {
    let generics = &ast.generics;
    let name = &ast.ident;
    let generics_without_bound = remove_generic_bounds(generics);
    let typ = quote! {
        #name #generics_without_bound
    };

    match ast.body {
        Body::Enum(ref variants) => derive_clone_enum(name, typ, ast.generics.clone(), variants),
        Body::Struct(VariantData::Struct(ref fields)) => derive_clone_struct(name, typ, &ast.generics, fields),
        _ => panic!("Expected enum or struct"),
    }
}

fn derive_clone_enum(name: &Ident, typ: Tokens, mut generics: Generics, variants: &[Variant]) -> Tokens {
    let variant_idents_values = gen_idents_count(variants);
    let variant_patterns = variant_idents_values.iter().map(|&(ref ident, value_count)| {
        if value_count > 0 {
            let value_idents = gen_idents(value_count);
            quote! {
                #name::#ident(#(ref #value_idents),*)
            }
        }
        else {
            quote! {
                #name::#ident
            }
        }
    });
    let variant_values = variant_idents_values.iter().map(|&(ref ident, value_count)| {
        if value_count > 0 {
            let value_idents = gen_idents(value_count);
            quote! {
                #name::#ident(#(#value_idents.clone()),*)
            }
        }
        else {
            quote! {
                #name::#ident
            }
        }
    });

    let path = quote! {
        Clone
    };
    let path = parse_path(path.as_str()).expect("Clone is a path");
    if let Some(param) = generics.ty_params.get_mut(0) {
        param.bounds = vec![
            Trait(PolyTraitRef {
                bound_lifetimes: vec![],
                trait_ref: path,
            }, TraitBoundModifier::None)
        ];
    }
    let where_clause = gen_where_clause(&generics);

    quote! {
        impl #generics Clone for #typ #where_clause {
            fn clone(&self) -> Self {
                match *self {
                    #(#variant_patterns => #variant_values,)*
                }
            }
        }
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

fn derive_clone_struct(name: &Ident, typ: Tokens, generics: &Generics, fields: &[Field]) -> Tokens {
    let idents: Vec<_> = fields.iter().map(|field| field.ident.clone().unwrap()).collect();
    let idents1 = &idents;
    let idents2 = &idents;
    let where_clause = gen_where_clause(generics);
    quote! {
        impl #generics Clone for #typ #where_clause {
            fn clone(&self) -> Self {
                #name {
                    #(#idents1: self.#idents2.clone(),)*
                }
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
        .map(|count| Ident::new("_"))
        .collect()
}

fn gen_idents(count: usize) -> Vec<Ident> {
    (0..count)
        .map(|count| Ident::new(format!("value{}", count)))
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
