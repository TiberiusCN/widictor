pub use derive_jface::JFace;
use jni::{objects::JClass, JNIEnv};
pub type Jr<T> = Result<T, Box<dyn std::error::Error>>;
pub fn unit() {}
pub fn hide<T>(_: T) {}

pub trait Throwable<T> {
  fn throw<L: FnOnce() -> T>(self, jenv: JNIEnv, or_else: L) -> T;
}
impl<T> Throwable<T> for Jr<T> {
  fn throw<L: FnOnce() -> T>(self, jenv: JNIEnv, or_else: L) -> T {
    match self {
      Ok(it) => it,
      Err(e) => {
        let _ = jenv.throw(e.to_string()).map_err(|e| log::warn!("can't throw: {}", e));
        or_else()
      }
    }
  }
}

pub trait JFace: Sized {
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
  fn box_raw<'a>(jenv: &JNIEnv<'a>, jclass: JClass<'a>) -> Jr<Option<Box<Self>>> {
    Self::mut_raw(jenv, jclass).and_then(|it| {
      it.map(|it| unsafe {
        jenv.set_field(jclass, "ptr", "J", 0.into()).map_err(Box::new)?;
        Ok(Box::from_raw(it))
      })
      .transpose()
    })
  }
  fn jni(self) -> *mut Self {
    Box::into_raw(Box::new(self))
  }
}

pub trait JRConv<T> {
  fn jni(self) -> Jr<*mut T>;
}
impl<T: JFace, E: Into<Box<dyn std::error::Error>>> JRConv<T> for Result<T, E> {
  fn jni(self) -> Jr<*mut T> {
    self.map_err(Into::into).map(JFace::jni)
  }
}
