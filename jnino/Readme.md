This project allows you to use rust objects like part of jvm objects.

You have to add ptr: Long to your jvm object and call corresponding native constructor. This value will be used as native pointer.

```
class Sample {
  @native private def nnew(): Long
  private val ptr = nnew()
}
```

On the rust side you can:

- use throw(JNIEnv, default_function)

```
#[no_mangle]
pub extern "system" fn Java_package_Sample_nnew(
  jenv: JNIEnv,
  _jclass: JClass,
) -> *mut Sample {
  Telua::empty().jni().throw(jenv, null_mut)
}
```

#[no_mangle]
pub extern "system" fn Java_org_apqm_jni_NativeLib_getPage(jenv: JNIEnv, _jclass: JClass, path: JString, language: JString) {
  let path: String = jenv.get_string(path).unwrap().into();
  let language: String = jenv.get_string(language).unwrap().into();
  wiki::scan(&path, &language);
}

#[no_mangle]
pub extern "system" fn Java_org_apqm_jni_Telua_close(
  jenv: JNIEnv,
  jclass: JClass,
) {
  Telua::box_raw(&jenv, jclass).map(hide).throw(jenv, unit)
}

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

