
static mut OBEJCT_ITEM_ID:i8 = 0;

mod object_list_xml{
    use crate::tests::traits::OBEJCT_ITEM_ID;


    #[test]
    fn from_xml(){
        use crate::traits::ObjectTrait;
        use crate::traits::ObjectListTrait;
        struct ObjectA {}

        impl ObjectTrait for ObjectA {
            fn from_oss(key:String,last_modified:String,etag:String,_type:String,size:String,storage_class:String) -> crate::errors::OssResult<Self> {
                unsafe {
                  if OBEJCT_ITEM_ID == 0 {
                    assert_eq!(key, "9AB932LY.jpeg".to_string());
                    assert_eq!(last_modified, "2022-06-26T09:53:21.000Z".to_string());
                    assert_eq!(etag, "F75A15996D0857B16FA31A3B16624C26".to_string());
                    assert_eq!(_type, "Normal".to_string());
                    assert_eq!(size, "18027".to_string());
                    assert_eq!(storage_class, "Standard".to_string());
                  }else if OBEJCT_ITEM_ID == 1 {
                    assert_eq!(key, "CHANGELOG.md".to_string());
                    assert_eq!(last_modified, "2022-06-12T06:11:06.000Z".to_string());
                    assert_eq!(etag, "09C37AC5B145D368D52D0AAB58B25213".to_string());
                    assert_eq!(_type, "Normal".to_string());
                    assert_eq!(size, "40845".to_string());
                    assert_eq!(storage_class, "Standard".to_string());
                  }else if OBEJCT_ITEM_ID == 2 {
                    assert_eq!(key, "LICENSE".to_string());
                    assert_eq!(last_modified, "2022-06-12T06:11:06.000Z".to_string());
                    assert_eq!(etag, "2CBAB10A50CC6905EA2D7CCCEF31A6C9".to_string());
                    assert_eq!(_type, "Normal".to_string());
                    assert_eq!(size, "1065".to_string());
                    assert_eq!(storage_class, "Standard".to_string());
                  }

                  OBEJCT_ITEM_ID += 1;
                }
                Ok(ObjectA{})
            }
        }
        
        struct ListB {}
        impl ObjectListTrait<ObjectA> for ListB {
            fn from_oss(name:String,prefix:String,max_keys:String,key_count:String,_object_list:Vec<ObjectA>,next_continuation_token:Option<String>,) -> crate::errors::OssResult<Self>where Self:Sized {
                assert_eq!(name, "foo_bucket".to_string());
                assert_eq!(prefix, "".to_string());
                assert_eq!(max_keys, "100".to_string());
                assert_eq!(key_count, "3".to_string());
                assert!(matches!(next_continuation_token, None));
                //assert_eq!(max_keys, "100".to_string());
                Ok(ListB {  })
            }
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

        let list1 = ListB::from_xml(xml.to_string());
        
        assert!(list1.is_ok());
    }

    #[test]
    fn from_xml_has_next(){
        use crate::traits::ObjectTrait;
        use crate::traits::ObjectListTrait;
        struct ObjectA {}

        impl ObjectTrait for ObjectA {
            fn from_oss(_key:String,_last_modified:String,_etag:String,_type:String,_size:String,_storage_class:String) -> crate::errors::OssResult<Self> {
                Ok(ObjectA{})
            }
        }
        
        struct ListB {}
        impl ObjectListTrait<ObjectA> for ListB {
            fn from_oss(name:String,prefix:String,max_keys:String,key_count:String,_object_list:Vec<ObjectA>,next_continuation_token:Option<String>,) -> crate::errors::OssResult<Self>where Self:Sized {
                assert_eq!(name, "foo_bucket".to_string());
                assert_eq!(prefix, "".to_string());
                assert_eq!(max_keys, "100".to_string());
                assert_eq!(key_count, "3".to_string());
                assert!(matches!(next_continuation_token, Some(t) if t == "CiphcHBzL1RhdXJpIFB1Ymxpc2ggQXBwXzAuMS42X3g2NF9lbi1VUy5tc2kQAA--".to_string()));
                //assert_eq!(max_keys, "100".to_string());
                Ok(ListB {  })
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

        let list1 = ListB::from_xml(xml.to_string());
        
        assert!(list1.is_ok());
    }
}
