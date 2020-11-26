use backtrace::Backtrace;
use futures::{Future, FutureExt};
use iota::Client;
use neon::prelude::*;
use once_cell::sync::Lazy;
use rand::{distributions::Alphanumeric, thread_rng, Rng};

use std::{
    any::Any,
    collections::HashMap,
    fmt,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::{Arc, RwLock},
};

mod classes;
use classes::*;

type ClientInstanceMap = Arc<RwLock<HashMap<String, Arc<RwLock<Client>>>>>;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub(crate) enum Error {
    AnyhowError(anyhow::Error),
    ClientError(iota::client::Error),
    Panic(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::AnyhowError(e) => e.fmt(f),
            Error::Panic(message) => write!(f, "Panic: {}", message),
            Error::ClientError(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Error::AnyhowError(error)
    }
}

impl From<iota::client::Error> for Error {
    fn from(error: iota::client::Error) -> Self {
        Error::ClientError(error)
    }
}

/// Gets the client instances map.
fn instances() -> &'static ClientInstanceMap {
    static INSTANCES: Lazy<ClientInstanceMap> = Lazy::new(Default::default);
    &INSTANCES
}

pub(crate) fn get_client(id: String) -> Arc<RwLock<Client>> {
    let map = instances()
        .read()
        .expect("failed to lock client instances: get_client()");
    map.get(&id)
        .expect("client dropped or not initialised")
        .clone()
}

pub(crate) fn store_client(client: Client) -> String {
    let mut map = instances()
        .write()
        .expect("failed to lock client instances: get_client()");
    let id: String = thread_rng().sample_iter(&Alphanumeric).take(10).collect();
    map.insert(id.clone(), Arc::new(RwLock::new(client)));
    id
}

pub(crate) fn remove_client(id: String) {
    let mut map = instances()
        .write()
        .expect("failed to lock client instances: get_client()");
    map.remove(&id);
}

fn panic_to_response_message(panic: Box<dyn Any>) -> String {
    let msg = if let Some(message) = panic.downcast_ref::<String>() {
        format!("Internal error: {}", message)
    } else if let Some(message) = panic.downcast_ref::<&str>() {
        format!("Internal error: {}", message)
    } else {
        "Internal error".to_string()
    };
    let current_backtrace = Backtrace::new();
    format!("{}\n\n{:?}", msg, current_backtrace)
}

pub(crate) async fn convert_async_panics<T, F: Future<Output = Result<T>>>(
    f: impl FnOnce() -> F,
) -> Result<T> {
    match AssertUnwindSafe(f()).catch_unwind().await {
        Ok(result) => result,
        Err(panic) => Err(Error::Panic(panic_to_response_message(panic))),
    }
}

pub(crate) fn convert_panics<T, F: FnOnce() -> Result<T>>(f: F) -> Result<T> {
    match catch_unwind(AssertUnwindSafe(|| f())) {
        Ok(result) => result,
        Err(panic) => Err(Error::Panic(panic_to_response_message(panic))),
    }
}

register_module!(mut cx, {
    cx.export_class::<JsClient>("Client")?;
    cx.export_class::<JsTopicSubscriber>("TopicSubscriber")?;
    Ok(())
});
