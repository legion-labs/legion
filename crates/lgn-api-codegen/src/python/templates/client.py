import requests
response = requests.get('https://api.github.com')
print(response)
print(response.headers)
print(response.content)