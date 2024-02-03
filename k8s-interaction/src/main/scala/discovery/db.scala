package discovery

import com.redis.RedisClient
import discovery.globals.{Metrics, PodsInfo, REDIS_HOST, REDIS_PORT}
import org.json4s.DefaultFormats
import org.json4s.native.{Serialization, parseJson}

object db {

  implicit val formats = DefaultFormats
  val r = new RedisClient(REDIS_HOST, REDIS_PORT)



  /**
   *
   * @param node_ip: master node's ip
   * @return boolean value to check if the value is stored successfully
   */
  def set_node_ip(node_ip: String): Boolean = {
    r.set("node", node_ip)
  }

  /**
   *
   * @param pod_name: the pod name
   * @return the metrics calculated for the given pod
   */
  def get_metrics(pod_name: String): Metrics = {
    val data = r.get(pod_name+"-metrics").getOrElse("None")
    if(data.equals("None")) Metrics("45",0.0,0.0,0.0)
    else parseJson(data).extract[Metrics];
  }

  /**
   *
   * @param pods: list of the currently existing pod names
   * @return boolean value to check if the value is stored successfully
   */
  def set_pods_names(pods:Array[String]): Boolean ={
    val data = Serialization.write(pods)
    r.set("pods",data)
  }

  /**
   *
   * @param pod_info case classe taht contains the metrics of the pod
   * @return boolean value to check if the value is stored successfully
   */
  def set_pod_info(pod_info: PodsInfo): Boolean = {
    val data = Serialization.write(pod_info)
    r.set(pod_info.name, data)
  }

  /**
   *
   * @param depls: list of existing deployments in the kubernetes cluster
   * @return boolean value to check if the value is stored successfully
   */
  def set_hosts(depls: Array[String]): Boolean = {
    r.set("hosts", Serialization.write(depls))
  }
  def set_nodes(nodes:String): Boolean = {
    r.set("nodes", nodes)
  }

  /**
   *
   * @param depls list of deployments currently existing in the kubernetes cluster
   * @param pods list of pods informations (name, ip and namespace)
   * @return boolean value to check if the value is stored successfully
   */
  def set_depl(depls: Array[String], pods: Array[PodsInfo]): Boolean = {
    for (depl <- depls) {
      val b = depl.split("\\.")
      val repl = Serialization.write(pods.filter(pod => pod.name.contains(b(1)) && pod.namespace.equals(b(0))).map(pod => pod.name))
      val a = r.set(depl, repl)
      if (!a) return false
    }
    true
  }

  /**
   *
   * @param host: host to be written in the location host of the nginx configuration file (deployment name by default)
   * @param port: the port of the application
   * @return boolean value to check if the value is stored successfully
   */
  def set_configurations(host: String, port: String): Boolean = {
    r.set(host, port)
  }


  /**
   *
   * @param pod_name: the name of the pod
   * @return object PodsInfo containing (ip, name and namespaces)
   */
  def get_pod_infos(pod_name: String): PodsInfo = {
    val a = r.get(pod_name).get
    parseJson(a).extract[PodsInfo]
  }

  /**
   *
   * @return get all pods names from the db
   */
  def get_pods: List[String] ={
    val a = r.get("pods").get
    parseJson(a).extract[List[String]]
  }

  def get_replicaset(depl:String): List[String] ={
    val a = r.get(depl).get
    parseJson(a).extract[List[String]]
  }

  def get_algo(pod_name:String): String = {
    println(pod_name)
    val data = r.get(pod_name+"-algo").getOrElse("None")
    if(data.equals("None")) ""
    else data
  }

  def get_hosts(): List[String] ={
    val a = r.get("hosts").get
    parseJson(a).extract[List[String]]
  }


}
