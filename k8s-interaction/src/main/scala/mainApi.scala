import akka.actor.typed.ActorSystem
import akka.actor.typed.scaladsl.Behaviors
import akka.http.scaladsl.Http
import api.routes.routes

import scala.util.Properties.envOrElse

object mainApi {
  def main(args: Array[String]): Unit = {

    implicit val system: ActorSystem[Nothing] = ActorSystem(Behaviors.empty, "api")

    val API_PORT = envOrElse("API_PORT", "5000")
    val API_HOST = "0.0.0.0"


    try{
      Http().newServerAt(API_HOST, API_PORT.toInt).bind(routes)
      println(s"server is running at $API_HOST:$API_PORT")
    }catch{
      case e:Exception => println(e.getMessage)
    }
  }
}
