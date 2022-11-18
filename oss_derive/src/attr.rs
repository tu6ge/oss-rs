use syn::parse::{Parse, ParseStream, Result};

mod kw {
    syn::custom_keyword!(ASYNC);
}

pub struct Attribute {
    pub send: bool,
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Ok(Self { send: false });
        }

        let lookahead = input.lookahead1();
        if lookahead.peek(kw::ASYNC) {
            input.parse::<kw::ASYNC>()?;
            Ok(Self { send: true })
        } else {
            Ok(Self { send: false })
        }
    }
}
