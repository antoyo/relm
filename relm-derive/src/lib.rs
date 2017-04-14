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
use relm_gen_widget::gen_widget;
use syn::{Body, Ident, Item, MacroInput, Path, VariantData, parse_item, parse_items, parse_macro_input};
use syn::FnArg::Captured;
use syn::ItemKind::{Fn, Struct};
use syn::TokenTree::Delimited;
use syn::Ty::{self, Mac};

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

    quote! {
        #clone
        #display

        impl FnOnce<((),)> for #name {
            type Output = #name;

            extern "rust-call" fn call_once(self, args: ((),)) -> Self::Output {
                self.call(args)
            }
        }

        impl FnMut<((),)> for #name {
            extern "rust-call" fn call_mut(&mut self, args: ((),)) -> Self::Output {
                self.call(args)
            }
        }

        impl Fn<((),)> for #name {
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

    quote! {
        #clone
        #display
    }
}

fn derive_clone(ast: &MacroInput) -> Tokens {
    let name = &ast.ident;

    if let Body::Enum(ref variants) = ast.body {
        let variant_idents_values: Vec<_> = variants.iter().map(|variant| {
            let has_value =
                if let VariantData::Tuple(_) = variant.data {
                    true
                }
                else {
                    false
                };
            (&variant.ident, has_value)
        }).collect();
        let variant_patterns = variant_idents_values.iter().map(|&(ref ident, has_value)| {
            if has_value {
                quote! {
                    #name::#ident(ref value)
                }
            }
            else {
                quote! {
                    #name::#ident
                }
            }
        });
        let variant_values = variant_idents_values.iter().map(|&(ref ident, has_value)| {
            if has_value {
                quote! {
                    #name::#ident(value.clone())
                }
            }
            else {
                quote! {
                    #name::#ident
                }
            }
        });

        quote! {
            impl Clone for #name {
                fn clone(&self) -> Self {
                    match *self {
                        #(#variant_patterns => #variant_values,)*
                    }
                }
            }
        }
    }
    else {
        panic!("Expected enum");
    }
}

fn derive_display_variant(ast: &MacroInput) -> Tokens {
    let name = &ast.ident;

    if let Body::Enum(ref variants) = ast.body {
        let variant_idents_values: Vec<_> = variants.iter().map(|variant| {
            let has_value =
                if let VariantData::Tuple(_) = variant.data {
                    true
                }
                else {
                    false
                };
            (&variant.ident, has_value)
        }).collect();
        let variant_patterns = variant_idents_values.iter().map(|&(ref ident, has_value)| {
            if has_value {
                quote! {
                    #name::#ident(_)
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

        quote! {
            impl ::relm::DisplayVariant for #name {
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
    let name = Ident::new(format!("{}Widgets", &ast.ident));

    if let Struct(VariantData::Struct(ref fields), _) = ast.node {
        for field in fields {
            if field.ident == Some(Ident::new("widget")) {
                if let Mac(ref mac) = field.ty {
                    if let Delimited(syn::Delimited { ref tts, .. }) = mac.tts[0] {
                        let mut tokens = Tokens::new();
                        tokens.append_all(tts);
                        let msg_type = get_msg_type(&tokens);
                        let widget_impl = quote! {
                            impl ::relm::Widget<#msg_type> for #name {
                                #tokens
                            }
                        };
                        return gen_widget(widget_impl);
                    }
                }
            }
        }
    }

    panic!("Expecting `widget` field.");
}

fn get_msg_type(tokens: &Tokens) -> Path {
    let ast = parse_items(&tokens.to_string()).unwrap();
    for item in ast {
        if item.ident == Ident::new("update") {
            if let Fn(ref func, _, _, _, _, _) = item.node {
                if let Captured(_, Ty::Path(_, ref path)) = func.inputs[1] {
                    return path.clone();
                }
            }
        }
    }

    panic!("Expected `update` function with 3 parameters");
}
