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

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam and needs a signed-in account"]
async fn subscribing_round_trips() {
    // Skips rather than fails when there is no saved token: most machines
    // running these tests are anonymous, and an unrunnable test that reports
    // failure teaches people to ignore red.
    let mut session = Session::automatic(None).await.expect("session");
    let Some(account) = session.account().map(str::to_owned) else {
        println!("SKIPPED: anonymous session. Run `tapline login` to exercise this.");
        return;
    };
    println!("signed in as {account}");

    const APP: tapline::AppId = tapline::AppId(4000);
    const ITEM: tapline::PublishedFileId = tapline::PublishedFileId(104_691_717);

    // Subscribe, then unsubscribe, so the account is left as it was found.
    session
        .subscribe_workshop_item(APP, ITEM, false)
        .await
        .expect("subscribe");
    println!("subscribed to {ITEM}");

    session
        .unsubscribe_workshop_item(APP, ITEM)
        .await
        .expect("unsubscribe");
    println!("unsubscribed from {ITEM}");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn subscribing_anonymously_is_refused() {
    // The half that can be checked without an account: Steam does not silently
    // accept a subscription for nobody.
    let mut session = Session::anonymous().await.expect("session");
    let outcome = session
        .subscribe_workshop_item(
            tapline::AppId(4000),
            tapline::PublishedFileId(104_691_717),
            false,
        )
        .await;
    match outcome {
        Err(error) => println!("refused, as it should be: {error}"),
        Ok(()) => panic!("an anonymous session subscribed to something"),
    }
}
