use std::ptr::null_mut;

use jni::{JNIEnv, objects::{JClass, JString}};
use wiki::Telua;

type Jr<T> = Result<T, Box<dyn std::error::Error>>;
//#[derive(Debug)]
// struct Se(String);
// impl std::fmt::Display for Se {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     self.0.fmt(f)
//   }
// }
// impl From<&str> for Se {
//   fn from(it: &str) -> Self {
//     Self(it.to_owned())
//   }
// }
fn unit() {}
trait Throwable<T> {
  fn throw<L: FnOnce() -> T>(self, jenv: JNIEnv, or_else: L) -> T;
}
impl<T> Throwable<T> for Jr<T> {
  fn throw<L: FnOnce() -> T>(self, jenv: JNIEnv, or_else: L) -> T {
    match self {
      Ok(it) => it,
      Err(e) => {
        let _ = jenv.throw(e.to_string()).map_err(|e| eprintln!("can't throw: {}", e));
        or_else()
      }
    }
  }
}
pub trait JIface: Sized {
  fn mut_raw<'a>(jenv: &JNIEnv<'a>, jclass: JClass<'a>) -> Jr<Option<&'a mut Self>> {
    let r = jenv.get_field(jclass, "ptr", "J")?.j()? as isize;
    unsafe {
      let r: usize = std::mem::transmute(r);
      let r: *mut Self = r as _;
      if let Some(r) = r.as_mut() {
        Ok(Some(r))
      } else {
        Ok(None)
      }
    }
  }
  fn box_raw<'a>(jenv: &JNIEnv<'a>, jclass: JClass<'a>) -> Jr<Box<Self>> {
    Self::mut_raw(jenv, jclass)
      .and_then(|it| -> Jr<_> {
        unsafe {
          jenv.set_field(jclass, "ptr", "J", 0.into()).map_err(Box::new)?;
          Ok(Some(Box::from_raw(it)))
        }
      })
  }
}

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
  (|| -> Jr<_> {
    Ok(Box::into_raw(Box::new(Telua::empty()?)))
  })().throw(jenv, null_mut)
}
#[no_mangle]
pub extern "system" fn Java_org_apqm_jni_Telua_close(
  jenv: JNIEnv,
  jclass: JClass,
) {
  (|| jclass.destroy::<Telua>(&jenv))().throw(jenv, unit)
}
