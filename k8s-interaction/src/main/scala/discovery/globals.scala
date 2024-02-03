package discovery

import scala.util.Properties.envOrElse

/**
 * global variables and case classes
 */

object globals {

  final case class Metrics(pod:String,ram: Double, cpu: Double, bandwidth: Double)

  final case class PodsInfo(name: String, ip: String, namespace: String,uuid:String)

  final val static_algorithms = Array[String]("","custom","rr","least_conn", "ip_hash")

  val REDIS_HOST: String = envOrElse("REDIS_HOST", "localhost")
  val REDIS_PORT: Int = envOrElse("REDIS_PORT", "6379").toInt
}
