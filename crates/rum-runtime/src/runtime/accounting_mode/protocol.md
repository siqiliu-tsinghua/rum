The required unit test builds each control before its rejected counterpart,
saves complete compiler output, and pins the sole error code. These examples
exercise the public API, without privileged access to protocol internals.

### missing-root-traversal

```no_run
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor, FrozenValues};
struct Untraceable;
#[derive(FrameRootTrace)]
struct Frame { prefix: FrozenValues, extra: usize }
fn main() {}
```

```compile_fail,E0277
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor, FrozenValues};
struct Untraceable;
#[derive(FrameRootTrace)]
struct Frame { prefix: FrozenValues, extra: Untraceable }
fn main() {}
```

### private-constructor

```no_run
use rum_runtime::kernel_api::{Session, RetentionLease};
fn main() {
    let mut session = Session::new();
    session.with_unarmed_retention(|cx, values| cx.finish_list(values)).unwrap();
}
```

```compile_fail,E0624
use rum_runtime::kernel_api::{Session, RetentionLease};
fn main() {
    let mut session = Session::new();
    let _lease = RetentionLease::new(&mut session);
}
```

### moved-value

```no_run
use rum_runtime::kernel_api::Session;
fn main() {
    Session::new().with_unarmed_retention(|cx, values| {
        let result = cx.finish_list(values)?;
        Ok(result)
    }).unwrap();
}
```

```compile_fail,E0382
use rum_runtime::kernel_api::Session;
fn main() {
    Session::new().with_unarmed_retention(|cx, values| {
        let result = cx.finish_list(values)?;
        let _reused = values;
        Ok(result)
    }).unwrap();
}
```

### raw-extraction

```no_run
use rum_runtime::kernel_api::Session;
fn main() {
    Session::new().with_unarmed_retention(|cx, values| cx.finish_list(values)).unwrap();
}
```

```compile_fail,E0616
use rum_runtime::kernel_api::Session;
fn main() {
    Session::new().with_unarmed_retention(|cx, values| {
        let _raw = values.values;
        cx.finish_list(values)
    }).unwrap();
}
```

### brand-escape

```no_run
use rum_runtime::kernel_api::{Session, EvalError};
fn main() {
    let mut entered = false;
    let _ = Session::new().with_unarmed_retention(|_cx, _values| {
        entered = true;
        Err(EvalError::Abort)
    });
}
```

```compile_fail,E0521
use rum_runtime::kernel_api::{Session, EvalError};
fn main() {
    let mut escaped = None;
    let _ = Session::new().with_unarmed_retention(|_cx, values| {
        escaped = Some(values);
        Err(EvalError::Abort)
    });
}
```

### adoption-before-next-step

```no_run
use rum_runtime::kernel_api::{Session, system_expr, CallbackValue};
fn main() {
    Session::new().with_unarmed_retention(|cx, values| {
        let value = cx.evaluate(values, system_expr("Null"));
        let (cx, values) = match value.and_then(CallbackValue::adopt) {
            Ok(next) => next,
            Err(failure) => return Ok(failure.finish()),
        };
        match cx.evaluate(values, system_expr("True")).and_then(CallbackValue::adopt) {
            Ok((cx, values)) => cx.finish_list(values),
            Err(failure) => Ok(failure.finish()),
        }
    }).unwrap();
}
```

```compile_fail,E0382
use rum_runtime::kernel_api::{Session, system_expr, CallbackValue};
fn main() {
    Session::new().with_unarmed_retention(|cx, values| {
        let value = cx.evaluate(values, system_expr("Null"));
        let _next_step = cx;
        match value.and_then(CallbackValue::adopt) {
            Ok((cx, values)) => cx.finish_list(values),
            Err(failure) => Ok(failure.finish()),
        }
    }).unwrap();
}
```

### error-extraction

```no_run
use rum_runtime::kernel_api::{Session, system_expr, CallbackValue};
fn main() {
    Session::new().with_unarmed_retention(|cx, values| {
        match cx.evaluate(values, system_expr("Null")).and_then(CallbackValue::adopt) {
            Ok((cx, values)) => cx.finish_list(values),
            Err(failure) => Ok(failure.finish()),
        }
    }).unwrap();
}
```

```compile_fail,E0616
use rum_runtime::kernel_api::{Session, system_expr, CallbackValue};
fn main() {
    Session::new().with_unarmed_retention(|cx, values| {
        match cx.evaluate(values, system_expr("Null")).and_then(CallbackValue::adopt) {
            Ok((cx, values)) => cx.finish_list(values),
            Err(failure) => { let _unleased = failure.error; Ok(failure.finish()) },
        }
    }).unwrap();
}
```
