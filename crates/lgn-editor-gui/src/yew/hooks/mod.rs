use std::rc::Rc;

use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

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

#[hook]
pub fn use_async_data<R>(
    options: <R as Request>::Options,
) -> UseStateHandle<AsyncData<Rc<<R as Request>::Output>, <R as Request>::Error>>
where
    R: 'static + Request,
    <R as Request>::Error: FromReqwestError,
    <R as Request>::Output: for<'de> Deserialize<'de>,
{
    let state = use_state(AsyncData::default);

    {
        let state = state.clone();

        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
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
                        Ok(data) => Rc::new(data),
                        Err(error) => {
                            state.set(AsyncData::Error(
                                <<R as Request>::Error>::from_reqwest_error(error),
                            ));

                            return;
                        }
                    };

                    state.set(AsyncData::Data(data));
                });

                move || {}
            },
            (),
        );
    }

    state
}

#[hook]
pub fn use_async_data_<R>(
) -> UseStateHandle<AsyncData<Rc<<R as Request>::Output>, <R as Request>::Error>>
where
    R: 'static + Request<Options = ()>,
    <R as Request>::Error: FromReqwestError,
    <R as Request>::Output: for<'de> Deserialize<'de>,
{
    use_async_data::<R>(())
}
