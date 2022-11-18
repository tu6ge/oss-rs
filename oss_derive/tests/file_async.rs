use async_trait::async_trait;
use oss_derive::oss_file;

fn main() {
    assert!(true);
}

//#[oss_file(ASYNC)]
#[async_trait]
pub trait File: Send + Sync {
    async fn foo1<OP: Into<ObjectPath> + Send + Sync>(
        &self,
        _a: String,
        path: OP,
        _b: String,
    ) -> String {
        let _p = path.into();
        String::from("abc")
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

pub struct ArcPointer;

pub struct Base;

pub struct ObjectPath;

pub struct Object<T> {
    pub inner: T,
}

impl<T> Object<T> {
    pub fn path(&self) -> ObjectPath {
        ObjectPath
    }
}
