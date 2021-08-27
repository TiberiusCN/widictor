use jni::{JNIEnv, objects::{JClass, JString}};

mod remote;
mod scribunto;
mod wiki;

#[no_mangle]
pub extern "system" fn Java_org_apqm_native_NativeLib_getPage(jenv: JNIEnv, _jclass: JClass, path: JString, language: JString) {
  let path: String = jenv.get_string(path).unwrap().into();
  let language: String = jenv.get_string(language).unwrap().into();
  wiki::scan(&path, &language);
}

#[no_mangle]
pub extern "system" fn Java_org_apqm_native_Telua_empty(jenv: JNIEnv, jclass: JClass) -> u64 {
  1
}
