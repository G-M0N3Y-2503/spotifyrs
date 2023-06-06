use utils::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_session_store() {
    type R = Result<Option<String>, WebStoreError>;
    let mut session = SessionStore::new();
    assert_eq!(session.get(&"key-1".to_string()), R::Ok(None));
    assert_eq!(
        session.insert("key-1".to_string(), "value-1".to_string()),
        R::Ok(None)
    );
    assert_eq!(
        session.get(&"key-1".to_string()),
        R::Ok(Some("value-1".to_string()))
    );
    assert_eq!(session.get(&"key-2".to_string()), R::Ok(None));
    assert_eq!(
        session.get(&"key-1".to_string()),
        R::Ok(Some("value-1".to_string()))
    );

    assert_eq!(
        session.insert("key-1".to_string(), "value-2".to_string()),
        R::Ok(Some("value-1".to_string()))
    );
    assert_eq!(
        session.get(&"key-1".to_string()),
        R::Ok(Some("value-2".to_string()))
    );
    assert_eq!(session.get(&"key-2".to_string()), R::Ok(None));
    assert_eq!(
        session.insert("key-1".to_string(), "value-3".to_string()),
        R::Ok(Some("value-2".to_string()))
    );
    assert_eq!(
        session.insert("key-1".to_string(), "value-4".to_string()),
        R::Ok(Some("value-3".to_string()))
    );

    assert_eq!(
        session.remove(&"key-1".to_string()),
        R::Ok(Some("value-4".to_string()))
    );
    assert_eq!(session.get(&"key-1".to_string()), R::Ok(None));
    assert_eq!(session.remove(&"key-1".to_string()), R::Ok(None));
}

#[wasm_bindgen_test]
fn test_local_store() {
    type R = Result<Option<String>, WebStoreError>;
    let mut local = LocalStore::new();
    assert_eq!(local.get(&"key-1".to_string()), R::Ok(None));
    assert_eq!(
        local.insert("key-1".to_string(), "value-1".to_string()),
        R::Ok(None)
    );
    assert_eq!(
        local.get(&"key-1".to_string()),
        R::Ok(Some("value-1".to_string()))
    );
    assert_eq!(local.get(&"key-2".to_string()), R::Ok(None));
    assert_eq!(
        local.get(&"key-1".to_string()),
        R::Ok(Some("value-1".to_string()))
    );

    assert_eq!(
        local.insert("key-1".to_string(), "value-2".to_string()),
        R::Ok(Some("value-1".to_string()))
    );
    assert_eq!(
        local.get(&"key-1".to_string()),
        R::Ok(Some("value-2".to_string()))
    );
    assert_eq!(local.get(&"key-2".to_string()), R::Ok(None));
    assert_eq!(
        local.insert("key-1".to_string(), "value-3".to_string()),
        R::Ok(Some("value-2".to_string()))
    );
    assert_eq!(
        local.insert("key-1".to_string(), "value-4".to_string()),
        R::Ok(Some("value-3".to_string()))
    );

    assert_eq!(
        local.remove(&"key-1".to_string()),
        R::Ok(Some("value-4".to_string()))
    );
    assert_eq!(local.get(&"key-1".to_string()), R::Ok(None));
    assert_eq!(local.remove(&"key-1".to_string()), R::Ok(None));
}

#[wasm_bindgen_test]
fn test_url() {
    let _url: Url = ::url::Url::parse("http://localhost")
        .expect("A valid URL")
        .try_into()
        .expect("A base URL");
    let _url: Url = ::url::Url::parse("http://localhost")
        .expect("A valid URL")
        .try_into()
        .expect("A base URL");
    let url = Url::new(::url::Url::parse("data:text/plain,Stuff").expect("A valid URL"))
        .expect_err("Not a base URL");
    console_log!("{}", url);
    let url =
        Url::new(::url::Url::parse("http://localhost").expect("A valid URL")).expect("A base URL");

    assert_eq!(url.origin().unicode_serialization(), "http://localhost");
    assert_eq!(url.with_path([""]).as_str(), "http://localhost/");
    assert_eq!(url.with_path(["1"]).as_str(), "http://localhost/1");
    assert_eq!(url.with_path(["1", "2"]).as_str(), "http://localhost/1/2");
}

#[wasm_bindgen_test]
async fn test_request() {
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    struct Response {
        #[serde(rename = "ip")]
        _ip: std::net::IpAddr,
    }

    let _res: Response = request(&reqwest::Client::new(), |client| {
        client.get("http://ip.jsontest.com/")
    })
    .await
    .expect("Webside is online and returned an IP");
}

#[wasm_bindgen_test]
async fn test_delay() {
    delay(std::time::Duration::from_secs(1)).await;
}

#[wasm_bindgen_test]
async fn test_delayed_fn() {
    wasm_logger::init(wasm_logger::Config::default());
    static CALLBACK_TRIGGERED: std::sync::RwLock<bool> = std::sync::RwLock::new(false);
    let callback = || {
        log::info!("callback triggered");
        *CALLBACK_TRIGGERED.write().unwrap() = true;
    };
    let _delayed_fn = new_delayed_fn!(callback, std::time::Duration::from_millis(16));
    assert!(!*CALLBACK_TRIGGERED.read().unwrap());
    delay(std::time::Duration::from_millis(20)).await;
    assert!(*CALLBACK_TRIGGERED.read().unwrap());
}

#[wasm_bindgen_test]
async fn test_tracked_delayed_fn() {
    wasm_logger::init(wasm_logger::Config::default());
    static CALLBACK_TRIGGERED: std::sync::RwLock<bool> = std::sync::RwLock::new(false);
    let callback = || {
        log::info!("callback triggered");
        *CALLBACK_TRIGGERED.write().unwrap() = true;
    };
    let mut delayed_fn = new_delayed_fn!(Tracked, callback, std::time::Duration::from_millis(30));
    assert!(matches!(delayed_fn.is_executed(), Some(false)));
    assert!(!delayed_fn.is_stopped());
    assert!(!*CALLBACK_TRIGGERED.read().unwrap());
    delay(std::time::Duration::from_millis(15)).await;
    assert!(matches!(delayed_fn.is_executed(), Some(false)));
    assert!(!delayed_fn.is_stopped());
    assert!(!*CALLBACK_TRIGGERED.read().unwrap());
    assert!(stop_delayed_fn!(delayed_fn));
    assert!(matches!(delayed_fn.is_executed(), Some(false)));
    assert!(delayed_fn.is_stopped());
    assert!(!*CALLBACK_TRIGGERED.read().unwrap());
    delay(std::time::Duration::from_millis(20)).await;
    assert!(matches!(delayed_fn.is_executed(), Some(false)));
    assert!(delayed_fn.is_stopped());
    assert!(!*CALLBACK_TRIGGERED.read().unwrap());
}
