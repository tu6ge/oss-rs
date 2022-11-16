use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::FnArg;
use syn::Pat;
use syn::visit::{self, Visit};
use syn::{
    parse::{Parse, ParseStream, Result},
    ItemTrait, Signature,
};

pub struct FileTrait {
    pub(crate) input: ItemTrait,
    pub(crate) methods: Vec<Signature>,
    pub(crate) async_methods: Vec<Signature>,
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
struct IdentWrapper{
    key: String,
}

impl<'ast> Visit<'ast> for IdentWrapper {
    fn visit_ident(&mut self, node: &'ast Ident){
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

        let mut list = Vec::with_capacity(self.methods.len());
        for method in &self.methods {
            let generics = method.generics.clone();
            let where_clause = generics.where_clause.clone();
            let output = method.output.clone();
            let method_name = method.ident.clone();
            if method_name.to_string() == "get_url".to_string() {
                continue;
            }

            let inputs_has_self = method.inputs.clone().into_iter()
            .filter(|arg|{
                match arg {
                    FnArg::Receiver(_) => true,
                    FnArg::Typed(pattype) => {
                        match &*pattype.pat {
                            Pat::Ident(i) => {
                                let mut a = IdentWrapper::default();
                                a.visit_ident(&i.ident);
                                if a.is_key() {
                                    false
                                }else{
                                    true
                                }
                            },
                            _ => true,
                        }
                    }
                }
            });

            let all_inputs: Vec<TokenStream> = inputs_has_self
            .map(|f| f.to_token_stream())
            .collect();

            let inputs: Vec<TokenStream> = method.inputs.clone().into_iter()
            .filter(|arg|{
                match arg {
                    FnArg::Receiver(_) => false,
                    FnArg::Typed(_) => true,
                }
            })
            .map(|arg|{
                match arg {
                    FnArg::Receiver(_) => {
                        unreachable!("不会有这种情况");
                    },
                    FnArg::Typed(pattype) => {
                        match &*pattype.pat {
                            Pat::Ident(i) => {
                                i.ident.to_token_stream()
                            },
                            _ => {
                                panic!("没有考虑这种情况");
                            }
                        }
                    },
                }
            }).collect();

            let inputs_str = quote!{ #(#all_inputs,)* };
            let inner_inputs_str = quote!{ #(#inputs,)* };
            let filer = quote!{ filer: &Ft };


            let final_params = if generics.params.is_empty() {
                quote!{ Ft }
            } else {
                let token = generics.params.to_token_stream();
                quote!{ #token , Ft}
            };

            let where_clause = match where_clause {
                Some(e) => {
                    let predicates = e.predicates.to_token_stream();
                    quote!{ where #predicates Ft: File  }
                },
                None => {
                    quote!{ where Ft: File  }
                }
            };

            list.push(quote! {
                pub fn #method_name < #final_params >(#inputs_str #filer ) #output #where_clause  {
                    let ref key = self.base.path().to_string();

                    filer. #method_name ( #inner_inputs_str )
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
}
