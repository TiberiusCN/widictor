package org.apqm.widictor

import org.parboiled2._
import scala.io.AnsiColor._

object Main extends App {
  val out = Processor.sandbox("""
    | args = ...
    | template = args:template("cog")
    | template:set("1", "it")
    | template:set("2", "mangiare")
    | result = template:evaluate('manger')
    | print(result)
    |""".stripMargin)
  println(s"[$out]")
  // val out = WikiParser.run(java.nio.file.Files.readString(java.nio.file.Paths.get("test")), "French")
  // out match {
  //   case Left(e) => println(s"${RED}$e${RESET}")
  //   case Right(s) => println(s)
  // }
}
