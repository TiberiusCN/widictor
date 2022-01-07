package org.apqm.widictor

import cats.implicits._
import org.parboiled2._
import scala.util._

sealed trait WikiAst[A]
case class Language[A](lang: String, var sections: List[Section[A]], var text: A) extends WikiAst[A] {
  def map[E, B](f: (Option[String], A) => Either[E, B]) = for {
    text <- f(None, text)
    sections <- sections.map(_.map(f)).sequence
  } yield Language(lang, sections, text)
}
case class Section[A](name: String, level: Int, var subsections: List[Section[A]], var text: A) extends WikiAst[A] {
  def map[E, B](f: (Option[String], A) => Either[E, B]): Either[E, Section[B]] = for {
    text <- f(name.some, text)
    subsections <- subsections.map(_.map(f)).sequence
  } yield Section(name, level, subsections, text)
}
case object NewLine extends WikiAst[RawText]
case object Separator extends WikiAst[RawText]
case object LineSpace extends WikiAst[RawText]
sealed trait Clarifiable
sealed trait WikiTextAst {
  type Clarifier = (Clarifiable) => Either[Throwable, String]
  def clarify(clarifier: Clarifier): Either[Vector[Throwable], String]
}
case class RawText(var text: String) extends WikiAst[RawText] with WikiTextAst {
  def clarify(clarifier: Clarifier) = text.asRight
}
case class WikiText(text: List[WikiTextAst]) extends WikiAst[WikiTextAst] with WikiTextAst {
  def clarify(clarifier: Clarifier) = text.partitionMap(_.clarify(clarifier)) match {
    case (Nil, rights) => Right(rights.foldLeft("")(_+_))
    case (lefts, _) => Left(lefts.flatten.toVector)
  }
}
case class RawWikiTemplate(main: String, params: Map[String, String]) extends Clarifiable {
  def wikify = "{{" + main + params.foldLeft("")((acc, p) => s"$acc|${p._1}=${p._2}") + "}}"
}
case class WikiTemplate(main: String, params: Seq[WikiTextAst]) extends WikiTextAst {
  def clarify(clarifier: Clarifier) = for {
    newParams <- params.partitionMap(_.clarify(clarifier)) match {
      case (Nil, rights) => {
        var unnamed = Seq[String]()
        var named = scala.collection.mutable.HashMap[String, String]()
        rights.foreach(_.split("=", 2) match {
          case Array(full) => unnamed :+ full
          case Array(name, value) => named += name -> value
        })
        var id = 0
        val generator = { () =>
          id += 1
          var test = s"$id"
          while (named.contains(test)) {
            id += 1
            test = s"$id"
          }
          test
        }
        unnamed.foreach(named += generator() -> _)
        Right(named.toMap)
      }
      case (lefts, _) => Left(lefts.flatten.toVector)
    }
    template <- clarifier(RawWikiTemplate(main, newParams)).left.map(Vector(_))
  } yield template
}
case class WikiLink(display: WikiText, link: WikiText) extends WikiTextAst {
  def clarify(clarifier: Clarifier) = display.clarify(clarifier)
}
case class WikiQuote(text: WikiText) extends WikiTextAst {
  def clarify(clarifier: Clarifier) = text.clarify(clarifier)
}
case class WikiBold(text: WikiText) extends WikiTextAst {
  def clarify(clarifier: Clarifier) = text.clarify(clarifier)
}

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
  def separator = rule { "----" ~ push(Separator) }
  def empty = rule { push(LineSpace) }
  def emptyLine = rule { '\n' ~ &('\n') ~ push(NewLine) }
  def tabMark1 = rule { '#' }
  def tabMark2 = rule { '*' }
  def tabMark3 = rule { ':' }
  def tabMark = rule { tabMark1 | tabMark2 | tabMark3 }
  def tabs = rule { capture(tabMark.+) ~> (_.length) ~> (_ => ()) }
  def line = rule { section |  rawText | separator | emptyLine | empty }
  def total: Rule1[Option[Language[RawText]]] = rule { (tabs.? ~ line).+(nl) ~ nl.? ~> { seq =>
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
            | (_, Separator)
            => up :: tail
        case (up: Section[_], down: Section[_]) if (up.level < down.level) =>
          up.subsections = down :: up.subsections
          down :: up :: tail
        case _ => insert(tail, line)
      }
    }
  }

  def openTemplate = rule { "{{" }
  def closeTemplate = rule { "}}" }
  def openInternalLink = rule { "[[" }
  def closeInternalLink = rule { "]]" }
  def internalLinkSimple = rule { openInternalLink ~ textRaw ~ closeInternalLink ~> { j =>
    val l = WikiText(j.toList)
    WikiLink(l, l)
  }}
  def internalLinkComplex = rule { openInternalLink ~ textRaw ~ '|' ~ textRaw ~ closeInternalLink ~> { (link, display) =>
    WikiLink(WikiText(display.toList), link = WikiText(link.toList))
  }}
  def openExternalLink = rule { "[" }
  def closeExternalLink = rule { "]" }
  def externalLinkSimple = rule { openExternalLink ~ textRaw ~ closeExternalLink ~> { j =>
    val l = WikiText(j.toList)
    WikiLink(l, l)
  }}
  def externalLinkComplex = rule { openExternalLink ~ textRaw ~ '|' ~ textRaw ~ closeExternalLink ~> { (link, display) =>
    WikiLink(WikiText(display.toList), link = WikiText(link.toList))
  }}
  def link = rule { internalLinkComplex | internalLinkSimple | externalLinkComplex | externalLinkSimple }
  def quoteMark = rule { "''" }
  def quote = rule { quoteMark ~ textRaw ~ quoteMark ~> (j => WikiQuote(WikiText(j.toList))) }
  def boldMark = rule { "'''" }
  def bold = rule { boldMark ~ textRaw ~ boldMark ~> (j => WikiBold(WikiText(j.toList))) }
  def unicodePrefix = rule { "\\u" }
  def unicodeStr = rule { unicodePrefix ~ capture(4.times(CharPredicate.HexDigit)) ~> (_.foldLeft("")(_+_)) }
  def unicode = rule { unicodeStr ~> (j => Integer.parseInt(j, 16)) ~> (j => RawText(j.toChar.toString)) }
  def pureText = rule { capture(noneOf("\\\n{}|[]'").+) ~> (RawText(_)) }
  def templateParams: Rule1[Seq[Seq[WikiTextAst]]] = rule { { '|' ~ textRaw }.* }
  def template = rule { openTemplate ~ pureText ~ templateParams ~ closeTemplate ~> { (name, params) =>
    WikiTemplate(name.text, params.map(j => WikiText(j.toList)))
  }}
  def apostroph = rule { "'" ~ &(noneOf("'")) ~ push(RawText("'")) }
  def textElement = rule { template | link | unicode | bold | quote | pureText }
  def textRaw: Rule1[Seq[WikiTextAst]] = rule { textElement.* }
  def text: Rule1[WikiText] = rule { textRaw ~ EOI ~> { text: Seq[WikiTextAst] =>
    WikiText(text.toList)
  }}
}
object WikiParser {
  def run(source: String, lang: String) = {
    val word = "manger"
    val onError = { parser: WikiParser => { z: Throwable => z match {
      case e: ParseError => parser.formatError(e)
      case n => n.toString
    }}}
    val p = new WikiParser(source, lang)
    for {
      total <- p.total.run().toEither.left.map(onError(p))
      language <- total.toRight(s"Language $lang not found")
      wikitext <- language.map[String, WikiText] { case (_, j) =>
        val p = new WikiParser(j.text, lang)
        p.text.run().toEither.left.map(onError(p))
      }
      text <- wikitext.map { case (section, j) =>
        j.clarify({
          case template @ RawWikiTemplate(main, params) =>
            Jsons.expandTemplate(word, template)
        })
      }
    } yield text
  }
}

object Jsons {
  import play.api.libs.json._
  case class Wikitext(wikitext: String)
  implicit val WikitextFormat = Json.format[Wikitext]
  case class ExpandedTemplate(expandtemplates: Wikitext)
  implicit val ExpanedTemplateFormat = Json.format[ExpandedTemplate]
  def expandTemplate(word: String, template: RawWikiTemplate) = Try {
    val templateValue = java.net.URLEncoder.encode(template.wikify)
    val url = s"https://en.wiktionary.org/w/api.php?action=expandtemplates&format=json&prop=wikitext&title=$word&text=$templateValue"
    val out = scala.io.Source.fromURL(url).mkString
    Json.parse(out).as[ExpandedTemplate].expandtemplates.wikitext
  }.toEither
}
