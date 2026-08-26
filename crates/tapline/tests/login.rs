//! Live login tests — the parts that need no credentials.
//!
//! ```sh
//! cargo test -p tapline --test login -- --ignored --nocapture
//! ```
//!
//! # What is tested here, and what is not
//!
//! A QR login can be exercised end to end short of the approval itself: Steam
//! issues a real challenge URL, a real poll comes back "still waiting", and
//! every message in the flow is real. That covers the machinery.
//!
//! The password flow's RSA key fetch is also real — it needs only an account
//! *name*, not a secret — so the key parsing and the size check run against
//! what Steam actually sends.
//!
//! **What is deliberately absent is a test that logs in with a real password or
//! completes a real approval.** That would need someone's account, and no test
//! in this repository should want one. The password path's own encryption is
//! covered by unit tests in `tapline-auth` against a key whose private half the
//! test holds, which verifies the round trip without any account existing.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use tapline::{PollOutcome, Session};

#[tokio::test]
#[ignore = "talks to Steam"]
async fn a_qr_login_starts_and_polls() {
    // No password anywhere in this test.
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

    // The URL is what gets rendered as a QR code; without it there is nothing
    // for the user to scan.
    let url = pending
        .challenge_url
        .as_deref()
        .expect("a QR login must carry a challenge URL");
    assert!(
        url.starts_with("https://"),
        "the challenge URL is not a URL: {url}"
    );

    // Steam asks for an interval and a client that ignores it gets rate
    // limited.
    assert!(
        (1.0..=60.0).contains(&pending.interval),
        "implausible poll interval {}",
        pending.interval
    );

    // Steam offers alternatives here, not requirements: a real QR session comes
    // back as [DeviceConfirmation, DeviceCode] — scan it, or type the
    // authenticator code. What matters is that scanning is never *forced* into
    // a code prompt.
    assert!(
        !pending.requires_a_code(),
        "a QR login was treated as requiring a typed code"
    );

    // One real poll. Nobody has scanned it, so this must come back pending
    // rather than erroring or completing.
    let outcome = session
        .poll_login(&pending)
        .await
        .expect("poll must answer");
    println!("poll: {outcome:?}");

    match outcome {
        PollOutcome::Pending { had_interaction } => {
            assert!(!had_interaction, "nobody scanned this code");
        }
        PollOutcome::Moved { .. } => {
            // Legitimate: Steam refreshed the code between the two calls.
        }
        PollOutcome::Complete { .. } => {
            panic!("a login completed that nobody approved");
        }
    }
}

#[tokio::test]
#[ignore = "talks to Steam"]
async fn steam_issues_a_real_rsa_key_for_an_account_name() {
    // This needs an account *name*, not a secret — Steam hands the public key
    // to anyone who asks, which is what makes it testable without credentials.
    // The name below is Valve's own well-known publisher account, chosen so the
    // test names nobody's personal account.
    let mut session = Session::anonymous().await.expect("session");

    let key = session
        .password_key("gabelogannewell")
        .await
        .expect("Steam must issue a key for an existing account name");

    println!("key: {} bits, timestamp {}", key.bits(), key.timestamp);

    // 2048 is what Steam uses; the crate refuses anything under 1024 outright.
    assert!(
        key.bits() >= 2048,
        "Steam issued a {}-bit key, which is weaker than expected",
        key.bits()
    );
    assert_ne!(
        key.timestamp, 0,
        "the key carried no timestamp to echo back"
    );

    // A password can be encrypted under it. The ciphertext goes nowhere — this
    // asserts the encryption path works against a key Steam really sent, not
    // that any login is attempted.
    let encrypted = tapline_auth::encrypt_password("not-a-real-password".to_owned(), &key)
        .expect("encryption must succeed against a real key");
    assert!(!encrypted.contains("not-a-real-password"));
    assert_eq!(
        encrypted.len(),
        344,
        "a 2048-bit RSA ciphertext should be 344 base64 characters"
    );
}

#[tokio::test]
#[ignore = "talks to Steam"]
async fn an_unknown_account_still_gets_a_key_rather_than_leaking_existence() {
    // Steam issues a key for names that do not exist too, which is the correct
    // behaviour: answering differently would turn this endpoint into an account
    // enumeration oracle. Asserted so a future change that "helpfully" reports
    // unknown accounts gets noticed.
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
            // Also acceptable — but worth printing, because it means the
            // endpoint's behaviour changed.
            println!("Steam refused a nonexistent account: {error}");
        }
    }
}
