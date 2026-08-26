//! The RPC bindings later milestones depend on must exist, and must pair the
//! right request with the right response.
//!
//! These assertions are cheap and they fail loudly. Without them, a schema
//! update that renamed or re-typed a method would compile fine here and surface
//! as a download that cannot find a CDN server, three milestones later.

use tapline_wire::Rpc;

/// Asserts a request type's target string and that its response type is what we
/// expect, by constructing the association at compile time.
macro_rules! assert_binding {
    ($request:ty => $response:ty, $target:literal) => {{
        assert_eq!(<$request as Rpc>::TARGET, $target);
        // Fails to compile if the generated Response type is not this one.
        fn _check(r: <$request as Rpc>::Response) -> $response {
            r
        }
    }};
}

#[test]
fn the_content_system_bindings_are_present() {
    use tapline_proto::steammessages_contentsystem_steamclient::*;

    // Without a manifest request code, no modern manifest can be fetched at all.
    assert_binding!(
        CContentServerDirectory_GetManifestRequestCode_Request
            => CContentServerDirectory_GetManifestRequestCode_Response,
        "ContentServerDirectory.GetManifestRequestCode"
    );

    // The CDN host list. Every host it returned on 2026-08-26 reported
    // "https_support":"mandatory", which is why rustls is a core dependency.
    assert_binding!(
        CContentServerDirectory_GetServersForSteamPipe_Request
            => CContentServerDirectory_GetServersForSteamPipe_Response,
        "ContentServerDirectory.GetServersForSteamPipe"
    );
}

#[test]
fn the_authentication_bindings_are_present() {
    use tapline_proto::steammessages_auth_steamclient::*;

    assert_binding!(
        CAuthentication_GetPasswordRSAPublicKey_Request
            => CAuthentication_GetPasswordRSAPublicKey_Response,
        "Authentication.GetPasswordRSAPublicKey"
    );
    assert_binding!(
        CAuthentication_BeginAuthSessionViaCredentials_Request
            => CAuthentication_BeginAuthSessionViaCredentials_Response,
        "Authentication.BeginAuthSessionViaCredentials"
    );
    assert_binding!(
        CAuthentication_BeginAuthSessionViaQR_Request
            => CAuthentication_BeginAuthSessionViaQR_Response,
        "Authentication.BeginAuthSessionViaQR"
    );
    assert_binding!(
        CAuthentication_PollAuthSessionStatus_Request
            => CAuthentication_PollAuthSessionStatus_Response,
        "Authentication.PollAuthSessionStatus"
    );
}

#[test]
fn the_workshop_binding_is_present() {
    use tapline_proto::steammessages_publishedfile_steamclient::*;

    // How a Workshop item's hcontent_file — the manifest id in the app's
    // workshop depot — is discovered.
    assert_binding!(
        CPublishedFile_GetDetails_Request => CPublishedFile_GetDetails_Response,
        "PublishedFile.GetDetails"
    );
}

#[test]
fn a_shared_request_type_still_exposes_its_other_target() {
    // Valve points both PublishedFile.GetUserFiles and .GetUserFileCount at the
    // same request type. Rpc is keyed on that type so only one can carry the
    // binding — the other is emitted as a constant rather than dropped.
    use tapline_proto::steammessages_publishedfile_steamclient::{
        CPublishedFile_GetUserFiles_Request, TARGET_PUBLISHED_FILE_GET_USER_FILE_COUNT,
    };

    assert_eq!(
        <CPublishedFile_GetUserFiles_Request as Rpc>::TARGET,
        "PublishedFile.GetUserFiles"
    );
    assert_eq!(
        TARGET_PUBLISHED_FILE_GET_USER_FILE_COUNT,
        "PublishedFile.GetUserFileCount"
    );
}
