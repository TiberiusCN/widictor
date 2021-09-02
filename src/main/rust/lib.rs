use std::ptr::null_mut;

use jni::{JNIEnv, objects::{JClass, JString}};
use jnino::*;
use wiki::Telua;

trait JRef {
  fn as_mut<T>(&self, jenv: &JNIEnv) -> Jr<&mut T>;
  fn nullify(&self, jenv: &JNIEnv) -> Jr<()>;
  fn destroy<T>(&self, jenv: &JNIEnv) -> Jr<()> {
    unsafe {
      Box::from_raw(self.as_mut::<T>(jenv)?);
    }
    self.nullify(jenv)
  }
}
impl JRef for JClass<'_> {
  fn as_mut<T>(&self, jenv: &JNIEnv) -> Jr<&mut T> {
    unsafe {
      let r = jenv.get_field(*self, "ptr", "J")?.j()? as *mut T;
      if let Some(r) = r.as_mut() {
        Ok(r)
      } else {
        Err("null pointer".into())
      }
    }
  }
  fn nullify(&self, jenv: &JNIEnv) -> Jr<()> {
    Ok(jenv.set_field(*self, "ptr", "J", 0.into())?)
  }
}

mod remote;
mod scribunto;
mod wiki;

#[no_mangle]
pub extern "system" fn Java_org_apqm_jni_NativeLib_getPage(jenv: JNIEnv, _jclass: JClass, path: JString, language: JString) {
  let path: String = jenv.get_string(path).unwrap().into();
  let language: String = jenv.get_string(language).unwrap().into();
  wiki::scan(&path, &language);
}

#[no_mangle]
pub extern "system" fn Java_org_apqm_jni_Telua_nnew(
  jenv: JNIEnv,
  _jclass: JClass,
) -> *mut Telua {
  Telua::empty().jni().throw(jenv, null_mut)
}
#[no_mangle]
pub extern "system" fn Java_org_apqm_jni_Telua_close(
  jenv: JNIEnv,
  jclass: JClass,
) {
  Telua::box_raw(&jenv, jclass).map(hide).throw(jenv, unit)
}
