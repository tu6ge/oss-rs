use file::FileTrait;
use impl_object::impl_object;
use proc_macro::TokenStream;

use quote::quote;
use syn::parse_macro_input;
mod file;
mod impl_object;

#[proc_macro_attribute]
pub fn oss_file(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(input as FileTrait);
    impl_object(&mut item);
    TokenStream::from(quote!(#item))
}

#[cfg(test)]
mod tests{
    #[test]
    fn ui() {
        let t = trybuild::TestCases::new();
        t.pass("tests/file.rs");
    }
}