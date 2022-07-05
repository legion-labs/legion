use serde::Deserialize;
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;

use crate::{errors::FromReqwestError, types::Request};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum AsyncData<T, E> {
    Init,
    Data(T),
    Error(E),
}

impl<T, E> Default for AsyncData<T, E> {
    fn default() -> Self {
        Self::Init
    }
}

pub fn create_async_data<'a, R>(
    cx: Scope<'a>,
    options: <R as Request>::Options,
) -> RcSignal<AsyncData<<R as Request>::Output, <R as Request>::Error>>
where
    R: Request,
    <R as Request>::Error: 'a + FromReqwestError,
    <R as Request>::Output: 'a + for<'de> Deserialize<'de>,
    <R as Request>::Options: 'a,
{
    let state = create_rc_signal(AsyncData::default());

    spawn_local_scoped(cx, {
        let state = state.clone();

        async move {
            let client = reqwest::Client::new();

            let req = R::request(options);

            let res = match client.execute(req).await {
                Ok(res) => res,
                Err(error) => {
                    state.set(AsyncData::Error(
                        <<R as Request>::Error>::from_reqwest_error(error),
                    ));

                    return;
                }
            };

            let data = match res.json::<<R as Request>::Output>().await {
                Ok(data) => data,
                Err(error) => {
                    state.set(AsyncData::Error(
                        <<R as Request>::Error>::from_reqwest_error(error),
                    ));

                    return;
                }
            };

            state.set(AsyncData::Data(data));
        }
    });

    state
}

#[allow(dead_code)]
pub fn create_async_data_<'a, R>(
    cx: Scope<'a>,
) -> RcSignal<AsyncData<<R as Request>::Output, <R as Request>::Error>>
where
    R: Request<Options = ()>,
    <R as Request>::Error: 'a + FromReqwestError,
    <R as Request>::Output: 'a + for<'de> Deserialize<'de>,
{
    create_async_data::<R>(cx, ())
}
