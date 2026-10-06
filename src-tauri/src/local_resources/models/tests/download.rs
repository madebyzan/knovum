use super::*;

#[test]
fn progress_and_completion_obey_the_download_protocol() {
    let (sender, mut receiver) = mpsc::channel(16);
    let mut complete = false;
    observe(
        r#"{"event":"progress","downloaded":5,"total":10}"#,
        10,
        &mut complete,
        &sender,
    )
    .unwrap();
    assert_eq!(receiver.try_recv().unwrap().downloaded, 5);
    observe(r#"{"event":"complete"}"#, 10, &mut complete, &sender).unwrap();
    assert!(complete);
    assert!(observe(r#"{"event":"complete"}"#, 10, &mut complete, &sender).is_err());
}

#[test]
fn invalid_or_failed_download_cannot_be_published() {
    let (sender, _) = mpsc::channel(16);
    for line in [
        r#"{"event":"progress","downloaded":11,"total":10}"#,
        r#"{"event":"progress","downloaded":1,"total":20}"#,
        r#"{"event":"complete","extra":true}"#,
        r#"{"event":"error","message":"download failed"}"#,
        "unexpected SDK output",
    ] {
        assert!(observe(line, 10, &mut false, &sender).is_err(), "{line}");
    }
}

#[test]
fn observer_disconnection_does_not_abort_the_shared_download() {
    let (sender, receiver) = mpsc::channel(1);
    drop(receiver);
    observe(
        r#"{"event":"progress","downloaded":5,"total":10}"#,
        10,
        &mut false,
        &sender,
    )
    .unwrap();
}
