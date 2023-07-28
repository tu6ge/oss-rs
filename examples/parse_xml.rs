use aliyun_oss_client::decode::{InnerListError, RefineObject, RefineObjectList};
use aliyun_oss_client::DecodeListError;
use thiserror::Error;

struct MyFile {
    key: String,
    #[allow(dead_code)]
    other: String,
}

impl RefineObject<MyError> for MyFile {
    fn set_key(&mut self, key: &str) -> Result<(), MyError> {
        self.key = key.to_string();
        Ok(())
    }
}

#[derive(Default)]
struct MyBucket {
    name: String,
    files: Vec<MyFile>,
}

impl RefineObjectList<MyFile, MyError> for MyBucket {
    fn set_name(&mut self, name: &str) -> Result<(), MyError> {
        self.name = name.to_string();
        Ok(())
    }
    fn set_list(&mut self, list: Vec<MyFile>) -> Result<(), MyError> {
        self.files = list;
        Ok(())
    }
}

#[derive(Debug, Error, DecodeListError)]
#[error("my error")]
struct MyError {}

fn get_with_xml() -> Result<(), InnerListError> {
    // 这是阿里云接口返回的原始数据
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ListBucketResult>
          <Name>foo_bucket</Name>
          <Prefix></Prefix>
          <MaxKeys>100</MaxKeys>
          <Delimiter></Delimiter>
          <IsTruncated>false</IsTruncated>
          <NextContinuationToken>CiphcHBzL1RhdXJpIFB1Ymxpc2ggQXBwXzAuMS42X3g2NF9lbi1VUy5tc2kQAA--</NextContinuationToken>
          <Contents>
            <Key>9AB932LY.jpeg</Key>
            <LastModified>2022-06-26T09:53:21.000Z</LastModified>
            <ETag>"F75A15996D0857B16FA31A3B16624C26"</ETag>
            <Type>Normal</Type>
            <Size>18027</Size>
            <StorageClass>Standard</StorageClass>
          </Contents>
          <KeyCount>3</KeyCount>
        </ListBucketResult>"#;

    // 除了设置Default 外，还可以做更多设置
    let mut bucket = MyBucket::default();

    // 利用闭包对 MyFile 做一下初始化设置
    fn init_file(_list: &mut MyBucket) -> std::io::Result<MyFile> {
        Ok(MyFile {
            key: String::default(),
            other: "abc".to_string(),
        })
    }

    bucket.decode(xml, init_file)?;

    assert!(bucket.name == "foo_bucket");
    assert!(bucket.files[0].key == "9AB932LY.jpeg");

    Ok(())
}

pub fn main() {
    let res = get_with_xml();

    if let Err(err) = res {
        eprintln!("{}", err);
    }
}
