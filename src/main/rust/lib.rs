use jni::{JNIEnv, objects::{JClass, JString}};

mod remote;
mod scribunto;
mod wiki;

#[no_mangle]
pub extern "C" fn Java_org_apqm_native_NativeLib_getPage(jenv: JNIEnv, jclass: JClass, path: JString, language: JString) {
}
