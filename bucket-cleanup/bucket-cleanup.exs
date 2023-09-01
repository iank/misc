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

objects = S3Paginator.list_all_objects(client, System.get_env("AWS_BUCKET"))

one_day_ago =
  DateTime.utc_now()
  |> DateTime.add(-1, :day)

objects_to_delete =
  Enum.filter(objects, fn object ->
    case object do
      %{"LastModified" => timestamp} ->
        dt = DateTime.from_iso8601(timestamp) |> elem(1)
        DateTime.compare(dt, one_day_ago) == :lt

      _ ->
        false
    end
  end)

objects_to_delete
|> IO.inspect()

length(objects_to_delete) |> IO.puts()

objects_to_delete
|> Task.async_stream(fn object ->
    AWS.S3.delete_object(client, System.get_env("AWS_BUCKET"), object["Key"], %{})
  end, max_concurrency: 10)
|> Enum.each(fn
  {:ok, {:ok, _, %{status_code: 204}}} ->
    nil
  {:ok, {:ok, _, %{status_code: code}}} ->
    IO.puts(code)
end)

# Enum.each(objects_to_delete, fn object ->
#  AWS.S3.delete_object(client, System.get_env("AWS_BUCKET"), object["Key"], %{})
# end)
