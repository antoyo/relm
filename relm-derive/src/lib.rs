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

extern crate proc_macro;

mod gen;

use quote::{quote, quote_spanned};
use proc_macro2::TokenStream;
use syn::{
    GenericParam,
    Generics,
    Ident,
    Item,
    LifetimeDef,
    TypeParam,
    parse,
};
use syn::spanned::Spanned;

use gen::{gen_widget, gen_where_clause, parser::dummy_ident};

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

fn derive_display_variant(ast: &Item, krate: &Ident) -> TokenStream {
    if let Item::Enum(ref enum_item) = *ast {
        let generics = &enum_item.generics;
        let name = &enum_item.ident;
        let generics_without_bound = remove_generic_bounds(generics);
        let typ = quote! {
            #name #generics_without_bound
        };

        let variant_patterns = enum_item.variants.iter().map(|variant| {
            let doc_ident = dummy_ident("doc");
            let attrs = variant.attrs.iter().filter(|attr| !attr.path.is_ident(&doc_ident));
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
    generics
}
