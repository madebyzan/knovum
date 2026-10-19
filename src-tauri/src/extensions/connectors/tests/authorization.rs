use super::{callback_page, denial, AUTHORIZATION_DEADLINE};

#[test]
fn a_provider_refusal_is_reported_as_a_denial() {
    assert_eq!(
        denial("error=access_denied&error_description=The+user+declined"),
        Some("The user declined".into()),
        "the provider's own wording says more than the error code"
    );
    assert_eq!(
        denial("error=access_denied"),
        Some("access_denied".into()),
        "a refusal without a description is still a refusal"
    );
}

#[test]
fn a_successful_callback_is_not_a_denial() {
    assert_eq!(denial("code=authorization-code&state=opaque"), None);
    assert_eq!(denial(""), None);
    assert_eq!(
        denial("error_description=stale+description"),
        None,
        "only an error parameter refuses an authorization"
    );
}

#[test]
fn the_callback_page_states_the_outcome_and_closes_itself() {
    let complete = callback_page(None).0;
    assert!(complete.contains("Authorization complete"));
    assert!(complete.contains("window.close()"));
    assert!(complete.contains("return to Knovum"));
    let declined = callback_page(Some("The user declined".into())).0;
    assert!(declined.contains("Authorization was declined"));
    assert!(declined.contains("The user declined"));
}

#[test]
fn the_browser_half_is_given_time_for_a_real_sign_in() {
    assert!(
        AUTHORIZATION_DEADLINE.as_secs() >= 600,
        "signing in, approving a workspace and a second factor outlast a couple of minutes"
    );
}
