use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_quote,
    visit_mut::{self, VisitMut},
    ItemImpl, TypeArray,
};

pub struct FormQuery {
    inner: ItemImpl,
    count: u8,
}

impl Parse for FormQuery {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            inner: input.parse()?,
            count: 1,
        })
    }
}

impl ToTokens for FormQuery {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.inner.to_tokens(tokens);

        self.extend_to_tokens(tokens);
    }
}

impl FormQuery {
    fn extend_to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for index in 2..=self.count {
            let mut item = self.inner.clone();
            let mut visit = UpdateUsize(index);
            visit.visit_item_impl_mut(&mut item);

            item.to_tokens(tokens);
        }
    }
}

pub fn update_count(query: &mut FormQuery, count: u8) {
    query.count = count;
}

pub use attr::GetCount;

mod attr {
    use syn::{
        parse::{Parse, ParseStream, Result},
        LitInt,
    };

    pub struct GetCount {
        pub count: u8,
    }

    impl Parse for GetCount {
        fn parse(input: ParseStream) -> Result<Self> {
            let lit: LitInt = input.parse()?;
            Ok(Self {
                count: lit.base10_parse()?,
            })
        }
    }
}

struct UpdateUsize(u8);

impl VisitMut for UpdateUsize {
    fn visit_item_impl_mut(&mut self, i: &mut ItemImpl) {
        visit_mut::visit_item_impl_mut(self, i);
    }

    fn visit_type_array_mut(&mut self, node: &mut TypeArray) {
        let len = self.0 as usize;
        node.len = parse_quote! {#len};
        visit_mut::visit_type_array_mut(self, node);
    }
}
