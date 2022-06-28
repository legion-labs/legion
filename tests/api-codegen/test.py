
import api

client = api.Client("http://127.0.0.1:3000")

response = client.get_car(api.GetCarRequest(
    "kd_space",
    0,
))
print(response)

response = client.create_car(api.CreateCarRequest(
    "kd_space",
    "kd_span",
    api.Car(
        api.CarColor.BLUE, "", 0, False, "Opel"
    )
))
print(response)

response = client.create_car(api.CreateCarRequest(
    "kd_space",
    "kd_span",
    api.Car(
        api.CarColor.RED, "", 1, True, "Lada"
    )
))
print(response)

response = client.get_car(api.GetCarRequest(
    "kd_space",
    1
))
print(response)

response = client.get_cars(api.GetCarsRequest(
    "kd_space",
    "other_query",
    ["Opel", "Lada"]
))
print(response)

response = client.delete_car(api.DeleteCarRequest("kd_space", 0))
print(response)

response = client.test_binary(api.TestBinaryRequest("kd_space", "-- test string --"))
print(response)

response = client.test_one_of(api.TestOneOfRequest())
print(response)

