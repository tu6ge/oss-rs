use std::{fs, path::Path};

use aliyun_oss_client::{
    config::ObjectPath,
    file::{File, FileError, Files},
    BucketName, Client, EndPoint, KeyId, KeySecret,
};

struct MyObject {
    path: ObjectPath,
}

impl MyObject {
    const KEY_ID: KeyId = KeyId::from_static("xxxxxxxxxxxxxxxx");
    const KEY_SECRET: KeySecret = KeySecret::from_static("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
    const END_POINT: EndPoint = EndPoint::CnShanghai;
    const BUCKET: BucketName = unsafe { BucketName::from_static2("xxxxxx") };

    fn new(path: &Path) -> Result<MyObject, FileError> {
        Ok(MyObject {
            path: path.try_into()?,
        })
    }
}

impl File for MyObject {
    type Client = Client;
    fn get_path(&self) -> ObjectPath {
        self.path.clone()
    }

    fn oss_client(&self) -> Self::Client {
        Client::new(
            Self::KEY_ID,
            Self::KEY_SECRET,
            Self::END_POINT,
            Self::BUCKET,
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), FileError> {
    for entry in fs::read_dir("examples")? {
        let path = entry?.path();
        let path = path.as_path();

        if !path.is_file() {
            continue;
        }

        let obj = MyObject::new(path)?;
        let content = fs::read(path)?;

        let res = obj.put_oss(content, Client::DEFAULT_CONTENT_TYPE).await?;

        println!("result status: {}", res.status());
    }

    Ok(())
}
