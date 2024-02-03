import discovery.Discoverer
import discovery.db.{set_depl, set_hosts, set_nodes, set_pod_info, set_pods_names}
import io.kubernetes.client.openapi.ApiException

import java.io.IOException
import java.lang.Thread.sleep


object main {
  def main(args: Array[String]): Unit = {
    val d = new Discoverer
//    set_nodes(d.get_node_ip)
    while(true){
      try{
        val hosts = d.get_replicaset ++ d.get_replicaset_from_sfs
        val pods_info = d.get_pods_info
        pods_info.foreach(pod => set_pod_info(pod))
        set_pods_names(pods_info.map(pod => pod.name))
        set_depl(hosts,pods_info)
        set_hosts(hosts)
        println("update done")
        sleep(2000)
      }catch {
        case e1: ApiException => println(e1.getResponseBody)
        case e2: IOException => println(e2.getMessage)
        case e:Exception => println(e.getMessage)
      }
    }
  }
}
