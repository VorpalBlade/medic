use pretty_assertions::assert_eq;

use crate::medic;
use crate::Check;
use crate::CheckResult;

#[test]
fn test_medic() {
    let checks = [
        Check {
            name: "Check 1",
            func: || Ok((CheckResult::Ok, "All good".to_string())),
        },
        Check {
            name: "Check 2",
            func: || Ok((CheckResult::Warning, "Not so good\nNot at all".to_string())),
        },
        Check {
            name: "Check 3",
            func: || Ok((CheckResult::Fatal, "Very bad".to_string())),
        },
    ];
    // Get rid of formatting for ease of testing
    let mut out_buf = anstream::StripStream::new(Vec::new());

    let result = medic(&mut out_buf, checks.iter()).unwrap();
    assert_eq!(result, CheckResult::Fatal);

    let out_buf = out_buf.into_inner();
    let out = String::from_utf8(out_buf).unwrap();
    let expected = indoc::indoc! {"
        RESULT   CHECK    MESSAGE
        Ok       Check 1  All good
        Warning  Check 2  Not so good
                          Not at all
        Fatal    Check 3  Very bad\n"};
    assert_eq!(out, expected);
}
