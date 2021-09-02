use proc_macro2::TokenStream;
use quote::quote;

#[proc_macro_derive(JFace, attributes())]
pub fn jface_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let ast = syn::parse(input).unwrap();
  impl_jface(&ast).into()
}

fn impl_jface(ast: &syn::DeriveInput) -> TokenStream {
  let name = &ast.ident;

  let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
  quote! {
    impl #impl_generics JFace for #name #ty_generics #where_clause {}
  }
}
