package api

import akka.actor.typed.ActorSystem
import akka.actor.typed.scaladsl.Behaviors
import akka.http.scaladsl.marshallers.sprayjson.SprayJsonSupport
import akka.http.scaladsl.model.{ContentTypes, HttpEntity, StatusCodes}
import akka.http.scaladsl.server.Directives._
import akka.http.scaladsl.server.Route
import discovery.db.{get_metrics, get_pods, set_configurations}
import discovery.globals.{Metrics, static_algorithms}
import org.json4s.DefaultFormats
import org.json4s.native.Serialization
import spray.json.DefaultJsonProtocol
import api.helpers.{add_port, get_algo, get_all_metrics, set_algo}

import scala.collection.mutable.ArrayBuffer
import scala.language.postfixOps

object routes extends SprayJsonSupport with DefaultJsonProtocol {

  final case class Data(namespace: String, host:String,port:String)
  final case class Algo(namespace: String, host:String, algo:String)
  implicit val system :ActorSystem[Nothing] = ActorSystem(Behaviors.empty,"api")

  implicit val dataFormat = jsonFormat3(Data)
  implicit val algoFormat = jsonFormat3(Algo)
  implicit val MetricsFormat = jsonFormat4(Metrics)
  implicit val formats = DefaultFormats

  /**
   * route to add the host with its port to the redis db for the agent to recognize it
   */
  val add_port_route:Route = (post & path("ports")){
    entity(as[Data]){
      req => {
        val test = add_port(req.namespace, req.host, req.port)
        onSuccess(test){
          case true => complete("host/port added successfully")
          case false => complete(StatusCodes.ExpectationFailed,"failed")
        }
      }
    }

  }

  val get_metrics_route:Route = (get & path("metrics" / Segment )){
    pod => {
      try{
        val metrics = helpers.get_metrics(pod)
        onSuccess(metrics){
          case metrics => complete(200,HttpEntity(ContentTypes.`application/json`,Serialization.write(metrics)))
        }
      }catch{
        case _:Exception => complete(404,"not found")
      }
    }
  }

  val get_all_metrics_route:Route = (get & path("metrics")){
    val metrics = get_all_metrics()
    onSuccess(metrics){
      case metrics =>  complete(200,HttpEntity(ContentTypes.`application/json`,Serialization.write(metrics)))
    }

  }

  val get_all_algos_route:Route = (get & path("algo")){
    val algos = get_algo()
    onSuccess(algos){
      case algos =>  complete(200,HttpEntity(ContentTypes.`application/json`,Serialization.write(algos)))
    }

  }

  val set_algo_route:Route = (post & path("algo")){
    entity(as[Algo]){
      req => {
        val test = set_algo(req.namespace, req.host, req.algo)
        onSuccess(test){
          case "ok" => complete("load balancing algorithm is updated successfully !")
          case "not ok" => complete(404, "not found")
          case "wrong" => complete(400, "choose from these algorithms: ['custom' or '', 'rr', 'least_conn', 'ip_hash']")
        }

      }
    }
  }

  val routes = concat(
    get_metrics_route, get_all_metrics_route, add_port_route,set_algo_route,get_all_algos_route
  )


}
