use array2query::{update_count, FormQuery, GetCount};
use derive::impl_custom_item_error;
use derive::impl_custom_list_error;
use file::attr::Attribute;
use file::impl_object;
use file::FileTrait;
use proc_macro::TokenStream;

use quote::quote;
use syn::parse_macro_input;
mod array2query;
mod file;
mod gen_rc;
use crate::gen_rc::GenImpl;

/// # 转换 File trait 的各种方法到 Object 结构体中
/// 例如 `Client` 结构体中有 `put_file` 方法，通过这个 macro ，可以让 Object 结构体支持 `put_file` 方法
///
/// 注意，之前的方法签名是这样的 `put_file(filename, path)`，由于 Object 本身有 path 属性，转换后的方法是这样的
/// `put_file(filename, &filer)`，其中 filer 可以传入实现 `File` trait 的结构体，在 oss-rs 项目中，有 `Client`, `Bucket`, `ObjectList`
/// 等结构体已实现了该trait，可以直接传入使用，其他的也可以
#[proc_macro_attribute]
pub fn oss_file(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as Attribute);
    let mut item = parse_macro_input!(input as FileTrait);
    impl_object(&mut item, attr.send);
    TokenStream::from(quote!(#item))
}

/// # 根据 Arc 自动生成 Rc 代码
/// 目前支持的转换为：
///
/// ArcPointer => RcPointer
///
/// Arc => Rc
///
/// Arc::clone() => Rc::clone()
///
/// ClientArc => ClientRc
///
/// 还会在新生成的 `impl {}` 语句块之前添加 `#[cfg(feature = "blocking")]` 标记
#[proc_macro_attribute]
pub fn oss_gen_rc(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as GenImpl);
    TokenStream::from(quote!(#item))
}

/// # 根据长度1的数组，自动生成多个长度的数组的 impl
///
/// 例如:如果 `[(&str, &str)]` 可以转化成 Query
///
/// 则 `#[array2query(2)]` 可以让 `[(&str, &str), (&str, &str)]` 也可以转化成 Query
///
/// 以此类推
#[proc_macro_attribute]
pub fn array2query(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as GetCount);
    let mut item = parse_macro_input!(input as FormQuery);
    update_count(&mut item, attr.count);
    TokenStream::from(quote!(#item))
}

mod path_where;

/// # 为 `OP` 自动生成 `where` 语句
///
/// ```rust,ignore
/// where:
///     OP: TryInto<Path> + Send + Sync,
///     <OP as TryInto<Path>>::Error: Into<Self::Error>,
/// ```
#[proc_macro_attribute]
pub fn path_where(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as path_where::GenWhere);
    TokenStream::from(quote!(#item))
}

mod derive;

/// # 用于实现 `#[derive(CustomItemError)]`
/// 为实现 `RefineObject`,`RefineBucket` 等解析 trait 的外部类型，提供便捷的 Error 实现方式
#[proc_macro_derive(DecodeItemError)]
pub fn derive_decode_item_error(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_custom_item_error(&ast)
}

/// # 用于实现 `#[derive(CustomListError)]`
/// 为实现 `RefineObjectList`,`RefineBucketList` 等解析 trait 的外部类型，提供便捷的 Error 实现方式
#[proc_macro_derive(DecodeListError)]
pub fn derive_decode_list_error(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_custom_list_error(&ast)
}
