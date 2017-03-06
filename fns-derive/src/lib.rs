extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;

#[proc_macro_derive(Fns)]
pub fn fns(input: TokenStream) -> TokenStream {
    let string = input.to_string();
    let ast = syn::parse_macro_input(&string).unwrap();
    let gen = impl_fns(&ast);
    gen.parse().unwrap()
}

fn impl_fns(ast: &syn::MacroInput) -> quote::Tokens {
    let name = &ast.ident;

    quote! {
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
