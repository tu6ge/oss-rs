use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_quote,
    visit_mut::{self, VisitMut},
    TraitItemMethod, WhereClause,
};

pub(crate) struct GenWhere(TraitItemMethod);

impl Parse for GenWhere {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self(input.parse()?))
    }
}

impl ToTokens for GenWhere {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut item = self.0.clone();
        AppendWhere.visit_trait_item_method_mut(&mut item);

        item.to_tokens(tokens);
    }
}

struct AppendWhere;

impl VisitMut for AppendWhere {
    fn visit_trait_item_method_mut(&mut self, item: &mut TraitItemMethod) {
        visit_mut::visit_trait_item_method_mut(self, item);
    }

    fn visit_where_clause_mut(&mut self, i: &mut WhereClause) {
        i.predicates
            .push(parse_quote! {OP: TryInto<ObjectPath> + Send + Sync});
        i.predicates
            .push(parse_quote! {<OP as TryInto<ObjectPath>>::Error: Into<Self::Error>});

        visit_mut::visit_where_clause_mut(self, i);
    }
}
