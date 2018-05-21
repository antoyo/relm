extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate relm_gen_widget;
extern crate syn;

use proc_macro2::TokenStream;
use relm_gen_widget::gen_where_clause;
use syn::{
    Fields,
    GenericParam,
    Generics,
    Ident,
    Item,
    LifetimeDef,
    TypeParam,
};

pub fn impl_msg(ast: &Item, krate: Ident) -> TokenStream {
    let display = derive_display_variant(ast, &krate);
    let into_option = derive_into_option(ast, &krate);

    quote! {
        #display
        #into_option
    }
}

pub fn impl_simple_msg(ast: &Item, krate: Ident) -> TokenStream {
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
        let where_clause = gen_where_clause(&generics);

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
