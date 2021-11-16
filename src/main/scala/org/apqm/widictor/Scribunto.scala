package org.apqm.widictor

import org.parboiled2._
import cats.implicits._
import shapeless.HNil

sealed trait WikiAst[A]
case class Language[A](lang: String, var sections: List[Section[A]], var text: A) extends WikiAst[A] {
  def map[B](f: A => B) = Language[B](lang, sections.map(_.map(f)), f(text))
}
case class Section[A](name: String, level: Int, var subsections: List[Section[A]], var text: A) extends WikiAst[A] {
  def map[B](f: A => B): Section[B] = Section[B](name, level, subsections.map(_.map(f)), f(text))
}
case object NewLine extends WikiAst[RawText]
case object LineSpace extends WikiAst[RawText]
sealed trait WikiTextAst
case class RawText(var text: String) extends WikiAst[RawText] with WikiTextAst
case class WikiText(text: List[WikiTextAst]) extends WikiAst[WikiTextAst] with WikiTextAst
case class WikiTemplate(main: String, params: Map[String, WikiTextAst]) extends WikiTextAst

// case class Template(rule: Seq[List[WikiAst]], params: Seq[List[WikiAst]]) extends WikiAst

class WikiParser(val input: ParserInput, val langFilter: String) extends Parser {
  def nl = rule { '\n' }
  def printable = rule { oneOrMore { CharPredicate.AlphaNum ++ " " } }
  def mark = rule { "=" }
  def sectionMark = rule { "==" ~ capture(mark.*) ~> (_.length) }
  def section = rule {
    sectionMark ~ capture(printable) ~ sectionMark ~> { (b, p, e) =>
      test(b == e) ~ push(if (e == 0) Language(p, List(), RawText("")) else Section(p, e - 1, List(), RawText("")))
    }
  }
  def rawText = rule { capture(noneOf("\n").+) ~> (j => RawText(j)) }
  // def template = rule { openTemplate ~ closeTemplate ~ push(RawText("TEMPLATE")) }
  def empty = rule { push(LineSpace) }
  def emptyLine = rule { '\n' ~ &('\n') ~ push(NewLine) }
  def line = rule { section | rawText | emptyLine | empty }
  def total: Rule1[Option[Language[RawText]]] = rule { line.+(nl) ~ nl.? ~> { seq =>
    var filter = false
    seq.toList.flatMap {
      case node: Language[_] =>
        filter = node.lang == langFilter
        if (filter) Some(node) else None
      case node => if (filter) Some(node) else None
    }.foldLeft(List[WikiAst[RawText]]())(insert).lastOption.flatMap {
      case l: Language[_] => Some(l)
      case _ => None
    }
  } ~ EOI }

  def openTemplate = rule { "{{" }
  def closeTemplate = rule { "}}" }
  def unicodePrefix = rule { "\\u" }
  def pureText = rule { capture(noneOf("\n{}|").+) ~> (RawText(_)) }
  def templateParamsRaw: Rule1[Seq[Seq[WikiTextAst]]] = rule { { '|' ~ textRaw }.* }
  def templateParams: Rule1[Map[String, WikiTextAst]] = rule { templateParamsRaw ~> { params: Seq[Seq[WikiTextAst]] =>
    var id = 0
    params.map { seq =>
      id += 1
      id.toString -> WikiText(seq.toList)
    }.toMap
  }}
  def template = rule { openTemplate ~ pureText ~ templateParams ~ closeTemplate ~> { (name, params) =>
    WikiTemplate(name.text, params)
  }}
  def textElement = rule { template | pureText }
  def textRaw: Rule1[Seq[WikiTextAst]] = rule { textElement.* }
  def text: Rule1[Seq[WikiTextAst]] = rule { textRaw ~ EOI }

  @scala.annotation.tailrec
  private def insert(acc: List[WikiAst[RawText]], line: WikiAst[RawText]): List[WikiAst[RawText]] = {
    (acc, line) match {
      case (Nil, line: Language[_]) => line :: acc
      case (Nil, _) => Nil
      case (up :: tail, line) => (up, line) match {
        case (up: RawText, RawText(down)) =>
          up.text += down
          up :: tail
        case (up: RawText, NewLine) =>
          up.text += '\n'
          up :: tail
        case (l: Language[_], s: Section[_]) =>
          l.sections = s :: l.sections
          s :: l :: tail
        case (l: Language[_], RawText(t)) =>
          l.text.text += t
          l.text :: l :: tail
        case (s: Section[_], RawText(t)) =>
          s.text.text += t
          s.text :: s :: tail
        case (_: Language[_], NewLine)
            | (_: Section[_], NewLine)
            | (_, LineSpace)
            => up :: tail
        case (up: Section[_], down: Section[_]) if (up.level < down.level) =>
          up.subsections = down :: up.subsections
          down :: up :: tail
        case _ => insert(tail, line)
      }
    }
  }
}
object WikiParser {
  def run(source: String, lang: String) = {
    val onError = { parser: WikiParser => { z: Throwable => z match {
      case e: ParseError => parser.formatError(e)
      case n => n.toString
    }}}
    val p = new WikiParser(source, lang)
    p.total.run().toEither.left.map(onError(p)).map(_.map(_.map { j =>
      val p = new WikiParser(j.text, lang)
      val z = p.text.run().toEither.left.map(onError(p))
      import scala.io.AnsiColor._
      println(s"$RED$j$RESET → $z")
    }))
  }
}
