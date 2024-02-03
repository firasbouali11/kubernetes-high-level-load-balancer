package discovery

import akka.actor.typed.ActorSystem
import akka.actor.typed.javadsl.Behaviors
import discovery.globals.PodsInfo
import io.kubernetes.client.openapi.Configuration
import io.kubernetes.client.openapi.apis.{AppsV1Api, CoreV1Api}
import io.kubernetes.client.openapi.models._
import io.kubernetes.client.util.Config
import org.json4s.DefaultFormats
import org.json4s.native.Serialization

import scala.concurrent.ExecutionContextExecutor

/**
 * this class interacts with k8s cluster to discover the pods and their ip addresses
 */

class Discoverer {
  implicit val formats = DefaultFormats
  implicit val system: ActorSystem[Nothing] = ActorSystem(Behaviors.empty, "discoverer")
  implicit val executionContext: ExecutionContextExecutor = system.executionContext

  /**
   * connect with the k8s cluster
   */
//  private val client = Config.fromCluster()
  private val client = Config.defaultClient()

  Configuration.setDefaultApiClient(client)
  private val api = new CoreV1Api
  private val exapi = new AppsV1Api()


  /**
   *
   * @return the master node's ip
   */
  def get_node_ip ={
    val nodes = api.listNode(null,null,null,null,null,null,null,null,null,null)
    nodes.getItems.toArray().map(node => node.asInstanceOf[V1Node].getStatus.getAddresses.toArray().apply(0).asInstanceOf[V1NodeAddress].getAddress).mkString(",")
  }

  /**
   *
   * @return the namespaces inside the k8s cluster
   */
  def get_namespaces: Array[String] ={
    val namespaces = api.listNamespace(null,null,null,null,null,null,null,null,null,null)
      .getItems.toArray.map(e => e.asInstanceOf[V1Namespace].getMetadata.getName)
    namespaces
  }

  /**
   *
   * @return the pods that are in the same replicaset
   */
  def get_replicaset: Array[String] = {
    val a =exapi.listDeploymentForAllNamespaces(null,null,null,null,null,null,null,null,null,null)
    val info = a.getItems.toArray().map(e => e.asInstanceOf[V1Deployment].getMetadata.getNamespace +"."+e.asInstanceOf[V1Deployment].getMetadata.getName)
    info.filter(!_.equals(""))
  }

  def get_replicaset_from_sfs: Array[String] = {
    val a = exapi.listStatefulSetForAllNamespaces(null,null,null,null,null,null,null,null,null,null)
    val info = a.getItems.toArray().map(e => e.asInstanceOf[V1StatefulSet].getMetadata.getNamespace + "." + e.asInstanceOf[V1StatefulSet].getMetadata.getName)
    info.filter(!_.equals(""))
  }

  /**
   *
   * @return pods informations such as name, ip and namespace
   */
  def get_pods_info: Array[PodsInfo] ={
    val pods = api.listPodForAllNamespaces(null,null,null,null,null,null,null,null,null,null)
      .getItems

    val infos = pods.toArray.map(pod => {
      val podInstance = pod.asInstanceOf[V1Pod]
      val podName = podInstance.getMetadata.getName
      val podNameSpace = podInstance.getMetadata.getNamespace
      val podIP = podInstance.getStatus.getPodIP
      val podUuid = podInstance.getMetadata.getUid
      PodsInfo(
        podNameSpace + "." + podName,
        podIP,
        podNameSpace,
        podUuid
      )
    })
    infos.foreach( i => println(Serialization.write(i)))
    infos
  }

}