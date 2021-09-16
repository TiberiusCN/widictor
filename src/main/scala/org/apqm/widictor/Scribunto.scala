package org.apqm.widictor

import org.parboiled2._

class ScribuntoAnswer(val input: ParserInput) extends Parser {
  def Null = rule { 'N' }
  def Close = rule { '}' }
  def Open = rule { '{' }
  def Separator = rule { ':' }
  def prefix = rule { capture((CharPredicate.AlphaNum ++ "_").+) }
  def Lua = rule { Null ~ EOI }
}
