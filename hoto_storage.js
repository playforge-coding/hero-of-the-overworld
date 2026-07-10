// Web save-file backend for Hero of the Overworld.
//
// The native build writes save bytes to a file per slot; on the web we persist
// the exact same bytes in IndexedDB, one entry per slot. This registers a
// miniquad plugin exposing three functions the Rust `save::storage` module
// imports, each taking a slot index:
//
//   hoto_storage_save(slot, obj)  — store the byte buffer `obj` as slot's save.
//   hoto_storage_load(slot)       — return that slot's saved bytes (or nil).
//   hoto_storage_clear(slot)      — delete that slot's save.
//
// IndexedDB is asynchronous, but the Rust side loads synchronously at startup,
// so on init we eagerly read every slot into `cache`; `hoto_storage_load` then
// returns the cached buffer for the requested slot. Writes update the cache
// immediately and flush to IndexedDB in the background.
//
// Depends on sapp_jsutils.js (for `js_object` / `get_js_object`) and
// mq_js_bundle.js (for `miniquad_add_plugin`); load both before this file.
"use strict";

(function () {
    const DB_NAME = "hero-of-the-overworld";
    const STORE = "saves";
    // How many slots to preload. Must be >= the Rust `save::SLOTS`; a small
    // over-count is harmless (extra empty slots just cache nothing).
    const SLOTS = 8;

    // IndexedDB key for a slot's bytes. Slot 0 keeps the original "save" key so a
    // playthrough saved before slots existed still loads as slot 0.
    function keyFor(slot) {
        return slot === 0 ? "save" : "save-" + slot;
    }

    let db = null;
    let cache = []; // cache[slot] = Uint8Array of that slot's save, or undefined.

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

    // Preload every slot into `cache` so the synchronous Rust `load()` can see it.
    function preload() {
        openDb()
            .then(function (opened) {
                db = opened;
                const tx = db.transaction(STORE, "readonly");
                const store = tx.objectStore(STORE);
                for (let slot = 0; slot < SLOTS; slot++) {
                    (function (slot) {
                        const req = store.get(keyFor(slot));
                        req.onsuccess = function () {
                            if (req.result) {
                                cache[slot] = new Uint8Array(req.result);
                            }
                        };
                    })(slot);
                }
            })
            .catch(function (e) {
                console.warn("hoto save: could not open IndexedDB", e);
            });
    }

    function flush(slot, bytes) {
        const put = function () {
            try {
                const tx = db.transaction(STORE, "readwrite");
                tx.objectStore(STORE).put(bytes, keyFor(slot));
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

    function remove(slot) {
        cache[slot] = undefined;
        const del = function () {
            try {
                const tx = db.transaction(STORE, "readwrite");
                tx.objectStore(STORE).delete(keyFor(slot));
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
        importObject.env.hoto_storage_save = function (slot, obj_id) {
            // `get_js_object` returns the Uint8Array sapp_jsutils already copied
            // out of wasm memory; copy once more so it's independent of Rust's
            // JsObject lifetime.
            const src = get_js_object(obj_id);
            const bytes = new Uint8Array(src);
            cache[slot] = bytes;
            flush(slot, bytes);
        };
        importObject.env.hoto_storage_load = function (slot) {
            // Returns -1 (nil) when the slot is empty.
            return js_object(cache[slot] != null ? cache[slot] : null);
        };
        importObject.env.hoto_storage_clear = function (slot) {
            remove(slot);
        };
    }

    miniquad_add_plugin({
        register_plugin: register_plugin,
        on_init: preload,
        version: 1,
        name: "hoto_storage",
    });
})();
