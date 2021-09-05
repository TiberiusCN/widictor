This project allows you to use rust objects like part of jvm objects.

## JVM

- add native constructor with Long return and private Long property

- implement AutoCloseable as native method

- add finalizer

- add native methods

```
class Sample extends AutoCloseable {
  @native private def nnew(): Long
  @native def close: Unit
  @native def test: Unit
  private val ptr = nnew()
  override def finalize = close
}
```

- use try-with-resource (java) / Using (scala) or call close() implicitly (don't trust to finalization — it's a thin red line)

## Rust

- mark structure as JFace

```
#[derive(JFace)]
pub struct Sample { … }
```

- use throw(JNIEnv, default_function) for converting rust Jr<_> (Result<_, Box<dyn std::error::Error>> to default JVM exception

- use JFace::jni() for converting to raw heap pointer

- provide native constructor to JNI

```
#[no_mangle]
pub extern "system" fn Java_package_Sample_nnew(
  jenv: JNIEnv,
  _jclass: JClass,
) -> *mut Sample {
  Sample::new().jni().throw(jenv, null_mut)
}
```

- use unit for Result<(), _>::throw() or hide for Result<T, _>::throw for unneeded T

- use JFace::mut_raw and map/and_then for native methods

```
#[no_mangle]
pub extern "system" fn Java_org_apqm_jni_Sample_test(
  jenv: JNIEnv,
  jclass: JClass,
) {
  Sample::mut_raw(&jenv, jclass).map(|it| {
    todo!()
  }).throw(jenv, unit);
}
```

- use JFace::box_raw for destructor

```
#[no_mangle]
pub extern "system" fn Java_org_apqm_jni_Sample_close(
  jenv: JNIEnv,
  jclass: JClass,
) {
  Sample::box_raw(&jenv, jclass).map(hide).throw(jenv, unit)
}
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

