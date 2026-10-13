use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

use super::{StorageError, Store};

// Catch exceptions here and return only fixed classifications to Rust. Browser error
// objects can contain implementation-specific details; none of them includes save text.
#[wasm_bindgen(inline_js = r#"
const saveKey = 'low-kingdoms.active.v1';
function storage() {
    try {
        const value = window.localStorage;
        if (!value) throw 'unavailable';
        return value;
    } catch (_) {
        throw 'unavailable';
    }
}
export function checkSaveStorage() { storage(); }
export function readBattleSave() {
    const value = storage();
    try { return value.getItem(saveKey); }
    catch (_) { throw 'read'; }
}
export function writeBattleSave(data) {
    const value = storage();
    try { value.setItem(saveKey, data); }
    catch (error) {
        if (error?.name === 'QuotaExceededError' || error?.name === 'NS_ERROR_DOM_QUOTA_REACHED') {
            throw 'quota';
        }
        throw 'write';
    }
}
export function clearBattleSave() {
    const value = storage();
    try { value.removeItem(saveKey); }
    catch (_) { throw 'clear'; }
}
export function freshSeedBytes() {
    try {
        const bytes = new Uint8Array(8);
        globalThis.crypto.getRandomValues(bytes);
        return bytes;
    } catch (_) { throw 'entropy'; }
}
export function battleDocumentActive() {
    return document.visibilityState === 'visible' && document.hasFocus();
}
"#)]
extern "C" {
    #[wasm_bindgen(catch, js_name = checkSaveStorage)]
    fn check_storage() -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = readBattleSave)]
    fn read_save() -> Result<Option<String>, JsValue>;
    #[wasm_bindgen(catch, js_name = writeBattleSave)]
    fn write_save(data: &str) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = clearBattleSave)]
    fn clear_save() -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = freshSeedBytes)]
    fn seed_bytes() -> Result<Vec<u8>, JsValue>;
    #[wasm_bindgen(js_name = battleDocumentActive)]
    fn active_document() -> bool;
}

pub(super) fn default_store() -> Result<Box<dyn Store>, StorageError> {
    check_storage().map_err(|error| classify_error(&error))?;
    Ok(Box::new(BrowserStore))
}

struct BrowserStore;

impl Store for BrowserStore {
    fn read(&mut self) -> Result<Option<String>, StorageError> {
        read_save().map_err(|error| classify_error(&error))
    }

    fn write(&mut self, data: &str) -> Result<(), StorageError> {
        write_save(data).map_err(|error| classify_error(&error))
    }

    fn clear(&mut self) -> Result<(), StorageError> {
        clear_save().map_err(|error| classify_error(&error))
    }
}

fn classify_error(error: &JsValue) -> StorageError {
    match error.as_string().as_deref() {
        Some("quota") => StorageError::Quota,
        Some("read") => StorageError::Read("Browser storage rejected the read operation.".into()),
        Some("write") => {
            StorageError::Write("Browser storage rejected the write operation.".into())
        }
        Some("clear") => {
            StorageError::Clear("Browser storage rejected the clear operation.".into())
        }
        _ => StorageError::Unavailable("Allow local storage for this site and retry.".into()),
    }
}

pub(super) fn fresh_seed() -> Result<u64, StorageError> {
    let bytes = seed_bytes().map_err(|_| StorageError::Entropy)?;
    let Ok(bytes) = <[u8; 8]>::try_from(bytes.as_slice()) else {
        return Err(StorageError::Entropy);
    };
    Ok(u64::from_le_bytes(bytes))
}

pub(super) fn document_active() -> bool {
    active_document()
}
