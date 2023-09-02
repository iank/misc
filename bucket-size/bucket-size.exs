#!/usr/bin/env elixir

Mix.install([:aws, :hackney])

defmodule S3Paginator do
  def list_all_objects(client, bucket_name) do
    list_all_objects(client, bucket_name, nil, [])
  end

  defp list_all_objects(client, bucket_name, continuation_token, acc) do
    case AWS.S3.list_objects_v2(client, bucket_name, continuation_token) do
      {:ok,
       %{"ListBucketResult" => %{"Contents" => objects, "NextContinuationToken" => next_token}},
       _}
      when is_binary(next_token) ->
        list_all_objects(client, bucket_name, next_token, acc ++ objects)

      {:ok, %{"ListBucketResult" => %{"Contents" => objects}}, _} ->
        acc ++ objects

      {:error, _} = error ->
        error
    end
  end
end

client =
  AWS.Client.create(
    System.get_env("AWS_KEY_ID"),
    System.get_env("AWS_SECRET_KEY"),
    System.get_env("AWS_REGION")
  )
  |> AWS.Client.put_endpoint(System.get_env("AWS_ENDPOINT_HOST"))

{:ok, %{"ListAllMyBucketsResult" => %{ "Buckets" => %{ "Bucket" => buckets }}}, _} =
  AWS.S3.list_buckets(client)
bucket_names = Enum.map(buckets, fn bucket -> bucket["Name"] end)

bucket_names |> Enum.each(fn bucket_name ->
  objects = S3Paginator.list_all_objects(client, bucket_name)

  object_size =
    Enum.reduce(objects, 0, fn object, acc ->
      case object do
        %{"Size" => size} when is_binary(size) ->
          acc + String.to_integer(size)
      end
    end)

  IO.puts("#{bucket_name}: #{object_size/1024/1024 |> Float.round(1)} MiB")
end)
