use attr::Attribute;
use file::FileTrait;
use impl_object::impl_object;
use proc_macro::TokenStream;

use quote::quote;
use syn::parse_macro_input;
mod file;
mod impl_object;
mod attr;

#[proc_macro_attribute]
pub fn oss_file(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as Attribute);
    let mut item = parse_macro_input!(input as FileTrait);
    impl_object(&mut item, attr.send);
    TokenStream::from(quote!(#item))
}

#[cfg(test)]
mod tests {
    #[test]
    fn ui() {
        let t = trybuild::TestCases::new();
        t.pass("tests/file.rs");
    }
}
