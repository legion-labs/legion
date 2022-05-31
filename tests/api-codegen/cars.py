# ---------- Responses -------
from enum import Enum, auto
class GetCarsResponse(Enum):
    status_200 = 200 # list[Car] # List of cars.
    
    def __init__(self, response):
        print(response)
        match response.status_code:
            case 200:
                self.json = response.json()
                pass
            case _:
                raise Exception("unexpected status code: {}".format(response.status))
     

class CreateCarResponse(Enum):
    status_201 = 201 # Created.
    
    def __init__(self, response):
        match response.status_code:
            case 201:pass
            case _:
                raise Exception("unexpected status code: {}".format(response.status))
     


class GetCarResponse(Enum):
    status_200 = 200 # Car # A car.
    status_404 = 404 # Car not found.
    
    def __init__(self, response):
        match response.status_code:
            case 200:
                self.json = response.json()
                pass
            case 404:pass
            case _:
                raise Exception("unexpected status code: {}".format(response.status))
     

class DeleteCarResponse(Enum):
    status_200 = 200 # Car deleted.
    status_404 = 404 # Car not found.
    
    def __init__(self, response):
        match response.status_code:
            case 200:pass
            case 404:pass
            case _:
                raise Exception("unexpected status code: {}".format(response.status))
     


class TestBinaryResponse(Enum):
    status_200 = 200 # bytearray # Ok.
    
    def __init__(self, response):
        match response.status_code:
            case 200:
                self.bytes = response.contentpass
            case _:
                raise Exception("unexpected status code: {}".format(response.status))
     


class TestOneOfResponse(Enum):
    status_200 = 200 # TestOneOfResponse # Ok.
    
    def __init__(self, response):
        match response.status_code:
            case 200:
                self.json = response.json()
                pass
            case _:
                raise Exception("unexpected status code: {}".format(response.status))
     



# ---------- Api ----------
import abc

class Api(metaclass=abc.ABCMeta):

    @abc.abstractmethod
    def get_cars(
        self,
        space_id : str,
        names : list[str],
        q : str,
        body,
        extra,
    ) -> GetCarsResponse:
        raise NotImplementedError
    
    @abc.abstractmethod
    def create_car(
        self,
        space_id : str,
        span_id : str,
        body,
        extra,
    ) -> CreateCarResponse:
        raise NotImplementedError
    

    @abc.abstractmethod
    def get_car(
        self,
        space_id : str,
        car_id : int,
        body,
        extra,
    ) -> GetCarResponse:
        raise NotImplementedError
    
    @abc.abstractmethod
    def delete_car(
        self,
        space_id : str,
        car_id : int,
        body,
        extra,
    ) -> DeleteCarResponse:
        raise NotImplementedError
    

    @abc.abstractmethod
    def test_binary(
        self,
        space_id : str,
        body,
        extra,
    ) -> TestBinaryResponse:
        raise NotImplementedError
    

    @abc.abstractmethod
    def test_one_of(
        self,
        body,
        extra,
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

class Client(Api):
    def __init__(self, uri):
        self.uri = uri

    
    def get_cars(
        self,
        space_id : str,
        names : list[str],
        q : str,
        body,
        extra,
    ) -> GetCarsResponse:
        uri = "{}/v1/spaces/{}/car-service/cars".format(
            self.uri,
            space_id,
        )
        params = {
            "names" : names,
            "q" : q,
        }
        # Initalizing for consistency but not used
        _params = GetCarsQuery(
            names,
            q,
        )

        resp = requests.get(
            uri,
            params = params,
        )
        print(resp.status_code)
        print('label 0')
        return GetCarsResponse(resp)
    def create_car(
        self,
        space_id : str,
        span_id : str,
        body,
        extra,
    ) -> CreateCarResponse:
        uri = "{}/v1/spaces/{}/car-service/cars".format(
            self.uri,
            space_id,
        )
        headers = {
            "Content-type": "application/json",
        }
        
        if span_id != None:
            headers["span-id"] = "span_id"

        resp = requests.post(
            uri,
            headers = headers
        )

        return CreateCarResponse(resp)
    
    
    def get_car(
        self,
        space_id : str,
        car_id : int,
        body,
        extra,
    ) -> GetCarResponse:
        uri = "{}/v1/spaces/{}/car-service/cars/{}".format(
            self.uri,
            space_id,
            car_id,
        )

        resp = requests.get(
            uri,
        )

        return GetCarResponse(resp)
    def delete_car(
        self,
        space_id : str,
        car_id : int,
        body,
        extra,
    ) -> DeleteCarResponse:
        uri = "{}/v1/spaces/{}/car-service/cars/{}".format(
            self.uri,
            space_id,
            car_id,
        )

        resp = requests.delete(
            uri,
        )

        return DeleteCarResponse(resp)
    
    
    def test_binary(
        self,
        space_id : str,
        body,
        extra,
    ) -> TestBinaryResponse:
        uri = "{}/v1/spaces/{}/car-service/test-binary".format(
            self.uri,
            space_id,
        )
        headers = {
            "Content-type": "application/octet-stream",
        }
        

        resp = requests.post(
            uri,
            headers = headers
        )

        return TestBinaryResponse(resp)
    
    
    def test_one_of(
        self,
        body,
        extra,
    ) -> TestOneOfResponse:
        uri = "{}/test-one-of".format(
            self.uri,
        )

        resp = requests.get(
            uri,
        )

        return TestOneOfResponse(resp)
    
    

