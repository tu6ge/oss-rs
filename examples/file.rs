use std::{fs, path::Path};

use aliyun_oss_client::{
    config::get_url_resource,
    file::{File, FileError, GetStd},
    types::CanonicalizedResource,
    BucketName, Client, EndPoint, Error as OssError, KeyId, KeySecret,
};
use reqwest::Url;

struct MyObject {
    path: String,
}

impl MyObject {
    const KEY_ID: KeyId = KeyId::from_static("xxxxxxxxxxxxxxxx");
    const KEY_SECRET: KeySecret = KeySecret::from_static("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
    const END_POINT: EndPoint = EndPoint::CnShanghai;
    const BUCKET: BucketName = unsafe { BucketName::from_static2("xxxxxx") };

    fn new(path: &Path) -> Result<MyObject, FileError> {
        Ok(MyObject {
            path: path.to_str().unwrap().to_owned(),
        })
    }
}

impl GetStd for MyObject {
    fn get_std(&self) -> Option<(Url, CanonicalizedResource)> {
        let path = self.path.clone().try_into().unwrap();
        Some(get_url_resource(&Self::END_POINT, &Self::BUCKET, &path))
    }
}

impl File<Client> for MyObject {
    fn oss_client(&self) -> Client {
        Client::new(
            Self::KEY_ID,
            Self::KEY_SECRET,
            Self::END_POINT,
            Self::BUCKET,
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), OssError> {
    for entry in fs::read_dir("examples")? {
        let path = entry?.path();
        let path = path.as_path();

        if !path.is_file() {
            continue;
        }

        let obj = MyObject::new(path)?;
        let content = fs::read(path)?;

        let res = obj.put_oss(content, "application/pdf").await?;

        println!("result status: {}", res.status());
    }

    Ok(())
}
