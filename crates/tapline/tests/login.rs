#![allow(clippy::expect_used, clippy::unwrap_used)]

use tapline::{PollOutcome, Session};

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn a_qr_login_starts_and_polls() {
    let mut session = Session::anonymous().await.expect("session");

    let pending = session.begin_qr_login().await.expect("QR login must start");

    println!("client_id     {}", pending.client_id);
    println!("request_id    {} bytes", pending.request_id.len());
    println!("interval      {}s", pending.interval);
    println!("challenge_url {:?}", pending.challenge_url);
    println!("confirmations {:?}", pending.confirmations);
    println!("instruction   {}", pending.instruction());

    assert_ne!(pending.client_id, 0, "Steam issued no client id");
    assert!(!pending.request_id.is_empty(), "Steam issued no request id");

    let url = pending
        .challenge_url
        .as_deref()
        .expect("a QR login must carry a challenge URL");
    assert!(
        url.starts_with("https://"),
        "the challenge URL is not a URL: {url}"
    );

    assert!(
        (1.0..=60.0).contains(&pending.interval),
        "implausible poll interval {}",
        pending.interval
    );

    assert!(
        !pending.requires_a_code(),
        "a QR login was treated as requiring a typed code"
    );

    let outcome = session
        .poll_login(&pending)
        .await
        .expect("poll must answer");
    println!("poll: {outcome:?}");

    match outcome {
        PollOutcome::Pending { had_interaction } => {
            assert!(!had_interaction, "nobody scanned this code");
        }
        PollOutcome::Moved { .. } => {}
        PollOutcome::Complete { .. } => {
            panic!("a login completed that nobody approved");
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn steam_issues_a_real_rsa_key_for_an_account_name() {
    let mut session = Session::anonymous().await.expect("session");

    let key = session
        .password_key("gabelogannewell")
        .await
        .expect("Steam must issue a key for an existing account name");

    println!("key: {} bits, timestamp {}", key.bits(), key.timestamp);

    assert!(
        key.bits() >= 2048,
        "Steam issued a {}-bit key, which is weaker than expected",
        key.bits()
    );
    assert_ne!(
        key.timestamp, 0,
        "the key carried no timestamp to echo back"
    );

    let encrypted = tapline_auth::encrypt_password("not-a-real-password".to_owned(), &key)
        .expect("encryption must succeed against a real key");
    assert!(!encrypted.contains("not-a-real-password"));
    assert_eq!(
        encrypted.len(),
        344,
        "a 2048-bit RSA ciphertext should be 344 base64 characters"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn an_unknown_account_still_gets_a_key_rather_than_leaking_existence() {
    let mut session = Session::anonymous().await.expect("session");

    let result = session
        .password_key("tapline-no-such-account-9d3f8a2b1c")
        .await;

    match result {
        Ok(key) => {
            println!(
                "Steam issued a {}-bit key for a nonexistent account",
                key.bits()
            );
            assert!(key.bits() >= 1024);
        }
        Err(error) => {
            println!("Steam refused a nonexistent account: {error}");
        }
    }
}
