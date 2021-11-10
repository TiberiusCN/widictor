package org.apqm.widictor

import org.parboiled2._
import cats.implicits._
import shapeless.HNil

sealed trait WikiAst
case class Language(lang: String) extends WikiAst
case class Section(name: String, level: Int) extends WikiAst
case class RawText(text: String) extends WikiAst
case object NewLine extends WikiAst

class WikiParser(val input: ParserInput) extends Parser {
  def nl = rule { '\n' }
  def printable = rule { oneOrMore { CharPredicate.AlphaNum ++ " " } }
  def mark = rule { "=" }
  def sectionMark = rule { "==" ~ capture(mark.*) ~> (_.length) }
  def section = rule {
    sectionMark ~ capture(printable) ~ sectionMark ~> { (b, p, e) =>
      test(b == e) ~ push(if (e == 0) Language(p) else Section(p, e - 1))
    }
  }
  def rawText = rule { capture(noneOf("\n").+) ~> (RawText(_)) }
  def openTemplate = rule { "{{" }
  def closeTemplate = rule { "}}" }
  def unicodePrefix = rule { "\\u" }
  def quote = rule { "\\\"" }
  def template = rule { openTemplate ~ closeTemplate ~ push(RawText("TEMPLATE")) }
  def text: Rule1[WikiAst] = rule { rawText | template }
  def empty: Rule1[WikiAst] = rule { push(NewLine) }
  def line: Rule1[WikiAst] = rule { section | text | empty }
  def total: Rule1[List[List[WikiAst]]] = rule { line.+(nl) ~ nl.? ~> { seq =>
    val list = seq.toList
    list.foldLeft(List[List[WikiAst]]())(mergeTree)
  } ~ EOI }
  private def mergeTree(acc: List[List[WikiAst]], line: WikiAst): List[List[WikiAst]] = {
    val (current, branches) = acc match {
      case Nil => (List(), List())
      case a :: tail => (a, tail)
    }
    import scala.io.AnsiColor._
    merge(current, line, None) ++ branches
  }
  private def merge(current: List[WikiAst], line: WikiAst, root: Option[List[WikiAst]]): List[List[WikiAst]] = {
    val ret: List[WikiAst] => List[List[WikiAst]] = { next =>
      root match {
        case Some(root) => next :: root :: Nil
        case None => next :: Nil
      }
    }
    (current, line) match {
      case (Nil, j) => ret(j :: Nil)
      case (up :: tail, down) =>
        (up, down) match {
          case (RawText(up), RawText(down)) => ret(RawText(up + down) :: tail)
          case (_: Language, _: Section)
             | (_: Language, _: RawText)
             | (_: Section, _: RawText)
             => ret(down :: up :: tail)
          case (_: Language, NewLine)
             | (_: Section, NewLine)
             | (_: RawText, NewLine)
             => ret(up :: tail)
          case (Section(_, levelu), Section(_, leveld)) if (levelu < leveld) =>
            ret(down :: up :: tail)
          case _ => merge(tail, down, Some(root.getOrElse(current)))
        }
    }
  }
}
