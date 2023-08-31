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
  %{"data" => %{"image" => image_url}} = response.body

  new_image_url =
    String.replace(
      image_url,
      "https://s3.us-west-004.backblazeb2.com/BUCKET-NAME/",
      "https://BUCKET-NAME.web.example.com/"
    )

  # Patch the task
  updated_data = %{"data" => %{"image" => new_image_url}}

  Req.patch!(req, url: "/api/tasks/#{task_number}", json: updated_data)
end

#update_image_url.(8491)
#|> IO.inspect()

task_ids = [1,2,3,4]

Enum.each(task_ids, update_image_url)
