use syn::TraitItem;

use crate::file::FileTrait;

pub fn impl_object(file: &mut FileTrait, is_send: bool) {
    let items = &file.input.items;

    let mut methods = Vec::new();

    for inner in items {
        if let TraitItem::Method(method) = inner {
            methods.push(method.clone());
        }
    }

    if is_send {
        file.async_methods = methods;
    } else {
        file.methods = methods;
    }
}
