use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    token::Comma,
    visit::{self, Visit},
    FnArg, GenericParam, Generics, ItemTrait, Pat, Token, TraitItemMethod, WhereClause,
};

pub struct FileTrait {
    pub(crate) input: ItemTrait,
    pub(crate) methods: Vec<TraitItemMethod>,
    pub(crate) async_methods: Vec<TraitItemMethod>,
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
        let final_params = Self::get_params(&generics.params);
        let where_clause = Self::get_where_clause(&generics.where_clause);

        (final_params, where_clause)
    }

    #[inline]
    fn get_params(params: &Punctuated<GenericParam, Token![,]>) -> TokenStream {
        let final_params = if params.is_empty() {
            quote! { Ft }
        } else {
            let params: Vec<TokenStream> = params
                .into_iter()
                .filter(|p| match p {
                    GenericParam::Type(t) => {
                        if t.ident.to_string() == "OP" {
                            false
                        } else {
                            true
                        }
                    }
                    _ => true,
                })
                .map(|p| p.to_token_stream())
                .collect();

            quote! { #(#params,)* Ft }
        };

        final_params
    }

    #[inline]
    fn get_where_clause(where_clause: &Option<WhereClause>) -> TokenStream {
        match where_clause {
            Some(e) => {
                let predicates = e.predicates.to_token_stream();
                quote! { where #predicates Ft: File  }
            }
            None => {
                quote! { where Ft: File }
            }
        }
    }

    fn get_attrs(attrs: &Vec<syn::Attribute>) -> TokenStream {
        let attrs_token: Vec<TokenStream> = attrs
            .into_iter()
            .filter(|&attr| {
                if attr.path.to_token_stream().to_string() == "doc" {
                    false
                } else {
                    true
                }
            })
            .map(|res| res.to_token_stream())
            .collect();

        quote! { #(#attrs_token)* }
    }

    fn methods_to_tokens(&self, tokens: &mut TokenStream) {
        if self.methods.len() == 0 {
            return;
        }

        let mut list = Vec::with_capacity(self.methods.len());
        for TraitItemMethod { sig, attrs, .. } in &self.methods {
            let method = sig;
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

            let attrs_final = FileTrait::get_attrs(&attrs);

            list.push(quote! {
                #[inline]
                #attrs_final
                pub fn #method_name < #final_params >(#inputs_str #filer ) #output #where_clause  {
                    let path = self.path();
                    filer. #method_name ( #method_args_str )
                }
            });
        }

        let res = quote! {
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
        for TraitItemMethod { sig, attrs, .. } in &self.async_methods {
            let method = sig;
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
            let filer = quote! { filer: &Ft , };

            let attrs_final = FileTrait::get_attrs(&attrs);

            list.push(quote! {
                #[inline]
                #attrs_final
                pub async fn #method_name < #final_params >(#inputs_str #filer ) #output #where_clause  {
                    let path = self.path();
                    filer. #method_name ( #method_args_str ).await
                }
            });
        }

        let res = quote! {
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
        self.key == "path".to_string()
    }
}

impl ToTokens for FileTrait {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.input.to_tokens(tokens);

        self.methods_to_tokens(tokens);
        self.async_methods_to_tokens(tokens);
    }
}
