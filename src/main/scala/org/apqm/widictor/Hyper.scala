package org.apqm.widictor

import akka.actor.typed._
import akka.actor.typed.scaladsl._
import akka.http.scaladsl.Http
import akka.http.scaladsl.marshallers.sprayjson.SprayJsonSupport._
import akka.http.scaladsl.model._
import akka.http.scaladsl.server.Directives._
import akka.http.scaladsl.server.Route
import akka.stream.scaladsl._

object Hyper {
  def apply() = {
    (new Hyper).behavior
  }
  def tedit(template: String) = s"""<!DOCTYPE html>
    |<html lang="en">
    |  <head>
    |    <title>template $template</title>
    |    <style type="text/css">
    |      #editor {
    |        position: absolute;
    |        top: 0;
    |        right: 0;
    |        bottom: 0;
    |        left: 0;
    |      }
    |    </style>
    |  </head>
    |  <body>
    |    <div id="editor"></div>
    |    <script src="https://cdnjs.cloudflare.com/ajax/libs/ace/1.4.14/ace.js" type="text/javascript" charset="utf-8"></script>
    |    <script>
    |      var editor = ace.edit("editor"); // теперь обращаться к редактору будем через editor
    |      editor.setTheme("ace/theme/monokai")
    |      editor.getSession().setMode("ace/mode/lua")
    |      editor.setValue("local source = code")
    |    </script>
    |  </body>
    |""".stripMargin
}
class Hyper {
  import Hyper._

  def behavior: Behavior[Null] = Behaviors.setup { ctx =>
    val route: Route = optionalCookie("lang") { olang =>
      optionalCookie("auth") { oauth =>
        concat {
          path("tedit" / Segment ~ PathEnd) { tname =>
            complete(HttpResponse(
              entity = HttpEntity(ContentTypes.`text/html(UTF-8)`, Hyper.tedit(tname))
            ))
            //complete(s"$tname:$olang,$oauth")
          }
        }
      }
    }
    implicit val system = ctx.system.classicSystem
    val bind = Http().newServerAt("0.0.0.0", 8000).bind(route)

    Behaviors.receiveMessage {
      case _ => Behaviors.same
    }
  }
}
