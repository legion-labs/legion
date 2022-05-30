# ---------- Responses -------
from enum import Enum, auto
class GetCarsResponse(Enum):
    Ok = 200 # list[Car] # List of cars.
    
    def __init__(self, response):
        match response.status:
            case 200:
                self.json = response.json()
                
            case _:
                raise Exception("unexpected status code: {}".format(response.status))
     

class CreateCarResponse(Enum):
    Created = 201 # Car # Created.
    
    def __init__(self, response):
        match response.status:
            case 201:
                self.json = response.json()
                
            case _:
                raise Exception("unexpected status code: {}".format(response.status))
     


class TestBinaryResponse(Enum):
    Ok = 200 # bytearray # Ok.
    
    def __init__(self, response):
        match response.status:
            case 200:
                self.bytes = response.content
            case _:
                raise Exception("unexpected status code: {}".format(response.status))
     


class TestOneOfResponse(Enum):
    Ok = 200 # TestOneOfResponse # Ok.
    
    def __init__(self, response):
        match response.status:
            case 200:
                self.json = response.json()
                
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
        body,
        extra,
    ) -> CreateCarResponse:
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
        uri = "{}/spaces/{}/car-service/cars".format(
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

        return GetCarsResponse(resp)
    def create_car(
        self,
        space_id : str,
        body,
        extra,
    ) -> CreateCarResponse:
        uri = "{}/spaces/{}/car-service/cars".format(
            self.uri,
            space_id,
        )
        headers = {
            "Content-type": "application/json",
        }
        

        resp = requests.post(
            uri,
            headers = headers
        )

        return CreateCarResponse(resp)
    
    
    def test_binary(
        self,
        space_id : str,
        body,
        extra,
    ) -> TestBinaryResponse:
        uri = "{}/spaces/{}/car-service/test-binary".format(
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
    
    

