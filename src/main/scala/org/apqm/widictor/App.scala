package org.apqm.widictor

import org.parboiled2._
import scala.io.AnsiColor._

object Main extends App {
  val onError = { parser: WikiParser => { z: Throwable => z match {
    case e: ParseError => RED + parser.formatError(e) + RESET
    case n => n.toString
  }}}
  val p = new WikiParser(java.nio.file.Files.readString(java.nio.file.Paths.get("test")))
  List[String]().map { l =>
    val parser = new WikiParser(l)
    println(l + " → ")
    val out = parser.total.run().toEither.left.map(onError(parser))
    out match {
      case Left(e) => println(s"${RED}$e${RESET}")
      case Right(s) => s.map(println(_))
    }
  }
  val out = p.total.run().toEither.left.map(onError(p))
  out match {
    case Left(e) => println(s"${RED}$e${RESET}")
    case Right(s) => s.reverse.map { l =>
      var p = ""
      l.reverse.map { r =>
        println(p + r)
        p = p + "-"
      }
    }
  }
}
