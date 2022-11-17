use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::visit::{self, Visit};
use syn::FnArg;
use syn::Generics;
use syn::Pat;
use syn::WhereClause;
use syn::{
    parse::{Parse, ParseStream, Result},
    ItemTrait, Signature,
};

pub struct FileTrait {
    pub(crate) input: ItemTrait,
    pub(crate) methods: Vec<Signature>,
    pub(crate) async_methods: Vec<Signature>,
}

impl FileTrait {
    pub fn get_inputs(inputs: &Punctuated<FnArg, Comma>) -> Vec<TokenStream> {
        inputs
            .iter()
            .filter(|arg| match arg {
                FnArg::Receiver(_) => true,
                FnArg::Typed(pattype) => match &*pattype.pat {
                    Pat::Ident(i) => {
                        let mut a = IdentWrapper::default();
                        a.visit_ident(&i.ident);
                        if a.is_key() {
                            false
                        } else {
                            true
                        }
                    }
                    _ => true,
                },
            })
            .map(|f| f.to_token_stream())
            .collect()
    }

    pub fn get_method_arg(inputs: &Punctuated<FnArg, Comma>) -> Vec<TokenStream> {
        inputs
            .iter()
            .filter(|arg| match arg {
                FnArg::Receiver(_) => false,
                FnArg::Typed(_) => true,
            })
            .map(|arg| match arg {
                FnArg::Receiver(_) => {
                    unreachable!("不会有这种情况");
                }
                FnArg::Typed(pattype) => match &*pattype.pat {
                    Pat::Ident(i) => i.ident.to_token_stream(),
                    _ => {
                        panic!("没有考虑这种情况");
                    }
                },
            })
            .collect()
    }

    pub fn params_where(generics: &Generics) -> (TokenStream, TokenStream) {
        let final_params = if generics.params.is_empty() {
            quote! { Ft }
        } else {
            let token = generics.params.to_token_stream();
            quote! { #token , Ft}
        };

        let where_clause = Self::get_where_clause(&generics.where_clause);

        (final_params, where_clause)
    }

    #[inline]
    fn get_where_clause(where_clause: &Option<WhereClause>) -> TokenStream {
        match where_clause {
            Some(e) => {
                let predicates = e.predicates.to_token_stream();
                quote! { where #predicates Ft: File  }
            }
            None => {
                quote! { where Ft: File  }
            }
        }
    }

    fn methods_to_tokens(&self, tokens: &mut TokenStream) {
        if self.methods.len() == 0 {
            return;
        }

        let mut list = Vec::with_capacity(self.methods.len());
        for method in &self.methods {
            let ref output = method.output;
            let ref method_name = method.ident;
            if method_name.to_string() == "get_url".to_string() {
                continue;
            }

            let inputs = FileTrait::get_inputs(&method.inputs);
            let method_arg: Vec<TokenStream> = FileTrait::get_method_arg(&method.inputs);
            let (final_params, where_clause) = FileTrait::params_where(&method.generics);

            let inputs_str = quote! { #(#inputs,)* };
            let method_args_str = quote! { #(#method_arg,)* };
            let filer = quote! { filer: &Ft };

            list.push(quote! {
                pub fn #method_name < #final_params >(#inputs_str #filer ) #output #where_clause  {
                    let ref key = self.base.path().to_string();
                    filer. #method_name ( #method_args_str )
                }
            });
        }

        let res = quote! {
            use crate::object::Object;
            impl Object<RcPointer> {
                #(#list)*
            }
        };
        res.to_tokens(tokens);
    }

    fn async_methods_to_tokens(&self, tokens: &mut TokenStream) {
        if self.async_methods.len() == 0 {
            return;
        }

        let mut list = Vec::with_capacity(self.async_methods.len());
        for method in &self.async_methods {
            let ref output = method.output;
            let ref method_name = method.ident;
            if method_name.to_string() == "get_url".to_string() {
                continue;
            }

            let inputs = FileTrait::get_inputs(&method.inputs);
            let method_arg: Vec<TokenStream> = FileTrait::get_method_arg(&method.inputs);
            let (final_params, where_clause) = FileTrait::params_where(&method.generics);

            let inputs_str = quote! { #(#inputs,)* };
            let method_args_str = quote! { #(#method_arg,)* };
            let filer = quote! { filer: &Ft };

            list.push(quote! {
                pub fn async #method_name < #final_params >(#inputs_str #filer ) #output #where_clause  {
                    let ref key = self.base.path().to_string();
                    filer. #method_name ( #method_args_str ).await
                }
            });
        }

        let res = quote! {
            use crate::object::Object;
            impl Object<ArcPointer> {
                #(#list)*
            }
        };
        res.to_tokens(tokens);
    }
}

impl Parse for FileTrait {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            input: input.parse()?,
            methods: Vec::new(),
            async_methods: Vec::new(),
        })
    }
}

#[derive(Default)]
struct IdentWrapper {
    key: String,
}

impl<'ast> Visit<'ast> for IdentWrapper {
    fn visit_ident(&mut self, node: &'ast Ident) {
        self.key = node.to_string();

        visit::visit_ident(self, node);
    }
}

impl IdentWrapper {
    fn is_key(&self) -> bool {
        self.key == "key".to_string()
    }
}

impl ToTokens for FileTrait {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.input.to_tokens(tokens);

        self.methods_to_tokens(tokens);
        self.async_methods_to_tokens(tokens);
    }
}
