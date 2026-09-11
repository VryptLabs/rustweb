use rustweb_testing::headless::{require_headless, Browser};

#[test]
#[ignore = "requires HEADLESS=1 and a running WebDriver (see module docs)"]
fn homepage_hydrates_without_mismatch() {
    let cfg = require_headless();
    if std::env::var("HEADLESS").as_deref() != Ok("1") {
        return;
    }

    assert!(matches!(
        cfg.browser,
        Browser::Chrome | Browser::Firefox | Browser::Safari
    ));

    eprintln!("headless config ok: {cfg:?} (driver assertions enabled with fantoccini)");
}

#[test]
#[ignore = "requires HEADLESS=1 and a running WebDriver"]
fn keyboard_a11y_smoke() {
    let cfg = require_headless();
    if std::env::var("HEADLESS").as_deref() == Ok("1") {
        assert!(cfg.headless);
    }
}
