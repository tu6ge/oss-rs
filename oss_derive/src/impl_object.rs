use syn::TraitItem;

use crate::file::FileTrait;

pub fn impl_object(file: &mut FileTrait) {
    let items = &file.input.items;

    for inner in items {
        if let TraitItem::Method(method) = inner {
            let sig = &method.sig;
            match sig.asyncness {
                Some(_) => {
                    file.async_methods.push(sig.clone());
                }
                None => {
                    file.methods.push(sig.clone());
                }
            }
        }
    }
}
