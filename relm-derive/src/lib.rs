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
    let generics = &ast.generics;
    let name = &ast.ident;
    let typ = quote! {
        #name #generics
    };

    match ast.body {
        Body::Enum(ref variants) => derive_clone_enum(name, typ, ast.generics.clone(), variants),
        Body::Struct(VariantData::Struct(ref fields)) => derive_clone_struct(name, typ, &ast.generics, fields),
        _ => panic!("Expected enum or struct"),
    }
}

fn derive_clone_enum(name: &Ident, typ: Tokens, mut generics: Generics, variants: &[Variant]) -> Tokens {
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

    quote! {
        impl #generics Clone for #typ {
            fn clone(&self) -> Self {
                match *self {
                    #(#variant_patterns => #variant_values,)*
                }
            }
        }
    }
}

fn derive_clone_struct(name: &Ident, typ: Tokens, generics: &Generics, fields: &[Field]) -> Tokens {
    let idents: Vec<_> = fields.iter().map(|field| field.ident.clone().unwrap()).collect();
    let idents1 = &idents;
    let idents2 = &idents;
    quote! {
        impl #generics Clone for #typ {
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
    let typ = quote! {
        #name #generics
    };

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
            impl #generics ::relm::DisplayVariant for #typ {
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
