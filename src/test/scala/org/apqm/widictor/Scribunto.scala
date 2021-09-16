package org.apqm.widictor

import utest._
import scala.util._
import org.parboiled2._

object ScribuntoAnswerSpec extends TestSuite {
  val tests = Tests {
    "The ScribuntoAnswer" - {
      "parse 'N' to None" - {
        parse("N") ==> None
      }
    }
  }
  def parse(s: String): Object = {
    val parser = new ScribuntoAnswer(s)
    parser.Lua.run()
  }
}
