static mut OBEJCT_ITEM_ID: i8 = 0;
mod object_list_xml {

    use std::sync::Arc;

    use crate::builder::ArcPointer;
    use crate::tests::traits::OBEJCT_ITEM_ID;
    use crate::{
        config::BucketBase,
        traits::{InvalidObjectListValue, InvalidObjectValue},
        EndPoint,
    };

    #[test]
    fn from_xml() {
        use crate::traits::OssIntoObject;
        use crate::traits::OssIntoObjectList;

        #[derive(Default)]
        struct ObjectA {}

        impl OssIntoObject<ArcPointer> for ObjectA {
            fn set_key(self, key: &str) -> Result<Self, InvalidObjectValue> {
                unsafe {
                    if OBEJCT_ITEM_ID == 0 {
                        assert_eq!(key, "9AB932LY.jpeg");
                    } else if OBEJCT_ITEM_ID == 1 {
                        assert_eq!(key, "CHANGELOG.md");
                    } else if OBEJCT_ITEM_ID == 2 {
                        assert_eq!(key, "LICENSE");
                    }
                }
                Ok(self)
            }
            fn set_last_modified(
                self,
                last_modified: &str,
            ) -> Result<Self, InvalidObjectValue> {
                unsafe {
                    if OBEJCT_ITEM_ID == 0 {
                        assert_eq!(last_modified, "2022-06-26T09:53:21.000Z");
                    } else if OBEJCT_ITEM_ID == 1 {
                        assert_eq!(last_modified, "2022-06-12T06:11:06.000Z");
                    } else if OBEJCT_ITEM_ID == 2 {
                        assert_eq!(last_modified, "2022-06-12T06:11:06.000Z");
                    }
                }
                Ok(self)
            }
            fn set_etag(self, etag: &str) -> Result<Self, InvalidObjectValue> {
                unsafe {
                    if OBEJCT_ITEM_ID == 0 {
                        assert_eq!(etag, "F75A15996D0857B16FA31A3B16624C26");
                    } else if OBEJCT_ITEM_ID == 1 {
                        assert_eq!(etag, "09C37AC5B145D368D52D0AAB58B25213");
                    } else if OBEJCT_ITEM_ID == 2 {
                        assert_eq!(etag, "2CBAB10A50CC6905EA2D7CCCEF31A6C9");
                    }
                }
                Ok(self)
            }
            fn set_type(self, _type: &str) -> Result<Self, InvalidObjectValue> {
                unsafe {
                    if OBEJCT_ITEM_ID == 0 {
                        assert_eq!(_type, "Normal");
                    } else if OBEJCT_ITEM_ID == 1 {
                        assert_eq!(_type, "Normal");
                    } else if OBEJCT_ITEM_ID == 2 {
                        assert_eq!(_type, "Normal");
                    }
                }
                Ok(self)
            }
            fn set_size(self, size: &str) -> Result<Self, InvalidObjectValue> {
                unsafe {
                    if OBEJCT_ITEM_ID == 0 {
                        assert_eq!(size, "18027");
                    } else if OBEJCT_ITEM_ID == 1 {
                        assert_eq!(size, "40845");
                    } else if OBEJCT_ITEM_ID == 2 {
                        assert_eq!(size, "1065");
                    }
                }
                Ok(self)
            }
            fn set_storage_class(
                self,
                storage_class: &str,
            ) -> Result<Self, InvalidObjectValue> {
                assert_eq!(storage_class, "Standard");
                unsafe {
                    OBEJCT_ITEM_ID += 1;
                }
                Ok(self)
            }
            fn set_bucket(self, bucket: Arc<BucketBase>) -> Self {
                assert_eq!(bucket.name(), "abc");
                self
            }
        }
        struct ListB {}
        impl OssIntoObjectList<ObjectA, ArcPointer> for ListB {
            fn set_name(self, name: &str) -> Result<Self, InvalidObjectListValue> {
                assert_eq!(name, "foo_bucket");
                Ok(self)
            }
            fn set_prefix(self, prefix: &str) -> Result<Self, InvalidObjectListValue> {
                assert_eq!(prefix, "");
                Ok(self)
            }
            fn set_max_keys(self, max_keys: &str) -> Result<Self, InvalidObjectListValue> {
                assert_eq!(max_keys, "100");
                Ok(self)
            }
            fn set_key_count(self, key_count: &str) -> Result<Self, InvalidObjectListValue> {
                assert_eq!(key_count, "3");
                Ok(self)
            }
            fn set_next_continuation_token(
                self,
                token: Option<&str>,
            ) -> Result<Self, InvalidObjectListValue> {
                assert!(matches!(token, None));
                Ok(self)
            }
            // fn set_list(self, list: Vec<T>) -> Result<Self, InvalidObjectListValue>{
            //     Ok(self)
            // }
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

        let base = BucketBase::new("abc".try_into().unwrap(), EndPoint::CnQingdao);

        let list = ListB {};

        let list1 = list.from_xml(xml, Arc::new(base));

        assert!(list1.is_ok());
    }

    // bench result 5,210 ns/iter (+/- 151)
    // update to &str 4,262 ns/iter (+/- 96)
    // #[cfg(test)]
    // #[bench]
    // fn from_xml_bench(b: &mut test::Bencher) {
    //     use crate::traits::OssIntoObject;
    //     use crate::traits::OssIntoObjectList;

    //     #[derive(Default)]
    //     struct ObjectA {}

    //     impl OssIntoObject<ArcPointer> for ObjectA {
    //         fn set_key(self, key: &str) -> Result<Self, InvalidObjectValue> {

    //             Ok(self)
    //         }
    //         fn set_last_modified(self, last_modified: &str) -> Result<Self, InvalidObjectValue> {

    //             Ok(self)
    //         }
    //         fn set_etag(self, etag: &str) -> Result<Self, InvalidObjectValue> {

    //             Ok(self)
    //         }
    //         fn set_type(self, _type: &str) -> Result<Self, InvalidObjectValue> {

    //             Ok(self)
    //         }
    //         fn set_size(self, size: &str) -> Result<Self, InvalidObjectValue> {
    //             Ok(self)
    //         }
    //         fn set_storage_class(self, storage_class: &str) -> Result<Self, InvalidObjectValue> {

    //             Ok(self)
    //         }
    //         fn set_bucket(self, bucket: Arc<BucketBase>) -> Self {

    //             self
    //         }
    //     }
    //     struct ListB {}
    //     impl OssIntoObjectList<ObjectA, ArcPointer> for ListB {
    //         fn set_name(self, name: &str) -> Result<Self, InvalidObjectListValue> {
    //             Ok(self)
    //         }
    //         fn set_prefix(self, prefix: &str) -> Result<Self, InvalidObjectListValue> {
    //             Ok(self)
    //         }
    //         fn set_max_keys(self, max_keys: &str) -> Result<Self, InvalidObjectListValue> {
    //             Ok(self)
    //         }
    //         fn set_key_count(self, key_count: &str) -> Result<Self, InvalidObjectListValue> {
    //             Ok(self)
    //         }
    //         fn set_next_continuation_token(
    //             self,
    //             token: Option<&str>,
    //         ) -> Result<Self, InvalidObjectListValue> {
    //             Ok(self)
    //         }
    //         // fn set_list(self, list: Vec<T>) -> Result<Self, InvalidObjectListValue>{
    //         //     Ok(self)
    //         // }
    //     }

    //     let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    //     <ListBucketResult>
    //       <Name>foo_bucket</Name>
    //       <Prefix></Prefix>
    //       <MaxKeys>100</MaxKeys>
    //       <Delimiter></Delimiter>
    //       <IsTruncated>false</IsTruncated>
    //       <Contents>
    //         <Key>9AB932LY.jpeg</Key>
    //         <LastModified>2022-06-26T09:53:21.000Z</LastModified>
    //         <ETag>"F75A15996D0857B16FA31A3B16624C26"</ETag>
    //         <Type>Normal</Type>
    //         <Size>18027</Size>
    //         <StorageClass>Standard</StorageClass>
    //       </Contents>
    //       <Contents>
    //         <Key>CHANGELOG.md</Key>
    //         <LastModified>2022-06-12T06:11:06.000Z</LastModified>
    //         <ETag>"09C37AC5B145D368D52D0AAB58B25213"</ETag>
    //         <Type>Normal</Type>
    //         <Size>40845</Size>
    //         <StorageClass>Standard</StorageClass>
    //       </Contents>
    //       <Contents>
    //         <Key>LICENSE</Key>
    //         <LastModified>2022-06-12T06:11:06.000Z</LastModified>
    //         <ETag>"2CBAB10A50CC6905EA2D7CCCEF31A6C9"</ETag>
    //         <Type>Normal</Type>
    //         <Size>1065</Size>
    //         <StorageClass>Standard</StorageClass>
    //       </Contents>
    //       <KeyCount>3</KeyCount>
    //     </ListBucketResult>"#;

    //     b.iter(||{
    //         let list = ListB {};
    //         let base = BucketBase::new("abc".try_into().unwrap(), EndPoint::CnQingdao);
    //         list.from_xml(xml, Arc::new(base));
    //     })
    // }

    #[test]
    fn from_xml_has_next() {
        use crate::traits::OssIntoObject;
        use crate::traits::OssIntoObjectList;

        #[derive(Default)]
        struct ObjectA {}

        impl OssIntoObject<ArcPointer> for ObjectA {}

        struct ListB {}
        impl OssIntoObjectList<ObjectA, ArcPointer> for ListB {
            fn set_next_continuation_token(
                self,
                token: Option<&str>,
            ) -> Result<Self, InvalidObjectListValue> {
                assert!(
                    matches!(token, Some(v) if v=="CiphcHBzL1RhdXJpIFB1Ymxpc2ggQXBwXzAuMS42X3g2NF9lbi1VUy5tc2kQAA--")
                );
                Ok(self)
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

        let base = BucketBase::new("abc".try_into().unwrap(), EndPoint::CnQingdao);

        let list = ListB {};

        let list1 = list.from_xml(xml, Arc::new(base));

        assert!(list1.is_ok());
    }
}

mod bucket_xml {
    use crate::traits::InvalidBucketValue;

    #[test]
    fn from_xml() {
        use crate::traits::OssIntoBucket;

        struct BucketA {}

        impl OssIntoBucket for BucketA {
            fn set_name(self, name: &str) -> Result<Self, InvalidBucketValue> {
                assert_eq!(name, "foo");
                Ok(self)
            }
            fn set_creation_date(self, creation_date: &str) -> Result<Self, InvalidBucketValue> {
                assert_eq!(creation_date, "2016-11-05T13:10:10.000Z");
                Ok(self)
            }
            fn set_location(self, location: &str) -> Result<Self, InvalidBucketValue> {
                assert_eq!(location, "oss-cn-shanghai");
                Ok(self)
            }
            fn set_extranet_endpoint(
                self,
                extranet_endpoint: &str,
            ) -> Result<Self, InvalidBucketValue> {
                assert_eq!(extranet_endpoint, "oss-cn-shanghai.aliyuncs.com");
                Ok(self)
            }
            fn set_intranet_endpoint(
                self,
                intranet_endpoint: &str,
            ) -> Result<Self, InvalidBucketValue> {
                assert_eq!(intranet_endpoint, "oss-cn-shanghai-internal.aliyuncs.com");
                Ok(self)
            }
            fn set_storage_class(self, storage_class: &str) -> Result<Self, InvalidBucketValue> {
                assert_eq!(storage_class, "Standard");
                Ok(self)
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

        let info = BucketA {}.from_xml(xml);

        assert!(info.is_ok());
    }
}
static mut BUCKETS_ITEM_ID: i8 = 0;
mod bucket_list_xml {
    use crate::traits::{InvalidBucketListValue, InvalidBucketValue};

    use super::BUCKETS_ITEM_ID;

    #[test]
    fn from_xml() {
        use crate::traits::{OssIntoBucket, OssIntoBucketList};

        #[derive(Default)]
        struct BucketA {}

        impl OssIntoBucket for BucketA {
            fn set_name(self, name: &str) -> Result<Self, InvalidBucketValue> {
                unsafe {
                    if BUCKETS_ITEM_ID == 0 {
                        assert_eq!(name, "foo124442");
                    } else if BUCKETS_ITEM_ID == 1 {
                        assert_eq!(name, "foo342390bar");
                    }
                }

                Ok(self)
            }
            fn set_creation_date(self, creation_date: &str) -> Result<Self, InvalidBucketValue> {
                unsafe {
                    if BUCKETS_ITEM_ID == 0 {
                        assert_eq!(creation_date, "2020-09-13T03:14:54.000Z");
                    } else if BUCKETS_ITEM_ID == 1 {
                        assert_eq!(creation_date, "2016-11-05T13:10:10.000Z");
                    }
                }
                Ok(self)
            }
            fn set_location(self, location: &str) -> Result<Self, InvalidBucketValue> {
                unsafe {
                    if BUCKETS_ITEM_ID == 0 {
                        assert_eq!(location, "oss-cn-shanghai");
                    } else if BUCKETS_ITEM_ID == 1 {
                        assert_eq!(location, "oss-cn-shanghai");
                    }
                }
                Ok(self)
            }
            fn set_extranet_endpoint(
                self,
                extranet_endpoint: &str,
            ) -> Result<Self, InvalidBucketValue> {
                assert_eq!(extranet_endpoint, "oss-cn-shanghai.aliyuncs.com");
                Ok(self)
            }
            fn set_intranet_endpoint(
                self,
                intranet_endpoint: &str,
            ) -> Result<Self, InvalidBucketValue> {
                assert_eq!(intranet_endpoint, "oss-cn-shanghai-internal.aliyuncs.com");
                Ok(self)
            }
            fn set_storage_class(self, storage_class: &str) -> Result<Self, InvalidBucketValue> {
                assert_eq!(storage_class, "Standard");
                unsafe {
                    BUCKETS_ITEM_ID += 1;
                }
                Ok(self)
            }
        }

        struct ListA {}

        impl OssIntoBucketList<BucketA> for ListA {
            fn set_prefix(self, prefix: &str) -> Result<Self, InvalidBucketListValue> {
                assert_eq!(prefix, "");
                Ok(self)
            }
            fn set_marker(self, marker: &str) -> Result<Self, InvalidBucketListValue> {
                assert_eq!(marker, "");
                Ok(self)
            }
            fn set_max_keys(self, max_keys: &str) -> Result<Self, InvalidBucketListValue> {
                assert_eq!(max_keys, "");
                Ok(self)
            }
            fn set_is_truncated(self, is_truncated: bool) -> Result<Self, InvalidBucketListValue> {
                assert_eq!(is_truncated, false);
                Ok(self)
            }
            fn set_next_marker(self, next_marker: &str) -> Result<Self, InvalidBucketListValue> {
                assert_eq!(next_marker, "");
                Ok(self)
            }
            fn set_id(self, id: &str) -> Result<Self, InvalidBucketListValue> {
                assert_eq!(id, "100861222333");
                Ok(self)
            }
            fn set_display_name(self, display_name: &str) -> Result<Self, InvalidBucketListValue> {
                assert_eq!(display_name, "100861222");
                Ok(self)
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

        let list = ListA {}.from_xml(xml);

        assert!(list.is_ok());
    }
}
