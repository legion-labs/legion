
import cars

client = cars.Client("http://127.0.0.1:3000")

response = client.get_car(cars.GetCarRequest(
    "kd_space",
    0,
))
print(vars(response))

response = client.create_car(cars.CreateCarRequest(
    "kd_space",
    "kd_span",
    cars.Car(
        0, "Opel", cars.CarColor.BLUE, False, ""
    )
))
print(vars(response))

response = client.create_car(cars.CreateCarRequest(
    "kd_space",
    "kd_span",
    cars.Car(
        1, "Lada", cars.CarColor.RED, True, ""
    )
))
print(vars(response))
print(cars.CarColor("red"))

car_response = client.get_car(cars.GetCarRequest(
    "kd_space",
    1
))
print(car_response.car.__dict__)

response = client.get_cars(cars.GetCarsRequest(
    "kd_space",
    ["Opel", "Lada"]
))
print(vars(response))
for car in response.car:
    print(vars(car))

response = client.delete_car(cars.DeleteCarRequest("kd_space", 0))
print(vars(response))

response = client.test_binary(cars.TestBinaryRequest("kd_space", "-- test string --"))
print(vars(response))

response = client.test_one_of(cars.TestOneOfRequest())
print(response.json())
print(vars(response))

