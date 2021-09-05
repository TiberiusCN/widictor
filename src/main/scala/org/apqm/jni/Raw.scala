package org.apqm.jni

final class NativeLib {
  @native def getPage(path: String, language: String): Unit
}
object NativeLib {
  // private[this]
  val api = new NativeLib
}

class Telua extends AutoCloseable {
  @native private def nnew(): Long
  @native def close: Unit
  override def finalize = close
  private val ptr = nnew()
}
