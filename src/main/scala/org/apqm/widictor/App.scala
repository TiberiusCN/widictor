package org.apqm.widictor

import org.parboiled2._
import scala.io.AnsiColor._

object Main extends App {
  val out = Processor.sandbox("""
    | args = ...
    | args:hello()
    | print('1 + 2 = ', args:plus(1, 2))
    |""".stripMargin)
  println(s"[$out]")
  // val out = WikiParser.run(java.nio.file.Files.readString(java.nio.file.Paths.get("test")), "French")
  // out match {
  //   case Left(e) => println(s"${RED}$e${RESET}")
  //   case Right(s) => println(s)
  // }
}
