# ---------- Models ----------
from enum import Enum




# The car color.
class CarColor(Enum):

    RED = auto()

    BLUE = auto()

    YELLOW = auto()







class Car:
    def __init__(self):
    
        
        self.id = None # int64
    
        
        self.name = None # str
    
        # The car color.
        self.color = None # class CarColor
    
        
        self.is_new = None # bool
    
        
        self.extra = None # bytearray
    
            





class Pet:
    def __init__(self):
    
        
        self.name = None # str
    
            





class TestOneOfResponse(Enum):

    OPTION_1 # class Pet

    OPTION_2 # class Car

                

