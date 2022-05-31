import openapi

client = Client("http://127.0.0.1:3000")
response = client.create_car("kd", None, None)