package org.apqm.widictor

import org.apqm.jni._

object App {
  def main(args: Array[String]) {
    System.loadLibrary("widictor")

    val telua = new Telua
    // NativeLib.api.getPage(args(0), args(1));
  }
}
