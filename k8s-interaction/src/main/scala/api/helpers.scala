package api

import akka.actor.typed.ActorSystem
import akka.actor.typed.scaladsl.Behaviors
import discovery.db.{get_hosts, get_metrics, get_pods, get_replicaset, set_configurations}
import discovery.db
import discovery.globals.{Metrics, static_algorithms}

import scala.collection.mutable
import scala.collection.mutable.ArrayBuffer
import scala.concurrent.Future

object helpers {

  implicit val system :ActorSystem[Nothing] = ActorSystem(Behaviors.empty,"helpers")
  implicit val executor = system.executionContext


  def add_port(namespace:String, host:String, port:String): Future[Boolean] ={
    Future{
      val test = set_configurations(namespace + "." + host+ "-port", port)
      test
    }

  }

  def get_all_metrics(): Future[ArrayBuffer[Metrics]] ={
    Future{
      val pods = get_pods;
      val metrics =ArrayBuffer[Metrics]()
      pods.foreach(pod => {
        val metric = db.get_metrics(pod)
        if(!metric.pod.eq("45")) metrics.addOne(metric)
      })
      metrics
    }

  }

  def get_metrics(data:String): Future[ArrayBuffer[Metrics]] ={
    Future{
      val replicaset = get_replicaset(data)
      val metrics =ArrayBuffer[Metrics]()
      replicaset.foreach(pod => {
        val metric = db.get_metrics(pod)
        if (!metric.pod.equals("45")) metrics.addOne(metric)
      })
      metrics
    }
  }

  def set_algo(namespace:String, host:String, algo:String): Future[String] ={
    Future{
      if(!static_algorithms.contains(algo)) "wrong"
      else {
        val test = set_configurations(namespace +"."+host + "-algo", algo)
        if(test) "ok" else "not ok"
      }
    }
  }

  def get_algo(): Future[mutable.Map[String,String]] ={
    Future{
      val pods = get_hosts;
      val pods_algos =mutable.Map[String,String]()
      pods.foreach(pod => {
        val metric = db.get_algo(pod)
        if(!metric.eq("")) pods_algos.addOne(pod, metric)
      })
      pods_algos
    }
  }

}
