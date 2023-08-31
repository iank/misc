#!/usr/bin/env elixir

Mix.install([:req, :jason])

# Set up request object with Authorization token
req =
  Req.new(
    base_url: "https://labelstudio.example.com",
    headers: %{Authorization: "Token xxxXXXxxxXXXxxxXXXxxxXXXxxxXXXxxxXXXxxxX"}
  )

update_image_url = fn task_number when is_integer(task_number) ->
  # Get the image URL from the task
  response = Req.get!(req, url: "/api/tasks/#{task_number}")
  %{"data" => original_data} = response.body
  image_url = original_data["image"]

  new_image_url =
    String.replace(
      image_url,
      "https://s3.us-west-004.backblazeb2.com/BUCKET-NAME/",
      "https://BUCKET-NAME.web.example.com/"
    )

  # Patch the task
  updated_data = Map.put(original_data, "image", new_image_url)

  #IO.puts("#{image_url} -> #{new_image_url}")

  Req.patch!(req, url: "/api/tasks/#{task_number}", json: %{"data" => updated_data})
  #|> IO.inspect()

  task_number
end

task_ids = [1,2,3,4]

task_ids
|> Task.async_stream(fn task_id ->
    update_image_url.(task_id)
  end, max_concurrency: 10)
|> Enum.each(fn {:ok, result} ->
  IO.puts(result)
end)
