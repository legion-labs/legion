from enum import Enum
class NotEnum:
    def __init__():
        print("NotEnum")

class YesEnum(Enum):
    def __init__():
        print("YesEnum")


import cars

client = cars.Client("http://127.0.0.1:3000")
response = client.create_car("kd", None, None)