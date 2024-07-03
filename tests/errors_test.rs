extern crate gouqi;

use gouqi::Error;
#[test]
fn test_error_display() {
    let error = Error::Unauthorized;
    assert_eq!(
        format!("{}", error),
        "Could not connect to Jira: Unauthorized\n"
    );
    let error = Error::MethodNotAllowed;
    assert_eq!(
        format!("{}", error),
        "Jira request error: MethodNotAllowed\n"
    );

    let error = Error::NotFound;
    assert_eq!(format!("{}", error), "Jira request error: NotFound\n");
}
