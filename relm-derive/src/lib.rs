/*
 * Copyright (c) 2017-2019 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

/*
 * TODO: does an attribute #[msg] would simplify the implementation instead of #[derive(Msg)]?
 */

#![recursion_limit="256"]

#[macro_use]
extern crate lazy_static;
extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

mod gen;

use proc_macro2::TokenStream;
use syn::{
    Fields,
    GenericParam,
    Generics,
    Ident,
    Item,
    LifetimeDef,
    TypeParam,
    parse,
};
use syn::spanned::Spanned;

use gen::{gen_widget, gen_where_clause};

#[proc_macro_derive(SimpleMsg)]
pub fn simple_msg(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: Item = parse(input).expect("simple_msg > parse failed");
    let gen = impl_simple_msg(&ast, Ident::new("relm", ast.span()));
    gen.into()
}

#[proc_macro_derive(Msg)]
pub fn msg(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: Item = parse(input).expect("msg > parse failed");
    let gen = impl_msg(&ast, Ident::new("relm", ast.span()));
    gen.into()
}

#[proc_macro_attribute]
pub fn widget(_attributes: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: Item = parse(input).expect("widget.parse failed");
    let tokens = quote! {
        #ast
    };
    let expanded = gen_widget(tokens);
    expanded.into()
}

fn impl_msg(ast: &Item, krate: Ident) -> TokenStream {
    let display = derive_display_variant(ast, &krate);
    let into_option = derive_into_option(ast, &krate);

    quote! {
        #display
        #into_option
    }
}

fn impl_simple_msg(ast: &Item, krate: Ident) -> TokenStream {
    if let Item::Enum(ref enum_item) = *ast {
        let name = &enum_item.ident;

        let display = derive_display_variant(ast, &krate);
        let into_option = derive_into_option(ast, &krate);
        let match_clone = derive_partial_clone(ast);

        let generics = &enum_item.generics;
        let generics_without_bound = remove_generic_bounds(generics);
        let typ = quote! {
            #name #generics_without_bound
        };
        let where_clause = gen_where_clause(generics);

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
    else {
        panic!("expected enum");
    }
}

fn derive_partial_clone(ast: &Item) -> TokenStream {
    if let Item::Enum(ref enum_item) = *ast {
        let name = &enum_item.ident;
        let mut patterns = vec![];
        let mut values = vec![];
        for variant in &enum_item.variants {
            if variant.fields == Fields::Unit {
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
        panic!("Expected enum but found {:?}", ast);
    }
}

fn derive_display_variant(ast: &Item, krate: &Ident) -> TokenStream {
    if let Item::Enum(ref enum_item) = *ast {
        let generics = &enum_item.generics;
        let name = &enum_item.ident;
        let generics_without_bound = remove_generic_bounds(generics);
        let typ = quote! {
            #name #generics_without_bound
        };

        let variant_patterns = enum_item.variants.iter().map(|variant| {
            let attrs = &variant.attrs;
            let ident = &variant.ident;
            quote! {
                #(#attrs)* #name::#ident { .. }
            }
        });
        let variant_names = enum_item.variants.iter().map(|variant| {
            variant.ident.to_string()
        });
        let where_clause = gen_where_clause(generics);

        quote_spanned! { krate.span() =>
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

fn derive_into_option(ast: &Item, krate: &Ident) -> TokenStream {
    if let Item::Enum(ref enum_item) = *ast {
        let generics = &enum_item.generics;
        let name = &enum_item.ident;
        let generics_without_bound = remove_generic_bounds(generics);
        let typ = quote! {
            #name #generics_without_bound
        };
        let where_clause = gen_where_clause(generics);

        quote_spanned! { krate.span() =>
            impl #generics ::#krate::IntoOption<#typ> for #typ #where_clause {
                fn into_option(self) -> Option<#typ> {
                    Some(self)
                }
            }
        }
    }
    else {
        panic!("Expecting enum");
    }
}

fn remove_generic_bounds(generics: &Generics) -> Generics {
    let mut generics = generics.clone();
    for param in generics.params.iter_mut() {
        match *param {
            GenericParam::Lifetime(LifetimeDef { ref mut bounds, .. }) =>
                while bounds.pop().is_some() {
                },
            GenericParam::Type(TypeParam { ref mut bounds, .. }) =>
                while bounds.pop().is_some() {
                },
            _ => (),
        }
    }
    generics.clone()
}
