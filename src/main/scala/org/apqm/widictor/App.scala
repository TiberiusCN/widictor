package org.apqm.widictor

import org.parboiled2._
import scala.io.AnsiColor._

object Main extends App {
  val out = WikiParser.run(java.nio.file.Files.readString(java.nio.file.Paths.get("test")), "French", "fr")
  out match {
    case Left(e) => println(s"got error: ${RED}$e${RESET}")
    case Right(s) => println(s)
  }
}
