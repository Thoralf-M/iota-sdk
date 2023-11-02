// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import {
    customDatabase,
    testCustomDatabase,
    setCustomDatabase,
    getCustomDatabase,
    deleteCustomDatabase,
} from '@iota/sdk';

// Run with command:
// yarn run-example ./how_tos/client/test-db.ts

// In this example we will test a custom db
async function run() {
    try {
        let db = new Map<string, string>();

        const get_cb = function (err: any, key: string) {
            return db.get(key);
        };
        const set_cb = function (err: any, key: string, value: string) {
            return db.set(key, value);
        };
        const delete_cb = function (err: any, key: string) {
            return db.delete(key);
        };

        let js_db = customDatabase(get_cb, set_cb, delete_cb);

        await testCustomDatabase(js_db);
        console.log('Db in JS:', db);

        await setCustomDatabase(js_db, 'newKey', 'someValue');
        await setCustomDatabase(js_db, 'newKey1', 'someValue');
        // TODO: return actual string
        let r = await getCustomDatabase(js_db, 'newKey1');
        console.log('get result: ', r);
        console.log(db.get('newKey1'));
        await deleteCustomDatabase(js_db, 'newKey1');
        console.log(await getCustomDatabase(js_db, 'newKey1'));

        console.log('Db in JS:', Object.fromEntries(db));
    } catch (error) {
        console.error('Error: ', error);
    }
}

run().then(() => process.exit());
