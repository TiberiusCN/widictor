package org.apqm.jni;

class LuaTable[T: LuaName] extends AutoCloseable {
  @native private def nnew: Long
  @native def close
  private val ptr = nnew
  override def finalize = close
}

package implicits {
}
