package org.apqm.native

final class NativeLib {
  @native def getPage(path: String, language: String): Unit
}
object NativeLib {
  System.loadLibrary("widictor")
  private[this] val api = new NativeLib
}
