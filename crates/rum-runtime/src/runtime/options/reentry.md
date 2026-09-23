The compile-pass control uses the same imports, generic descriptor, and call
as the missing-descriptor proof. The required unit test compiles the control
first and checks the rejected call's actual diagnostic code is `E0061`.

```no_run
use rum_runtime::kernel_api::{OptionReentry, Session};
fn consume<R: OptionReentry>(session: &mut Session, descriptor: R) {
    let head = session.eval_request("f").unwrap().value;
    let _ = session.merged_option_exprs_for_system_symbol::<R>(descriptor, &head, &[], &[]);
}
fn main() {}
```

Omitting the re-entry cannot compile:

```compile_fail,E0061
use rum_runtime::kernel_api::{OptionReentry, Session};
fn consume<R: OptionReentry>(session: &mut Session, descriptor: R) {
    let head = session.eval_request("f").unwrap().value;
    let _ = session.merged_option_exprs_for_system_symbol::<R>(&head, &[], &[]);
}
fn main() {}
```
