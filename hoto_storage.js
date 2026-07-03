// Web save-file backend for Hero of the Overworld.
//
// The native build writes save bytes to a file; on the web we persist the exact
// same bytes in IndexedDB. This registers a miniquad plugin exposing three
// functions the Rust `save::storage` module imports:
//
//   hoto_storage_save(obj)  — store the byte buffer `obj` as the save.
//   hoto_storage_load()     — return the saved byte buffer (or nil).
//   hoto_storage_clear()    — delete the save.
//
// IndexedDB is asynchronous, but the Rust side loads synchronously at startup,
// so on init we eagerly read the save into `cache`; `hoto_storage_load` then
// returns that cached buffer. Writes update the cache immediately and flush to
// IndexedDB in the background.
//
// Depends on sapp_jsutils.js (for `js_object` / `get_js_object`) and
// mq_js_bundle.js (for `miniquad_add_plugin`); load both before this file.
"use strict";

(function () {
    const DB_NAME = "hero-of-the-overworld";
    const STORE = "saves";
    const KEY = "save";

    let db = null;
    let cache = null; // Uint8Array of the current save, or null if none.

    function openDb() {
        return new Promise(function (resolve, reject) {
            const req = indexedDB.open(DB_NAME, 1);
            req.onupgradeneeded = function () {
                req.result.createObjectStore(STORE);
            };
            req.onsuccess = function () {
                resolve(req.result);
            };
            req.onerror = function () {
                reject(req.error);
            };
        });
    }

    // Preload the save into `cache` so the synchronous Rust `load()` can see it.
    function preload() {
        openDb()
            .then(function (opened) {
                db = opened;
                const tx = db.transaction(STORE, "readonly");
                const req = tx.objectStore(STORE).get(KEY);
                req.onsuccess = function () {
                    if (req.result) {
                        cache = new Uint8Array(req.result);
                    }
                };
            })
            .catch(function (e) {
                console.warn("hoto save: could not open IndexedDB", e);
            });
    }

    function flush(bytes) {
        const put = function () {
            try {
                const tx = db.transaction(STORE, "readwrite");
                tx.objectStore(STORE).put(bytes, KEY);
            } catch (e) {
                console.warn("hoto save: write failed", e);
            }
        };
        if (db) {
            put();
        } else {
            openDb()
                .then(function (opened) {
                    db = opened;
                    put();
                })
                .catch(function (e) {
                    console.warn("hoto save: could not open IndexedDB", e);
                });
        }
    }

    function remove() {
        cache = null;
        const del = function () {
            try {
                const tx = db.transaction(STORE, "readwrite");
                tx.objectStore(STORE).delete(KEY);
            } catch (e) {
                console.warn("hoto save: delete failed", e);
            }
        };
        if (db) {
            del();
        } else {
            openDb()
                .then(function (opened) {
                    db = opened;
                    del();
                })
                .catch(function () {});
        }
    }

    function register_plugin(importObject) {
        importObject.env.hoto_storage_save = function (obj_id) {
            // `get_js_object` returns the Uint8Array sapp_jsutils already copied
            // out of wasm memory; copy once more so it's independent of Rust's
            // JsObject lifetime.
            const src = get_js_object(obj_id);
            const bytes = new Uint8Array(src);
            cache = bytes;
            flush(bytes);
        };
        importObject.env.hoto_storage_load = function () {
            // Returns -1 (nil) when cache is null.
            return js_object(cache);
        };
        importObject.env.hoto_storage_clear = function () {
            remove();
        };
    }

    miniquad_add_plugin({
        register_plugin: register_plugin,
        on_init: preload,
        version: 1,
        name: "hoto_storage",
    });
})();
