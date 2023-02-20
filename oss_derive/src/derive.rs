use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn impl_custom_item_error(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl aliyun_oss_client::decode::CustomItemError for #name {}
    };
    gen.into()
}

pub(crate) fn impl_custom_list_error(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl aliyun_oss_client::decode::CustomListError for #name {}
    };
    gen.into()
}
