//! Signing in, or deciding not to.
//!
//! ```sh
//! cargo test --release -p tapline --test auth -- --ignored --nocapture
//! ```

#![allow(clippy::expect_used, clippy::unwrap_used)]

use tapline::Session;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn automatic_signs_in_when_it_can_and_carries_on_when_it_cannot() {
    // The contract: never fails because of authentication. A machine with a
    // saved token gets a signed-in session, a machine without gets an anonymous
    // one, and neither is an error — a download that never needed an account
    // must not stop because a token expired.
    let session = Session::automatic(None)
        .await
        .expect("a session either way");

    match session.account() {
        Some(account) => println!("signed in as {account}"),
        None => println!("anonymous"),
    }
    assert!(session.cell_id() > 0, "a session should land in a cell");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn a_rubbish_token_falls_back_rather_than_failing() {
    // A saved token that Steam refuses is the normal end of a token's life.
    // The session should come back anonymous instead of leaving the caller
    // holding an error about credentials it never asked to use.
    let token = tapline_auth::StoredToken {
        account: "nobody-at-all".to_owned(),
        refresh_token: "not.a.token".to_owned(),
    };
    let direct = Session::with_token(&token).await;
    assert!(direct.is_err(), "a rubbish token should not log on");

    // And the same token reached through `automatic` is survivable.
    let session = Session::automatic(Some("nobody-at-all"))
        .await
        .expect("automatic should fall back to anonymous");
    assert_eq!(
        session.account(),
        None,
        "a refused token should leave an anonymous session, not a signed-in one"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn an_owned_depot_says_it_needs_a_login() {
    // The message a person acts on. Counter-Strike 2's client depots are not
    // anonymously accessible, so this is the refusal every unowned app gives.
    let mut session = Session::anonymous().await.expect("session");
    let dir = std::env::temp_dir().join("tapline-auth-denied");
    let options = tapline::InstallOptions {
        install_dir: dir.clone(),
        ..tapline::InstallOptions::default()
    };

    match session.install(tapline::AppId(730), &options).await {
        Ok(_) => println!("730 installed anonymously; nothing to assert"),
        Err(error) => {
            println!("{error}");
            assert!(
                error.needs_login(),
                "an access-denied depot should report as needing a login"
            );
            assert!(
                error.to_string().contains("tapline login"),
                "the message should name the fix: {error}"
            );
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}
