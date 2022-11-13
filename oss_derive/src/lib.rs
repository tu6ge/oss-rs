use proc_macro::TokenStream;
use quote::quote;
use syn;

/**
 * 

**/

#[proc_macro_derive(File)]
pub fn file_derive(input: TokenStream) -> TokenStream{
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_file_macro(&ast)
}

fn impl_file_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        use crate::file::File;
        impl #name<ArcPointer> {

            /// 在 OSS 上删除该文件
            pub async fn delete<F>(&self, filer: &F) -> OssResult<()>
            where
                F: File,
            {
                let ref key = self.path_string();

                filer.delete_object(key).await
            }
        }
    };
    gen.into()
}

