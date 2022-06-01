from enum import IntEnum, Enum, auto
class NotEnum:
    def __init__(self):
        print("NotEnum")

class YesEnum(Enum):
    YES = auto()

class Test:
    def __init__(self):
        self.x = 10

class CarColor(Enum):

    RED = "red"

    BLUE = "blue"

    YELLOW = "yellow"

color = CarColor.RED
print(color.__dict__)
class Car:
    def __init__(
        self,
        id : int, 
        name : str, 
        color : CarColor, # The car color.
        #test: Test,
        is_new : bool, 
        extra : bytearray, 
    ):
        self.id = id
        self.name = name
        self.color = color
        #self.test = test
        self.is_new = is_new
        self.extra = extra


from json import JSONEncoder
class ModelEncoder(JSONEncoder):
    def default(self, o):
        return o.__dict__

print(ModelEncoder().encode(CarColor.RED))

#x = YesEnum.YES
print(ModelEncoder().encode(Car(
        0,
        "Keks",
        CarColor.BLUE,
        #Test(),
        False, 
        ""
    )))


import cars

client = cars.Client("http://127.0.0.1:3000")

response = client.get_car(cars.GetCarRequest(
    "kd_space",
    0,
))

response = client.create_car(cars.CreateCarRequest(
    "kd_space",
    "kd_span",
    cars.Car(
        0, "Keks", cars.CarColor.BLUE, False, []
    )
))