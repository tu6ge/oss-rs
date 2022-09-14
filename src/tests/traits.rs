
mod object_list_xml{
    

    #[test]
    fn from_xml(){
        use crate::traits::ObjectTrait;
        use crate::traits::MockObjectListTrait;
        use mockall::predicate::*;
        use mockall::Predicate;

        struct ObjectA {}

        impl ObjectTrait for ObjectA {
            fn from_oss(key:String,last_modified:String,etag:String,_type:String,size:String,storage_class:String) -> crate::errors::OssResult<Self> {
                Ok(ObjectA{})
            }
        }
        
        let ctx = MockObjectListTrait::<ObjectA>::from_oss_context();
        ctx.expect().times(1).withf(|name,prefix, max_keys,key_count,_object_list,next_token|{
            
            eq("foo_bucket".to_string()).eval(name) 
            && eq("".to_string()).eval(prefix)
            && eq("100".to_string()).eval(max_keys)
            && eq("3".to_string()).eval(key_count)
            && eq(None).eval(next_token)
            //object_list
        });

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

        //let list1 = ctx::from_oss(xml.to_string());
        

        // assert!(list1.is_ok());
    }
}
