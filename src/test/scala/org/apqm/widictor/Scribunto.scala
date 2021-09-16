package org.apqm.widictor

import org.scalatest._
import scala.util._
import org.parboiled2._

object ScribuntoAnswerSpec extends WordSpec {
  "The ScribuntoAnswer" when {
    "N" should {
      "produce None" in {
        assert(parse("N"), None)
      }
    }
  }

  def parse(s: String): Object = {
    val parser = new ScribuntoAnswer(s)
    parser.Lua.run()
  }
}
