# ---------- Models ----------
from enum import Enum

from json import JSONEncoder

# TODO(kdaibov): starting with Python 3.11 we could use StrEnum
# in that case we could remove the ModelEncoder
class ModelEncoder(JSONEncoder):
    def default(self, o):
        if isinstance(o, Enum):
            return o.value
        else:
            return o.__dict__


# The car color.
class CarColor(Enum):
    RED = "red"
    BLUE = "blue"
    YELLOW = "yellow"
    def from_json(value):
        return CarColor(value)


# TODO(kdaibov): OneOf is not tested



class Car:
    def __init__(
        self,
        id : int, 
        name : str, 
        color : CarColor, # The car color.
        is_new : bool, 
        extra : str, 
    ):
        self.id = id
        self.name = name
        self.color = color
        self.is_new = is_new
        self.extra = extra

    def to_json(self):
        return ModelEncoder().encode(self)

    def from_json(body):
        id = body['id']
        name = body['name']
        color = CarColor.from_json(body['color'])
        is_new = body['is_new']
        extra = body['extra']
        return Car(
            id,
            name,
            color,
            is_new,
            extra,
        )
   



class Pet:
    def __init__(
        self,
        name : str, 
    ):
        self.name = name

    def to_json(self):
        return ModelEncoder().encode(self)

    def from_json(body):
        name = body['name']
        return Pet(
            name,
        )
   



class TestOneOfResponse:
    def __init__(self, value):
        self.value = value 
        self.type = None
    
        if isinstance(value, Pet):
            self.type = "Pet"
    
        if isinstance(value, Car):
            self.type = "Car"
    
                



# ---------- Requests -------



class GetCarsRequest:
    def __init__(
        self,
        space_id: str,
        names: list[str] = None,
        q: str = None,
    ):
        self.space_id = space_id
        
        if names:
            self.names = names
        if q:
            self.q = q
        pass


class CreateCarRequest:
    def __init__(
        self,
        space_id: str,
        span_id: str = None,
        body: Car = None,
    ):
        self.space_id = space_id
        
        if span_id:
            self.span_id = span_id
        self.body = body
        
        pass




class GetCarRequest:
    def __init__(
        self,
        space_id: str,
        car_id: int,
    ):
        self.space_id = space_id
        
        self.car_id = car_id
        
        pass


class DeleteCarRequest:
    def __init__(
        self,
        space_id: str,
        car_id: int,
    ):
        self.space_id = space_id
        
        self.car_id = car_id
        
        pass




class TestBinaryRequest:
    def __init__(
        self,
        space_id: str,
        body: str = None,
    ):
        self.space_id = space_id
        
        self.body = body
        
        pass




class TestOneOfRequest:
    def __init__(
        self,
    ):
        pass



# ---------- Responses -------

class GetCarsResponse:
    # status_200 = 200 # list[Car] # List of cars.
    
    def __init__(self, response):
        print("response: {}".format(response))
        print("response.text: {}".format(response.text))
        match response.status_code:
            case 200:
                self.json = response.json()
                pass
            case _:
                raise Exception("unexpected status code: {}".format(response.status_code))
     

class CreateCarResponse:
    # status_201 = 201 # Created.
    
    def __init__(self, response):
        print("response: {}".format(response))
        print("response.text: {}".format(response.text))
        match response.status_code:
            case 201:pass
            case _:
                raise Exception("unexpected status code: {}".format(response.status_code))
     


class GetCarResponse:
    # status_200 = 200 # Car # A car.
    # status_404 = 404 # Car not found.
    
    def __init__(self, response):
        print("response: {}".format(response))
        print("response.text: {}".format(response.text))
        match response.status_code:
            case 200:
                self.car = Car.from_json(response.json())
                pass
            case 404:pass
            case _:
                raise Exception("unexpected status code: {}".format(response.status_code))
     

class DeleteCarResponse:
    # status_200 = 200 # Car deleted.
    # status_404 = 404 # Car not found.
    
    def __init__(self, response):
        print("response: {}".format(response))
        print("response.text: {}".format(response.text))
        match response.status_code:
            case 200:pass
            case 404:pass
            case _:
                raise Exception("unexpected status code: {}".format(response.status_code))
     


class TestBinaryResponse:
    # status_200 = 200 # str # Ok.
    
    def __init__(self, response):
        print("response: {}".format(response))
        print("response.text: {}".format(response.text))
        match response.status_code:
            case 200:
                self.bytes = response.contentpass
            case _:
                raise Exception("unexpected status code: {}".format(response.status_code))
     


class TestOneOfResponse:
    # status_200 = 200 # TestOneOfResponse # Ok.
    
    def __init__(self, response):
        print("response: {}".format(response))
        print("response.text: {}".format(response.text))
        match response.status_code:
            case 200:
                self.test_one_of_response = TestOneOfResponse.from_json(response.json())
                pass
            case _:
                raise Exception("unexpected status code: {}".format(response.status_code))
     



# ---------- Api ----------
import abc

class Api(metaclass=abc.ABCMeta):

    @abc.abstractmethod
    def get_cars(
        self,
        request: GetCarsRequest,
    ) -> GetCarsResponse:
        raise NotImplementedError
    
    @abc.abstractmethod
    def create_car(
        self,
        request: CreateCarRequest,
    ) -> CreateCarResponse:
        raise NotImplementedError
    

    @abc.abstractmethod
    def get_car(
        self,
        request: GetCarRequest,
    ) -> GetCarResponse:
        raise NotImplementedError
    
    @abc.abstractmethod
    def delete_car(
        self,
        request: DeleteCarRequest,
    ) -> DeleteCarResponse:
        raise NotImplementedError
    

    @abc.abstractmethod
    def test_binary(
        self,
        request: TestBinaryRequest,
    ) -> TestBinaryResponse:
        raise NotImplementedError
    

    @abc.abstractmethod
    def test_one_of(
        self,
        request: TestOneOfRequest,
    ) -> TestOneOfResponse:
        raise NotImplementedError
    


# ---------- Parameters -------

class GetCarsQuery:
    def __init__(
        self,
        names : list[str],
        q : str,
    ):
        self.names = names
        self.q = q












# ---------- Client -------
import requests
import urllib

class Client(Api):
    def __init__(self, uri):
        self.uri = uri

    
    def get_cars(
        self,
        request: GetCarsRequest,
    ) -> GetCarsResponse:
        uri = "{}/v1/spaces/{}/car-service/cars".format(
            self.uri,
            request.space_id,
        )
        params = {}
        if hasattr(request, "names") and request.names:
            params["names{}".format(("","[]")[isinstance(request.names, list)])] = request.names
        if hasattr(request, "q") and request.q:
            params["q{}".format(("","[]")[isinstance(request.q, list)])] = request.q
        
        uri += "?{}".format(urllib.parse.urlencode(params, doseq=True))

        print("uri: {}".format(uri))
        print("params: {}".format(params))

        resp = requests.get(
            uri,
            #params = params,
        )


        return GetCarsResponse(resp)
    def create_car(
        self,
        request: CreateCarRequest,
    ) -> CreateCarResponse:
        uri = "{}/v1/spaces/{}/car-service/cars".format(
            self.uri,
            request.space_id,
        )
        headers = {
            "Content-type": "application/json",
        }
        
        
        if request.span_id != None:
            headers["span-id"] = request.span_id

        print("uri: {}".format(uri))
        print("headers: {}".format(headers))
        print("body: {}".format(request.body.to_json()))

        resp = requests.post(
            uri,
            headers = headers,
            data = request.body.to_json(),
        )


        return CreateCarResponse(resp)
    
    
    def get_car(
        self,
        request: GetCarRequest,
    ) -> GetCarResponse:
        uri = "{}/v1/spaces/{}/car-service/cars/{}".format(
            self.uri,
            request.space_id,
            request.car_id,
        )

        print("uri: {}".format(uri))

        resp = requests.get(
            uri,
        )


        return GetCarResponse(resp)
    def delete_car(
        self,
        request: DeleteCarRequest,
    ) -> DeleteCarResponse:
        uri = "{}/v1/spaces/{}/car-service/cars/{}".format(
            self.uri,
            request.space_id,
            request.car_id,
        )

        print("uri: {}".format(uri))

        resp = requests.delete(
            uri,
        )


        return DeleteCarResponse(resp)
    
    
    def test_binary(
        self,
        request: TestBinaryRequest,
    ) -> TestBinaryResponse:
        uri = "{}/v1/spaces/{}/car-service/test-binary".format(
            self.uri,
            request.space_id,
        )
        headers = {
            "Content-type": "application/octet-stream",
        }
        
        

        print("uri: {}".format(uri))
        print("headers: {}".format(headers))
        print("body: {}".format(request.body.to_json()))

        resp = requests.post(
            uri,
            headers = headers,
            data = request.body.to_json(),
        )


        return TestBinaryResponse(resp)
    
    
    def test_one_of(
        self,
        request: TestOneOfRequest,
    ) -> TestOneOfResponse:
        uri = "{}/test-one-of".format(
            self.uri,
        )

        print("uri: {}".format(uri))

        resp = requests.get(
            uri,
        )


        return TestOneOfResponse(resp)
    
    