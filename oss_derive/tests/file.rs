use oss_derive::oss_file;

fn main() {
    assert!(true);
}

#[oss_file]
pub trait File {
    fn foo1(&self, _a: String, key: &str, _b: String) -> String {
        String::from(key)
    }

    fn foo2<T: Into<String>>(&self, _a: T) -> String {
        String::from("abc")
    }

    fn foo3<T: Into<String>, F>(&self, _a: T, _b: F, key: &str) -> String
    where
        F: Fn(&Vec<u8>) -> &'static str,
    {
        let mut string = String::from(key);
        string.push_str("abc");
        string
    }
}

pub struct RcPointer;

pub struct Base;

impl Base {
    pub fn path(&self) -> String {
        String::from("path_bar")
    }
}

pub struct Object<T> {
    pub inner: T,
    base: Base,
}
