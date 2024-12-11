use aws_sdk_s3::config::BehaviorVersion;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::delete_objects::DeleteObjectsError;
use aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error;
use aws_sdk_s3::Client;
use indicatif::ProgressBar;
use std::env;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub async fn list_old_objects(
    client: &aws_sdk_s3::Client,
    bucket: &str,
) -> Result<Vec<String>, SdkError<ListObjectsV2Error>> {
    let mut keys_to_delete = vec![];
    let mut response = client
        .list_objects_v2()
        .bucket(bucket.to_owned())
        .into_paginator()
        .send();

    let one_day_ago = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
        - Duration::from_secs(86400))
    .as_secs_f64();

    let list_pb = ProgressBar::new_spinner();
    let mut count = 0;
    while let Some(result) = response.next().await {
        match result {
            Ok(output) => {
                for object in output.contents() {
                    list_pb.inc(1);
                    count += 1;
                    list_pb.set_message(format!("Listing obects: {}/???", count));
                    if let (Some(key), Some(last_modified)) = (object.key(), object.last_modified())
                    {
                        if last_modified.as_secs_f64() < one_day_ago {
                            keys_to_delete.push(key.to_owned());
                        }
                    }
                }
            }
            Err(e) => return Err(e),
        }
    }
    list_pb.finish_with_message("ListObjects complete.");

    Ok(keys_to_delete)
}

pub async fn delete_objects(
    client: &aws_sdk_s3::Client,
    keys_to_delete: &Vec<String>,
    bucket: &str,
) -> Result<(), SdkError<DeleteObjectsError>> {
    let mut delete_object_ids: Vec<aws_sdk_s3::types::ObjectIdentifier> = vec![];
    for obj in keys_to_delete {
        let obj_id = aws_sdk_s3::types::ObjectIdentifier::builder()
            .key(obj)
            .build()
            .unwrap();
        delete_object_ids.push(obj_id);
    }

    let delete_pb = ProgressBar::new(delete_object_ids.len() as u64);

    for chunk in delete_object_ids.chunks(1000) {
        client
                .delete_objects()
                .bucket(bucket)
                .delete(
                    aws_sdk_s3::types::Delete::builder()
                        .set_objects(Some(chunk.to_vec()))
                        .build()
                        .unwrap(),
                )
                .send()
                .await?;

        delete_pb.inc(chunk.len() as u64);
    }

    delete_pb.finish_with_message("DeleteObjects complete.");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), aws_sdk_s3::Error> {
    let key_id = env::var("AWS_KEY_ID").expect("AWS_KEY_ID must be set");
    let secret_key = env::var("AWS_SECRET_KEY").expect("AWS_SECRET_KEY must be set");
    let region_str = env::var("AWS_REGION").expect("AWS_REGION must be set");
    let endpoint_url = env::var("AWS_ENDPOINT_URL").expect("AWS_ENDPOINT_URL must be set");
    let bucket_name = env::var("AWS_BUCKET").expect("AWS_BUCKET must be set");

    let credentials = Credentials::new(key_id, secret_key, None, None, "manual");
    let region = Region::new(region_str);

    let config = aws_sdk_s3::Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .force_path_style(true)
        .credentials_provider(credentials)
        .region(region)
        .endpoint_url(endpoint_url)
        .build();

    let client = Client::from_conf(config);

    let keys_to_delete = list_old_objects(&client, &bucket_name).await.unwrap();
    delete_objects(&client, &keys_to_delete, &bucket_name)
        .await
        .unwrap();

    Ok(())
}
