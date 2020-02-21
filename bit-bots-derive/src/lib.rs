extern crate proc_macro;
use crate::proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(WritableTemplate)]
pub fn writable_template_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    // get the name of the type we want to implement the trait for
    let name = &input.ident;

    let expanded = quote! {
      impl crate::project::templates::WritableTemplate for #name {
        fn write(&self, dest: &::std::path::Path) -> ::std::io::Result<()> {
          let mut file = ::std::io::BufWriter::new(::std::fs::File::create(dest)?);
          file.write(self.render().unwrap().as_bytes())?;

          Ok(())
        }
      }
    };

    TokenStream::from(expanded)
}
