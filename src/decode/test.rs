static mut OBEJCT_ITEM_ID: i8 = 0;

use thiserror::Error;

use crate::decode::ListError;

#[derive(Debug, Error)]
#[error("custom")]
struct MyError {}

impl ListError for MyError {}

mod object_list_xml {

    use super::MyError;

    #[cfg(feature = "core")]
    use crate::object::{Object, ObjectList};

    use super::OBEJCT_ITEM_ID;

    #[test]
    fn from_xml() {
        use crate::decode::RefineObject;
        use crate::decode::RefineObjectList;

        #[derive(Default)]
        struct ObjectA {}

        impl RefineObject<MyError> for ObjectA {
            fn set_key(&mut self, key: &str) -> Result<(), MyError> {
                unsafe {
                    if OBEJCT_ITEM_ID == 0 {
                        assert_eq!(key, "9AB932LY.jpeg");
                    } else if OBEJCT_ITEM_ID == 1 {
                        assert_eq!(key, "CHANGELOG.md");
                    } else if OBEJCT_ITEM_ID == 2 {
                        assert_eq!(key, "LICENSE");
                    }
                }
                Ok(())
            }
            fn set_last_modified(&mut self, last_modified: &str) -> Result<(), MyError> {
                unsafe {
                    if OBEJCT_ITEM_ID == 0 {
                        assert_eq!(last_modified, "2022-06-26T09:53:21.000Z");
                    } else if OBEJCT_ITEM_ID == 1 {
                        assert_eq!(last_modified, "2022-06-12T06:11:06.000Z");
                    } else if OBEJCT_ITEM_ID == 2 {
                        assert_eq!(last_modified, "2022-06-12T06:11:06.000Z");
                    }
                }
                Ok(())
            }
            fn set_etag(&mut self, etag: &str) -> Result<(), MyError> {
                unsafe {
                    if OBEJCT_ITEM_ID == 0 {
                        assert_eq!(etag, "F75A15996D0857B16FA31A3B16624C26");
                    } else if OBEJCT_ITEM_ID == 1 {
                        assert_eq!(etag, "09C37AC5B145D368D52D0AAB58B25213");
                    } else if OBEJCT_ITEM_ID == 2 {
                        assert_eq!(etag, "2CBAB10A50CC6905EA2D7CCCEF31A6C9");
                    }
                }
                Ok(())
            }
            fn set_type(&mut self, _type: &str) -> Result<(), MyError> {
                unsafe {
                    if OBEJCT_ITEM_ID == 0 {
                        assert_eq!(_type, "Normal");
                    } else if OBEJCT_ITEM_ID == 1 {
                        assert_eq!(_type, "Normal");
                    } else if OBEJCT_ITEM_ID == 2 {
                        assert_eq!(_type, "Normal");
                    }
                }
                Ok(())
            }
            fn set_size(&mut self, size: &str) -> Result<(), MyError> {
                unsafe {
                    if OBEJCT_ITEM_ID == 0 {
                        assert_eq!(size, "18027");
                    } else if OBEJCT_ITEM_ID == 1 {
                        assert_eq!(size, "40845");
                    } else if OBEJCT_ITEM_ID == 2 {
                        assert_eq!(size, "1065");
                    }
                }
                Ok(())
            }
            fn set_storage_class(&mut self, storage_class: &str) -> Result<(), MyError> {
                assert_eq!(storage_class, "Standard");
                unsafe {
                    OBEJCT_ITEM_ID += 1;
                }
                Ok(())
            }
        }
        struct ListB {}
        impl RefineObjectList<ObjectA, MyError, MyError> for ListB {
            fn set_name(&mut self, name: &str) -> Result<(), MyError> {
                assert_eq!(name, "foo_bucket");
                Ok(())
            }
            fn set_prefix(&mut self, prefix: &str) -> Result<(), MyError> {
                assert_eq!(prefix, "foo_prefix");
                Ok(())
            }
            fn set_max_keys(&mut self, max_keys: &str) -> Result<(), MyError> {
                assert_eq!(max_keys, "100");
                Ok(())
            }
            fn set_key_count(&mut self, key_count: &str) -> Result<(), MyError> {
                assert_eq!(key_count, "3");
                Ok(())
            }
            fn set_next_continuation_token_str(&mut self, token: &str) -> Result<(), MyError> {
                assert!(token.is_empty());
                Ok(())
            }
            // fn set_list(self, list: Vec<T>) -> Result<Self, InvalidObjectListValue>{
            //     Ok(self)
            // }
        }

        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ListBucketResult>
          <Name>foo_bucket</Name>
          <Prefix>foo_prefix</Prefix>
          <MaxKeys>100</MaxKeys>
          <Delimiter></Delimiter>
          <IsTruncated>false</IsTruncated>
          <Contents>
            <Key>9AB932LY.jpeg</Key>
            <LastModified>2022-06-26T09:53:21.000Z</LastModified>
            <ETag>"F75A15996D0857B16FA31A3B16624C26"</ETag>
            <Type>Normal</Type>
            <Size>18027</Size>
            <StorageClass>Standard</StorageClass>
          </Contents>
          <Contents>
            <Key>CHANGELOG.md</Key>
            <LastModified>2022-06-12T06:11:06.000Z</LastModified>
            <ETag>"09C37AC5B145D368D52D0AAB58B25213"</ETag>
            <Type>Normal</Type>
            <Size>40845</Size>
            <StorageClass>Standard</StorageClass>
          </Contents>
          <Contents>
            <Key>LICENSE</Key>
            <LastModified>2022-06-12T06:11:06.000Z</LastModified>
            <ETag>"2CBAB10A50CC6905EA2D7CCCEF31A6C9"</ETag>
            <Type>Normal</Type>
            <Size>1065</Size>
            <StorageClass>Standard</StorageClass>
          </Contents>
          <KeyCount>3</KeyCount>
        </ListBucketResult>"#;

        // let base = BucketBase::new("abc".try_into().unwrap(), EndPoint::CnQingdao);

        let mut list = ListB {};

        let res = list.decode(xml, || ObjectA {});

        assert!(res.is_ok());
    }

    // bench result        5,210 ns/iter (+/- 151)
    // update to &str      4,262 ns/iter (+/- 96)
    // update to &mut self 3,718 ns/iter (+/- 281)
    #[cfg(test)]
    #[cfg(feature = "bench")]
    #[bench]
    fn from_xml_bench(b: &mut test::Bencher) {
        use crate::decode::RefineObject;
        use crate::decode::RefineObjectList;

        #[derive(Default)]
        struct ObjectA {}

        impl RefineObject for ObjectA {
            type Bucket = Arc<BucketBase>;
            type Error = OssError;
            fn set_key(&mut self, key: &str) -> Result<(), OssError> {
                Ok(())
            }
            fn set_last_modified(&mut self, last_modified: &str) -> Result<(), OssError> {
                Ok(())
            }
            fn set_etag(&mut self, etag: &str) -> Result<(), OssError> {
                Ok(())
            }
            fn set_type(&mut self, _type: &str) -> Result<(), OssError> {
                Ok(())
            }
            fn set_size(&mut self, size: &str) -> Result<(), OssError> {
                Ok(())
            }
            fn set_storage_class(&mut self, storage_class: &str) -> Result<(), OssError> {
                Ok(())
            }
            fn set_bucket(self, bucket: Arc<BucketBase>) -> Self {
                self
            }
        }
        struct ListB {}
        impl RefineObjectList<ObjectA> for ListB {
            type Error = OssError;
        }

        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ListBucketResult>
          <Name>foo_bucket</Name>
          <Prefix></Prefix>
          <MaxKeys>100</MaxKeys>
          <Delimiter></Delimiter>
          <IsTruncated>false</IsTruncated>
          <Contents>
            <Key>9AB932LY.jpeg</Key>
            <LastModified>2022-06-26T09:53:21.000Z</LastModified>
            <ETag>"F75A15996D0857B16FA31A3B16624C26"</ETag>
            <Type>Normal</Type>
            <Size>18027</Size>
            <StorageClass>Standard</StorageClass>
          </Contents>
          <Contents>
            <Key>CHANGELOG.md</Key>
            <LastModified>2022-06-12T06:11:06.000Z</LastModified>
            <ETag>"09C37AC5B145D368D52D0AAB58B25213"</ETag>
            <Type>Normal</Type>
            <Size>40845</Size>
            <StorageClass>Standard</StorageClass>
          </Contents>
          <Contents>
            <Key>LICENSE</Key>
            <LastModified>2022-06-12T06:11:06.000Z</LastModified>
            <ETag>"2CBAB10A50CC6905EA2D7CCCEF31A6C9"</ETag>
            <Type>Normal</Type>
            <Size>1065</Size>
            <StorageClass>Standard</StorageClass>
          </Contents>
          <KeyCount>3</KeyCount>
        </ListBucketResult>"#;

        let mut list = ListB {};

        b.iter(|| {
            let base = BucketBase::new("abc".parse().unwrap(), EndPoint::CnQingdao);
            list.decode(xml, Arc::new(base));
        })
    }

    // fn init_object_list(token: Option<String>, list: Vec<Object<RcPointer>>) -> ObjectList<RcPointer> {
    //     let client = ClientRc::new(
    //         "foo1".into(),
    //         "foo2".into(),
    //         "https://oss-cn-shanghai.aliyuncs.com".try_into().unwrap(),
    //         "foo4".try_into().unwrap(),
    //     );

    //     let object_list = ObjectList::<RcPointer>::new(
    //         BucketBase::from_str("abc.oss-cn-shanghai.aliyuncs.com").unwrap(),
    //         String::from("foo2"),
    //         100,
    //         200,
    //         list,
    //         token,
    //         Rc::new(client),
    //         [],
    //     );

    //     object_list
    // }

    #[cfg(feature = "core")]
    #[allow(dead_code)]
    fn init_object_list(token: Option<String>, list: Vec<Object>) -> ObjectList {
        use crate::builder::ArcPointer;
        use crate::client::Client;
        use std::sync::Arc;

        let client = Client::new(
            "foo1".into(),
            "foo2".into(),
            "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
            "foo4".parse().unwrap(),
        );

        let object_list = ObjectList::<ArcPointer>::new(
            "abc.oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
            Some("foo2/".parse().unwrap()),
            100,
            200,
            list,
            token,
            Arc::new(client),
            [],
        );

        object_list
    }

    // update to &mut self 5,015 ns/iter (+/- 212)
    #[cfg(test)]
    #[cfg(feature = "bench")]
    #[bench]
    fn from_xml_bench_real_object(b: &mut test::Bencher) {
        use crate::decode::RefineObject;
        use crate::decode::RefineObjectList;

        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ListBucketResult>
          <Name>foo_bucket</Name>
          <Prefix></Prefix>
          <MaxKeys>100</MaxKeys>
          <Delimiter></Delimiter>
          <IsTruncated>false</IsTruncated>
          <Contents>
            <Key>9AB932LY.jpeg</Key>
            <LastModified>2022-06-26T09:53:21.000Z</LastModified>
            <ETag>"F75A15996D0857B16FA31A3B16624C26"</ETag>
            <Type>Normal</Type>
            <Size>18027</Size>
            <StorageClass>Standard</StorageClass>
          </Contents>
          <Contents>
            <Key>CHANGELOG.md</Key>
            <LastModified>2022-06-12T06:11:06.000Z</LastModified>
            <ETag>"09C37AC5B145D368D52D0AAB58B25213"</ETag>
            <Type>Normal</Type>
            <Size>40845</Size>
            <StorageClass>Standard</StorageClass>
          </Contents>
          <Contents>
            <Key>LICENSE</Key>
            <LastModified>2022-06-12T06:11:06.000Z</LastModified>
            <ETag>"2CBAB10A50CC6905EA2D7CCCEF31A6C9"</ETag>
            <Type>Normal</Type>
            <Size>1065</Size>
            <StorageClass>Standard</StorageClass>
          </Contents>
          <KeyCount>3</KeyCount>
        </ListBucketResult>"#;

        let mut list = init_object_list(None, vec![]);
        b.iter(|| {
            let base = BucketBase::new("abc".parse().unwrap(), EndPoint::CnQingdao);
            list.decode(xml, Arc::new(base));
        })
    }

    #[test]
    fn from_xml_has_next() {
        use crate::decode::RefineObject;
        use crate::decode::RefineObjectList;

        #[derive(Default)]
        struct ObjectA {}

        impl RefineObject<MyError> for ObjectA {}

        struct ListB {}
        impl RefineObjectList<ObjectA, MyError, MyError> for ListB {
            fn set_next_continuation_token_str(&mut self, token: &str) -> Result<(), MyError> {
                assert!(
                    token == "CiphcHBzL1RhdXJpIFB1Ymxpc2ggQXBwXzAuMS42X3g2NF9lbi1VUy5tc2kQAA--"
                );
                Ok(())
            }
        }

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
          <Contents>
            <Key>CHANGELOG.md</Key>
            <LastModified>2022-06-12T06:11:06.000Z</LastModified>
            <ETag>"09C37AC5B145D368D52D0AAB58B25213"</ETag>
            <Type>Normal</Type>
            <Size>40845</Size>
            <StorageClass>Standard</StorageClass>
          </Contents>
          <Contents>
            <Key>LICENSE</Key>
            <LastModified>2022-06-12T06:11:06.000Z</LastModified>
            <ETag>"2CBAB10A50CC6905EA2D7CCCEF31A6C9"</ETag>
            <Type>Normal</Type>
            <Size>1065</Size>
            <StorageClass>Standard</StorageClass>
          </Contents>
          <KeyCount>3</KeyCount>
        </ListBucketResult>"#;

        //let base = BucketBase::new("abc".try_into().unwrap(), EndPoint::CnQingdao);

        let mut list = ListB {};

        let init_object = || ObjectA {};

        let res = list.decode(xml, init_object);

        assert!(res.is_ok());
    }
}

mod bucket_xml {
    use super::MyError;

    #[test]
    fn from_xml() {
        use crate::decode::RefineBucket;

        struct BucketA {}

        impl RefineBucket<MyError> for BucketA {
            fn set_name(&mut self, name: &str) -> Result<(), MyError> {
                assert_eq!(name, "foo");
                Ok(())
            }
            fn set_creation_date(&mut self, creation_date: &str) -> Result<(), MyError> {
                assert_eq!(creation_date, "2016-11-05T13:10:10.000Z");
                Ok(())
            }
            fn set_location(&mut self, location: &str) -> Result<(), MyError> {
                assert_eq!(location, "oss-cn-shanghai");
                Ok(())
            }
            fn set_extranet_endpoint(&mut self, extranet_endpoint: &str) -> Result<(), MyError> {
                assert_eq!(extranet_endpoint, "oss-cn-shanghai.aliyuncs.com");
                Ok(())
            }
            fn set_intranet_endpoint(&mut self, intranet_endpoint: &str) -> Result<(), MyError> {
                assert_eq!(intranet_endpoint, "oss-cn-shanghai-internal.aliyuncs.com");
                Ok(())
            }
            fn set_storage_class(&mut self, storage_class: &str) -> Result<(), MyError> {
                assert_eq!(storage_class, "Standard");
                Ok(())
            }
        }

        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <BucketInfo>
          <Bucket>
            <AccessMonitor>Disabled</AccessMonitor>
            <Comment></Comment>
            <CreationDate>2016-11-05T13:10:10.000Z</CreationDate>
            <CrossRegionReplication>Disabled</CrossRegionReplication>
            <DataRedundancyType>LRS</DataRedundancyType>
            <ExtranetEndpoint>oss-cn-shanghai.aliyuncs.com</ExtranetEndpoint>
            <IntranetEndpoint>oss-cn-shanghai-internal.aliyuncs.com</IntranetEndpoint>
            <Location>oss-cn-shanghai</Location>
            <Name>foo</Name>
            <ResourceGroupId>rg-foobar</ResourceGroupId>
            <StorageClass>Standard</StorageClass>
            <TransferAcceleration>Disabled</TransferAcceleration>
            <Owner>
              <DisplayName>100889</DisplayName>
              <ID>3004212</ID>
            </Owner>
            <AccessControlList>
              <Grant>public-read</Grant>
            </AccessControlList>
            <ServerSideEncryptionRule>
              <SSEAlgorithm>None</SSEAlgorithm>
            </ServerSideEncryptionRule>
            <BucketPolicy>
              <LogBucket></LogBucket>
              <LogPrefix></LogPrefix>
            </BucketPolicy>
          </Bucket>
        </BucketInfo>"#;

        let info = BucketA {}.decode(xml);

        assert!(info.is_ok());
    }
}
static mut BUCKETS_ITEM_ID: i8 = 0;
mod bucket_list_xml {
    use super::MyError;

    use super::BUCKETS_ITEM_ID;

    #[test]
    fn from_xml() {
        use crate::decode::{RefineBucket, RefineBucketList};

        #[derive(Default)]
        struct BucketA {}

        impl RefineBucket<MyError> for BucketA {
            fn set_name(&mut self, name: &str) -> Result<(), MyError> {
                unsafe {
                    if BUCKETS_ITEM_ID == 0 {
                        assert_eq!(name, "foo124442");
                    } else if BUCKETS_ITEM_ID == 1 {
                        assert_eq!(name, "foo342390bar");
                    }
                }

                Ok(())
            }
            fn set_creation_date(&mut self, creation_date: &str) -> Result<(), MyError> {
                unsafe {
                    if BUCKETS_ITEM_ID == 0 {
                        assert_eq!(creation_date, "2020-09-13T03:14:54.000Z");
                    } else if BUCKETS_ITEM_ID == 1 {
                        assert_eq!(creation_date, "2016-11-05T13:10:10.000Z");
                    }
                }
                Ok(())
            }
            fn set_location(&mut self, location: &str) -> Result<(), MyError> {
                unsafe {
                    if BUCKETS_ITEM_ID == 0 {
                        assert_eq!(location, "oss-cn-shanghai");
                    } else if BUCKETS_ITEM_ID == 1 {
                        assert_eq!(location, "oss-cn-shanghai");
                    }
                }
                Ok(())
            }
            fn set_extranet_endpoint(&mut self, extranet_endpoint: &str) -> Result<(), MyError> {
                assert_eq!(extranet_endpoint, "oss-cn-shanghai.aliyuncs.com");
                Ok(())
            }
            fn set_intranet_endpoint(&mut self, intranet_endpoint: &str) -> Result<(), MyError> {
                assert_eq!(intranet_endpoint, "oss-cn-shanghai-internal.aliyuncs.com");
                Ok(())
            }
            fn set_storage_class(&mut self, storage_class: &str) -> Result<(), MyError> {
                assert_eq!(storage_class, "Standard");
                unsafe {
                    BUCKETS_ITEM_ID += 1;
                }
                Ok(())
            }
        }

        struct ListA {}

        impl RefineBucketList<BucketA, MyError> for ListA {
            fn set_prefix(&mut self, prefix: &str) -> Result<(), MyError> {
                assert_eq!(prefix, "");
                Ok(())
            }
            fn set_marker(&mut self, marker: &str) -> Result<(), MyError> {
                assert_eq!(marker, "");
                Ok(())
            }
            fn set_max_keys(&mut self, max_keys: &str) -> Result<(), MyError> {
                assert_eq!(max_keys, "");
                Ok(())
            }
            fn set_is_truncated(&mut self, is_truncated: bool) -> Result<(), MyError> {
                assert_eq!(is_truncated, false);
                Ok(())
            }
            fn set_next_marker(&mut self, next_marker: &str) -> Result<(), MyError> {
                assert_eq!(next_marker, "");
                Ok(())
            }
            fn set_id(&mut self, id: &str) -> Result<(), MyError> {
                assert_eq!(id, "100861222333");
                Ok(())
            }
            fn set_display_name(&mut self, display_name: &str) -> Result<(), MyError> {
                assert_eq!(display_name, "100861222");
                Ok(())
            }
        }

        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ListAllMyBucketsResult>
          <Owner>
            <ID>100861222333</ID>
            <DisplayName>100861222</DisplayName>
          </Owner>
          <Buckets>
            <Bucket>
              <Comment></Comment>
              <CreationDate>2020-09-13T03:14:54.000Z</CreationDate>
              <ExtranetEndpoint>oss-cn-shanghai.aliyuncs.com</ExtranetEndpoint>
              <IntranetEndpoint>oss-cn-shanghai-internal.aliyuncs.com</IntranetEndpoint>
              <Location>oss-cn-shanghai</Location>
              <Name>foo124442</Name>
              <Region>cn-shanghai</Region>
              <StorageClass>Standard</StorageClass>
            </Bucket>
            <Bucket>
              <Comment></Comment>
              <CreationDate>2016-11-05T13:10:10.000Z</CreationDate>
              <ExtranetEndpoint>oss-cn-shanghai.aliyuncs.com</ExtranetEndpoint>
              <IntranetEndpoint>oss-cn-shanghai-internal.aliyuncs.com</IntranetEndpoint>
              <Location>oss-cn-shanghai</Location>
              <Name>foo342390bar</Name>
              <Region>cn-shanghai</Region>
              <StorageClass>Standard</StorageClass>
            </Bucket>
          </Buckets>
        </ListAllMyBucketsResult>"#;

        let mut list = ListA {};
        let res = list.decode(xml, || BucketA {});

        assert!(res.is_ok());
    }
}

mod some_tests {

    use std::{borrow::Cow, fmt};

    use crate::{
        decode::{InnerItemError, InnerListError, ListErrorKind, RefineObject, RefineObjectList},
        types::InvalidEndPoint,
    };

    use super::*;

    #[test]
    fn test_one_object_decode() {
        struct ObjectA {}
        impl RefineObject<MyError> for ObjectA {
            fn set_key(&mut self, key: &str) -> Result<(), MyError> {
                assert!(key == "LICENSE");
                Ok(())
            }
            fn set_last_modified(&mut self, last_modified: &str) -> Result<(), MyError> {
                assert!(last_modified == "2022-06-12T06:11:06.000Z");
                Ok(())
            }
            fn set_etag(&mut self, etag: &str) -> Result<(), MyError> {
                assert!(etag == "2CBAB10A50CC6905EA2D7CCCEF31A6C9");
                Ok(())
            }
            fn set_type(&mut self, _type: &str) -> Result<(), MyError> {
                assert!(_type == "Normal");
                Ok(())
            }
            fn set_size(&mut self, size: &str) -> Result<(), MyError> {
                assert!(size == "1065");
                Ok(())
            }
            fn set_storage_class(&mut self, storage_class: &str) -> Result<(), MyError> {
                assert!(storage_class == "Standard");
                Ok(())
            }
        }

        #[derive(Debug)]
        struct MyError {}

        impl From<InnerItemError> for MyError {
            fn from(_: InnerItemError) -> Self {
                MyError {}
            }
        }

        impl From<quick_xml::Error> for MyError {
            fn from(_: quick_xml::Error) -> Self {
                MyError {}
            }
        }

        impl fmt::Display for MyError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "demo")
            }
        }
        impl std::error::Error for MyError {}

        let xml = r#"<Key>LICENSE</Key>
            <LastModified>2022-06-12T06:11:06.000Z</LastModified>
            <ETag>"2CBAB10A50CC6905EA2D7CCCEF31A6C9"</ETag>
            <Type>Normal</Type>
            <Size>1065</Size>
            <StorageClass>Standard</StorageClass>"#;

        let mut object = ObjectA {};
        let _ = object.decode(xml);
    }

    #[test]
    fn test_common_prefixes() {
        use std::error::Error;
        struct ObjectA {}
        impl RefineObject<MyError> for ObjectA {}

        struct ListA {}

        #[derive(Debug)]
        struct MyError {}

        impl From<InnerItemError> for MyError {
            fn from(_: InnerItemError) -> Self {
                MyError {}
            }
        }

        impl From<quick_xml::Error> for MyError {
            fn from(_: quick_xml::Error) -> Self {
                MyError {}
            }
        }

        impl fmt::Display for MyError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "demo")
            }
        }

        impl Error for MyError {}
        impl ListError for MyError {}

        impl RefineObjectList<ObjectA, MyError, MyError> for ListA {
            fn set_prefix(&mut self, prefix: &str) -> Result<(), MyError> {
                assert!(prefix == "bar");
                Ok(())
            }

            fn set_common_prefix(&mut self, list: &[Cow<'_, str>]) -> Result<(), MyError> {
                assert!(list[0] == "foo1/");
                assert!(list[1] == "foo2/");
                Ok(())
            }
        }

        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ListBucketResult>
          <Prefix>bar</Prefix>
          <Contents>
            <Key>9AB932LY.jpeg</Key>
          </Contents>
          <Contents>
            <Key>9AB932LY.jpeg</Key>
          </Contents>
          <CommonPrefixes>
            <Prefix>foo1/</Prefix>
            <Prefix>foo2/</Prefix>
          </CommonPrefixes>
        </ListBucketResult>
        "#;

        let mut list = ListA {};

        let res = list.decode(xml, || ObjectA {});

        assert!(res.is_ok());
    }

    #[test]
    fn test_item_from() {
        let string = InvalidEndPoint { _priv: () };
        let err = InnerItemError::from(string);
        assert_eq!(
            format!("{err}"),
            "endpoint must not with `-` prefix or `-` suffix or `oss-` prefix"
        );
    }

    #[test]
    fn test_error_item_source() {
        let err = InnerItemError::new();
        assert_eq!(format!("{}", err.get_source().unwrap()), "demo");
    }

    #[test]
    fn test_list_from() {
        let string = InvalidEndPoint { _priv: () };
        let err: InnerListError = string.into();
        assert_eq!(
            format!("{err}"),
            "endpoint must not with `-` prefix or `-` suffix or `oss-` prefix"
        );
    }

    #[test]
    fn test_error_list_display() {
        let err = InnerItemError::new();
        let err_list: InnerListError = err.into();
        assert_eq!(err_list.to_string(), "demo");

        let err = InnerListError::from_xml();
        assert_eq!(format!("{err}"), "Cannot read text, expecting Event::Text");

        let string = InvalidEndPoint { _priv: () };
        let kind = ListErrorKind::Custom(Box::new(string));

        let err = InnerListError { kind };
        assert_eq!(
            format!("{err}"),
            "endpoint must not with `-` prefix or `-` suffix or `oss-` prefix"
        );
    }

    #[test]
    fn test_error_list_get_source() {
        let err = InnerItemError::new();
        let err_list: InnerListError = err.into();
        assert_eq!(format!("{}", err_list.get_source().unwrap()), "demo");

        let err_list = InnerListError::from_xml();
        assert_eq!(
            format!("{}", err_list.get_source().unwrap()),
            "Cannot read text, expecting Event::Text"
        );

        let err_list = InnerListError::from_custom();
        assert_eq!(format!("{}", err_list.get_source().unwrap()), "custom");
    }

    #[test]
    fn test_error_list_from_item() {
        let err = InnerListError {
            kind: ListErrorKind::Xml(quick_xml::Error::TextNotFound),
        };
        assert_eq!(format!("{err}"), "Cannot read text, expecting Event::Text");

        fn bar() -> InnerListError {
            InnerItemError::new().into()
        }

        assert_eq!(
            format!("{:?}", bar()),
            "InnerListError { kind: Item(InnerItemError(MyError)) }"
        );
    }

    #[test]
    fn test_error_list_from_xml() {
        let err = InnerListError {
            kind: ListErrorKind::Xml(quick_xml::Error::TextNotFound),
        };
        assert_eq!(format!("{err}"), "Cannot read text, expecting Event::Text");

        fn bar() -> InnerListError {
            quick_xml::Error::TextNotFound.into()
        }

        assert_eq!(
            format!("{:?}", bar()),
            "InnerListError { kind: Xml(TextNotFound) }"
        );
    }
}
