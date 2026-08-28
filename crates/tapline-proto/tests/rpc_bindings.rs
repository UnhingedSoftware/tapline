use tapline_wire::Rpc;

macro_rules! assert_binding {
    ($request:ty => $response:ty, $target:literal) => {{
        assert_eq!(<$request as Rpc>::TARGET, $target);
        fn _check(r: <$request as Rpc>::Response) -> $response {
            r
        }
    }};
}

#[test]
fn the_content_system_bindings_are_present() {
    use tapline_proto::steammessages_contentsystem_steamclient::*;

    assert_binding!(
        CContentServerDirectory_GetManifestRequestCode_Request
            => CContentServerDirectory_GetManifestRequestCode_Response,
        "ContentServerDirectory.GetManifestRequestCode"
    );

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

    assert_binding!(
        CPublishedFile_GetDetails_Request => CPublishedFile_GetDetails_Response,
        "PublishedFile.GetDetails"
    );
}

#[test]
fn a_shared_request_type_still_exposes_its_other_target() {
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
