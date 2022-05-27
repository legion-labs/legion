import requests

class Client:
    def __init__(self, uri):
        self.uri = uri

    def get_cars(
        self,
        space_id,
        names,
        q,
        extra
    ):
        ""


response = requests.get('https://api.github.com')
print(response)
print(response.headers)
print(response.content)