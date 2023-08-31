I had tasks in a Label Studio instance that referred to images in S3. I've moved
the images to Garage and needed to rewrite the URLs.

I got the task IDs straight from the database because I didn't want to deal
with LS's pagination today:

```
select string_agg(id::text, ',') from task where data->>'image' LIKE '%backblaze%';
```
