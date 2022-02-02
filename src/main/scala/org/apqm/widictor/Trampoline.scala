package org.apqm.widictor

import akka.actor.typed._
import akka.actor.typed.scaladsl._
import akka.cluster.typed.Cluster
import cats.implicits._
import com.typesafe.config.ConfigFactory

object Trampoline {
  private def genConfig(hostname: String, port: Int, seeds: Array[String], role: String) = {
    val seed = seeds.map("\"akka://cluster@" + _ + "\"").foldLeft("")(_ + "\n" + _)
    val raw = s"""
      |akka {
      |  loglevel = "WARNING"
      |  stdout-loglevel = "WARNING"
      |  actor {
      |    provider = cluster
      |    serialization-bindings {
      |      "org.apqm.widictor.Cbored" = jackson-cbor
      |    }
      |  }
      |  remote.artery {
      |    canonical.port = $port
      |    bind.port = $port
      |    canonical.hostname = "$hostname"
      |    bind.hostname = "$hostname"
      |  }
      |  cluster {
      |    roles = [$role]
      |    seed-nodes = [
      |      $seed
      |    ]
      |    downing-provider-class = "akka.cluster.sbr.SplitBrainResolverProvider"
      |  }
      |}""".stripMargin
    println(s"$raw")
    ConfigFactory.parseString(raw).withFallback(ConfigFactory.load())
  }
  def apply[T](actor: Behavior[T], role: String, args: Array[String]) = {
    val behavior = Behaviors.setup[Nothing] { ctx =>
      val cluster = Cluster(ctx.system)
      ctx.spawn(actor, "main")
      Behaviors.empty[Nothing]
    }
    val config = genConfig(
      args(0),
      args(1).toInt,
      args(2).split(',').filter(!_.isEmpty),
      role,
    )
    ActorSystem[Nothing](behavior, "cluster", config)
  }
}
