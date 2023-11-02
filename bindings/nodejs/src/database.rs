// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{fmt::Debug, str::from_utf8, sync::Arc};

use iota_sdk_bindings_core::iota_sdk::client::storage::StorageAdapter;
use neon::prelude::*;
use tokio::sync::RwLock;

type JsCallback = Root<JsFunction<JsObject>>;

/// A storage adapter that uses closures from the JS side.
pub struct NodeJsStorage {
    pub db_methods: Arc<RwLock<DatabaseMethods>>,
    pub channel: Channel,
}

impl Debug for NodeJsStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "node_js_store")
    }
}

impl Finalize for NodeJsStorage {}

pub struct DatabaseMethods {
    pub(crate) get: Arc<JsCallback>,
    pub(crate) set: Arc<JsCallback>,
    pub(crate) delete: Arc<JsCallback>,
}

#[async_trait::async_trait]
impl StorageAdapter for NodeJsStorage {
    type Error = iota_sdk_bindings_core::iota_sdk::wallet::Error;

    async fn get_bytes(&self, key: &str) -> iota_sdk_bindings_core::iota_sdk::wallet::Result<Option<Vec<u8>>> {
        let db_methods = self.db_methods.read().await;
        call_get_callback(&self.channel, db_methods.get.clone(), key.to_owned());
        Ok(None)
    }

    async fn set_bytes(&self, key: &str, record: &[u8]) -> iota_sdk_bindings_core::iota_sdk::wallet::Result<()> {
        let db_methods = self.db_methods.write().await;
        call_set_callback(
            &self.channel,
            db_methods.set.clone(),
            key.to_owned(),
            from_utf8(record).unwrap().to_owned(),
        );
        Ok(())
    }

    async fn delete(&self, key: &str) -> iota_sdk_bindings_core::iota_sdk::wallet::Result<()> {
        let db_methods = self.db_methods.write().await;
        call_delete_callback(&self.channel, db_methods.delete.clone(), key.to_owned());
        Ok(())
    }
}

fn call_get_callback(channel: &neon::event::Channel, callback: Arc<JsCallback>, key: String) {
    channel.send(move |mut cx| {
        let cb = (*callback).to_inner(&mut cx);
        let this = cx.undefined();
        let args = [cx.undefined().upcast::<JsValue>(), cx.string(key).upcast::<JsValue>()];

        cb.call(&mut cx, this, args)?;
        Ok(())
    });
}

fn call_set_callback(channel: &neon::event::Channel, callback: Arc<JsCallback>, key: String, data: String) {
    channel
        .send(move |mut cx| {
            let cb = (*callback).to_inner(&mut cx);

            let this = cx.undefined();

            let args = [
                cx.undefined().upcast::<JsValue>(),
                cx.string(key).upcast::<JsValue>(),
                cx.string(data).upcast::<JsValue>(),
            ];

            cb.call(&mut cx, this, args)?;
            Ok(())
        })
        .join()
        .unwrap();
}

fn call_delete_callback(channel: &neon::event::Channel, callback: Arc<JsCallback>, key: String) {
    channel.send(move |mut cx| {
        let cb = (*callback).to_inner(&mut cx);
        let this = cx.undefined();
        let args = [cx.undefined().upcast::<JsValue>(), cx.string(key).upcast::<JsValue>()];

        cb.call(&mut cx, this, args)?;
        Ok(())
    });
}

pub fn custom_database(mut cx: FunctionContext) -> JsResult<JsBox<Arc<NodeJsStorage>>> {
    let get_bytes_callback = Arc::new(cx.argument::<JsFunction>(0)?.root(&mut cx));
    let set_bytes_callback = Arc::new(cx.argument::<JsFunction>(1)?.root(&mut cx));
    let delete_bytes_callback = Arc::new(cx.argument::<JsFunction>(2)?.root(&mut cx));

    let channel = cx.channel();

    let db_methods = DatabaseMethods {
        get: get_bytes_callback,
        set: set_bytes_callback,
        delete: delete_bytes_callback,
    };

    let storage = NodeJsStorage {
        db_methods: Arc::new(RwLock::new(db_methods)),
        channel,
    };

    Ok(cx.boxed(Arc::new(storage)))
}

pub fn set_custom_database(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let db = Arc::clone(&cx.argument::<JsBox<Arc<NodeJsStorage>>>(0).unwrap());
    let key = cx.argument::<JsString>(1)?.value(&mut cx);
    let value = cx.argument::<JsString>(2)?.value(&mut cx);

    let (deferred, promise) = cx.promise();
    crate::RUNTIME.spawn(async move {
        db.set_bytes(&key, value.as_bytes()).await.unwrap();

        deferred.settle_with(&db.channel, move |mut cx| Ok(cx.boxed(())));
    });
    Ok(promise)
}

pub fn get_custom_database(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let db = Arc::clone(&cx.argument::<JsBox<Arc<NodeJsStorage>>>(0).unwrap());
    let key = cx.argument::<JsString>(1)?.value(&mut cx);

    let (deferred, promise) = cx.promise();
    crate::RUNTIME.spawn(async move {
        let v = db.get_bytes(&key).await.unwrap();

        if let Some(v) = v {
            deferred.settle_with(&db.channel, move |mut cx| {
                Ok(cx.string(from_utf8(&v).unwrap().to_owned()))
            });
        } else {
            deferred.settle_with(&db.channel, move |mut cx| Ok(cx.boxed(())));
        }
    });
    Ok(promise)
}

pub fn delete_custom_database(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let db = Arc::clone(&cx.argument::<JsBox<Arc<NodeJsStorage>>>(0).unwrap());
    let key = cx.argument::<JsString>(1)?.value(&mut cx);

    let (deferred, promise) = cx.promise();
    crate::RUNTIME.spawn(async move {
        db.delete(&key).await.unwrap();

        deferred.settle_with(&db.channel, move |mut cx| Ok(cx.boxed(())));
    });
    Ok(promise)
}

pub fn test_custom_database(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let db = Arc::clone(&cx.argument::<JsBox<Arc<NodeJsStorage>>>(0).unwrap());

    let (deferred, promise) = cx.promise();
    crate::RUNTIME.spawn(async move {
        db.set_bytes("testKey", b"testValue").await.unwrap();

        let _v = db.get_bytes("testKey").await.unwrap();

        db.delete("testKey").await.unwrap();

        let _v = db.get_bytes("testKey").await.unwrap();

        db.set_bytes("testKey", b"finalValue").await.unwrap();

        deferred.settle_with(&db.channel, move |mut cx| Ok(cx.boxed(())));
    });
    Ok(promise)
}
