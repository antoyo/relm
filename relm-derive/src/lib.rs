extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;

use syn::{Body, MacroInput, VariantData, parse_macro_input};

#[proc_macro_derive(SimpleMsg)]
pub fn simple_msg(input: TokenStream) -> TokenStream {
    let string = input.to_string();
    let ast = parse_macro_input(&string).unwrap();
    let gen = impl_simple_msg(&ast);
    gen.parse().unwrap()
}

fn impl_simple_msg(ast: &MacroInput) -> quote::Tokens {
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

fn impl_msg(ast: &MacroInput) -> quote::Tokens {
    let clone = derive_clone(ast);
    let display = derive_display_variant(ast);

    quote! {
        #clone
        #display
    }
}

fn derive_clone(ast: &MacroInput) -> quote::Tokens {
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
                    #ident(ref value)
                }
            }
            else {
                quote! {
                    #ident
                }
            }
        });
        let variant_values = variant_idents_values.iter().map(|&(ref ident, has_value)| {
            if has_value {
                quote! {
                    #ident(value.clone())
                }
            }
            else {
                quote! {
                    #ident
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

fn derive_display_variant(ast: &MacroInput) -> quote::Tokens {
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
                    #ident(_)
                }
            }
            else {
                quote! {
                    #ident
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
