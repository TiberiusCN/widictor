package org.apqm.jni

final class NativeLib {
  @native def getPage(path: String, language: String): Unit
}
object NativeLib {
  // private[this]
  val api = new NativeLib
}

class Telua {
  @native private def nnew(): Long
  private val ptr = nnew()
}
