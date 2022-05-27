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
    def __init__(self):
        self.OPTION_1 = None # class Pet
        self.OPTION_2 = None # class Car

# ---------- Parameters ----------
class GetCarsQuery:
    def __init__(self):
        self.names = None # [string]
        self.q = None # Option<String>

# ---------- Responses -----------

    pub enum GetCarsResponse {
        /// List of cars.
        Ok(Vec<crate::models::Car>),
    }

    impl GetCarsResponse {
        pub(crate) fn into_response(self) -> Response {
            match self {
                GetCarsResponse::Ok(inner) => {
                    let inner = Json(inner);
                    (StatusCode::from_u16(200).unwrap(), inner).into_response()
                }
            }
        }

        pub(crate) async fn from_reqwest(response: reqwest::Response) -> Result<Self> {
            match response.status().as_u16() {
                200 => Ok(Self::Ok(response.json().await?)),
                status => Err(Error::Internal(format!(
                    "unexpected status code: {}",
                    status
                ))),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub enum CreateCarResponse {
        /// Created.
        Created(crate::models::Car),
    }

    impl CreateCarResponse {
        pub(crate) fn into_response(self) -> Response {
            match self {
                CreateCarResponse::Created(inner) => {
                    let inner = Json(inner);
                    (StatusCode::from_u16(201).unwrap(), inner).into_response()
                }
            }
        }

        pub(crate) async fn from_reqwest(response: reqwest::Response) -> Result<Self> {
            match response.status().as_u16() {
                201 => Ok(Self::Created(response.json().await?)),
                status => Err(Error::Internal(format!(
                    "unexpected status code: {}",
                    status
                ))),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub enum TestBinaryResponse {
        /// Ok.
        Ok(ByteArray),
    }

    impl TestBinaryResponse {
        pub(crate) fn into_response(self) -> Response {
            match self {
                TestBinaryResponse::Ok(inner) => {
                    let inner: Vec<u8> = inner.into();
                    (StatusCode::from_u16(200).unwrap(), inner).into_response()
                }
            }
        }

        pub(crate) async fn from_reqwest(response: reqwest::Response) -> Result<Self> {
            match response.status().as_u16() {
                200 => Ok(Self::Ok(response.bytes().await?.into())),
                status => Err(Error::Internal(format!(
                    "unexpected status code: {}",
                    status
                ))),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub enum TestOneOfResponse {
        /// Ok.
        Ok(crate::models::TestOneOfResponse),
    }

    impl TestOneOfResponse {
        pub(crate) fn into_response(self) -> Response {
            match self {
                TestOneOfResponse::Ok(inner) => {
                    let inner = Json(inner);
                    (StatusCode::from_u16(200).unwrap(), inner).into_response()
                }
            }
        }

        pub(crate) async fn from_reqwest(response: reqwest::Response) -> Result<Self> {
            match response.status().as_u16() {
                200 => Ok(Self::Ok(response.json().await?)),
                status => Err(Error::Internal(format!(
                    "unexpected status code: {}",
                    status
                ))),
            }
        }
    }
