
mod test_async{
    use std::{env, path::PathBuf};

    use dotenv::dotenv;

    use assert_matches::assert_matches;
    use aliyun_oss_client::{client, types::{BucketName, Query}};

    #[tokio::test]
    async fn test_get_bucket_list(){
        dotenv().ok();

        let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
        let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
        let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
        let bucket      = env::var("ALIYUN_BUCKET").unwrap();

        let client = client(key_id,key_secret, endpoint, bucket);

        let bucket_list = client.get_bucket_list().await;

        assert_matches!(bucket_list, Ok(_));
    }

    #[tokio::test]
    async fn test_get_bucket_info(){
        dotenv().ok();

        let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
        let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
        let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
        let bucket      = env::var("ALIYUN_BUCKET").unwrap();

        let client = client(key_id,key_secret, endpoint, bucket);

        let bucket_list = client.get_bucket_info().await;

        assert_matches!(bucket_list, Ok(_));
    }

    #[tokio::test]
    async fn get_object_by_bucket_struct(){
        dotenv().ok();

        let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
        let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
        let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();

        let client = client(key_id,key_secret, endpoint, BucketName::from_static(""));

        let bucket_list = client.get_bucket_list().await.unwrap();
        let mut query = Query::new();
        query.insert("max-keys", "5");
        query.insert("prefix".to_string(), "babel".to_string());

        let buckets = bucket_list.buckets;
        let the_bucket = &buckets[0];
        let object_list = the_bucket.get_object_list(query).await;
        assert_matches!(object_list, Ok(_));
    }

    #[tokio::test]
    async fn test_get_object() {
        dotenv().ok();

        let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
        let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
        let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
        let bucket      = env::var("ALIYUN_BUCKET").unwrap();

        let client = client(key_id,key_secret, endpoint, bucket);
        let query = Query::new();

        let object_list = client.get_object_list(query).await;

        assert_matches!(object_list, Ok(_));
    }

    #[tokio::test]
    async fn test_put_and_delete_file(){
        dotenv().ok();

        let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
        let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
        let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
        let bucket      = env::var("ALIYUN_BUCKET").unwrap();

        let client = client(key_id,key_secret, endpoint, bucket);

        let object_list = client.put_file(PathBuf::from("examples/bg2015071010.png"), "examples/bg2015071010.png").await;

        assert_matches!(object_list, Ok(_));

        let result = client.delete_object("examples/bg2015071010.png").await;

        assert_matches!(result, Ok(_));
    }

}

#[cfg(feature = "blocking")]
mod test_blocking{
    
    use std::{env, path::PathBuf};
    use aliyun_oss_client::blocking::client;
    use aliyun_oss_client::types::Query;
    use dotenv::dotenv;
    use assert_matches::assert_matches;

    #[test]
    fn test_get_bucket_list(){
        dotenv().ok();

        let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
        let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
        let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
        let bucket      = env::var("ALIYUN_BUCKET").unwrap();

        let client = client(key_id,key_secret, endpoint, bucket);

        let bucket_list = client.get_bucket_list();

        assert_matches!(bucket_list, Ok(_));
    }

    #[test]
    fn test_get_bucket_info(){
        dotenv().ok();

        let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
        let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
        let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
        let bucket      = env::var("ALIYUN_BUCKET").unwrap();

        let client = client(key_id,key_secret, endpoint, bucket);

        let bucket_list = client.get_bucket_info();

        assert_matches!(bucket_list, Ok(_));
    }

    #[test]
    fn get_object_by_bucket_struct(){
        use aliyun_oss_client::types::BucketName;
        dotenv().ok();

        let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
        let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
        let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();

        let client = client(key_id,key_secret, endpoint, BucketName::from_static(""));

        let bucket_list = client.get_bucket_list().unwrap();
        let mut query = Query::new();
        query.insert("max-keys", "2");
        //query.insert("prefix".to_string(), "babel".to_string());

        let buckets = bucket_list.buckets;
        let the_bucket = &buckets[0];
        let object_list = the_bucket.get_object_list(query);
        assert_matches!(object_list, Ok(_));
        let mut object_list = object_list.unwrap();
        assert_matches!(object_list.next(), Some(_));
    }


    #[test]
    fn test_get_object() {
        dotenv().ok();

        let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
        let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
        let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
        let bucket      = env::var("ALIYUN_BUCKET").unwrap();

        let client = client(key_id,key_secret, endpoint, bucket);
        let query = Query::new();

        let object_list = client.get_object_list(query);

        assert_matches!(object_list, Ok(_));
    }

    #[test]
    fn test_get_object_next() {
        dotenv().ok();

        let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
        let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
        let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
        let bucket      = env::var("ALIYUN_BUCKET").unwrap();

        let client = client(key_id,key_secret, endpoint, bucket);
        let mut query = Query::new();
        query.insert("max-keys".to_string(), "2".to_string());
        let mut object_list = client.get_object_list(query).unwrap();

        assert_matches!(object_list.next(), Some(_));
        assert_matches!(object_list.next(), Some(_));
    }

    #[test]
    fn test_put_and_delete_file(){
        dotenv().ok();

        let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
        let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
        let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
        let bucket      = env::var("ALIYUN_BUCKET").unwrap();

        let client = client(key_id,key_secret, endpoint, bucket);

        // 第一种读取文件路径的方式
        let object_list = client.put_file(PathBuf::from("examples/bg2015071010.png"), "examples/bg2015071010.png");

        assert_matches!(object_list, Ok(_));

        let result = client.delete_object("examples/bg2015071010.png");

        assert_matches!(result, Ok(_));

        // 第二种读取文件路径的方式
        let object_list = client.put_file("examples/bg2015071010.png", "examples/bg2015071010.png");

        assert_matches!(object_list, Ok(_));

        let result = client.delete_object("examples/bg2015071010.png");

        assert_matches!(result, Ok(_));
    }

    // #[bench]
    // fn bench_get_object(b: &mut Bencher){
    //   dotenv().ok();

    //   let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
    //   let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
    //   let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
    //   let bucket      = env::var("ALIYUN_BUCKET").unwrap();

    //   let client = client::Client::new(key_id,key_secret, endpoint, bucket);
    //   b.iter(|| {
    //     client.get_object_list();
    //   });
    // }

}