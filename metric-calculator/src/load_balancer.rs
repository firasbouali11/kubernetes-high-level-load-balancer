use std::{borrow::BorrowMut, collections::HashMap};

use chrono::{DateTime, FixedOffset};
use redis::{Commands, Connection};

use crate::db::{deserialize_metrics, store_metrics};

/**
 * params: container_id: id of the container inside the pod, node_ip: ip of the master node, node_port: port of the master node
 * returns: the metrics of the container for the last 10s (cpu, ram and network bytes)
 */
fn get_metrics(
    container_id: String,
    node_ip: String,
    node_port: String,
) -> Result<serde_json::Value, ureq::Error> {
    let json: serde_json::Value = ureq::get(
        format!(
            "http://{}:{}/api/v1.3/containers/{}",
            node_ip, node_port, container_id
        )
        .as_str(),
    )
    .call()?
    .into_json()?;

    Ok(json)
}

/**
 * params: conn: connection with redis db, node_ip: ip of the master node, node_port: port of the master node
 * returns: map with the pod name and its corresponding container's id
 */
pub fn get_container_id(
    conn: &mut Connection,
    node_ip: String,
    node_port: String,
) -> HashMap<String, String> {
    let nodes: Vec<&str> = node_ip.split(",").collect();
    let mut map: HashMap<String, String> = HashMap::new();
    for node in nodes.iter() {
        let data =
            ureq::get(format!("http://{}:{}/api/v1.3/subcontainers", node, node_port).as_str())
                .call();
        //.unwrap().into_json();

        if let Ok(data) = data {
            let json: Result<serde_json::Value, std::io::Error> = data.into_json();
            if let Ok(json) = json {
                let pods_names: String = conn.get("pods").unwrap();
                let pods_json = deserialize_metrics(pods_names);
                for elt in json.as_array().unwrap() {
                    for pod in pods_json.as_array().unwrap() {
                        let p: Vec<&str> = pod.as_str().unwrap().split(".").collect();
                        if let Some(x) = elt["spec"]["labels"]["io.kubernetes.pod.name"].as_str() {
                            let name = x.to_string();
                            let namespace = elt["spec"]["labels"]["io.kubernetes.pod.namespace"]
                                .as_str()
                                .unwrap();
                            if name.eq(p[1]) && namespace.eq(p[0]) {
                                map.insert(
                                    pod.as_str().unwrap().to_string(),
                                    elt["name"].as_str().unwrap().to_string(),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    map
}

/**
 * params: node_ip: ip of the master node, node_port: port of the master node
 * returns: the global ram usage of the worker node
 */
fn get_global_ram(node_ip: String, node_port: String) -> f64 {
    let json: serde_json::Value =
        ureq::get(format!("http://{}:{}/api/v1.3/machine", node_ip, node_port).as_str())
            .call()
            .unwrap()
            .into_json()
            .unwrap();

    json["memory_capacity"].as_f64().unwrap()
}

/**
 * params: map of the metrics of a certain container
 * returns: the id of the list of metrics to be able to navigate through the map
 */
// fn get_prefix(map: serde_json::Value) -> String {
//     let a: Vec<_> = map
//         .as_object()
//         .unwrap()
//         .iter()
//         .map(|(key, _)| key)
//         .collect();

//     a[0].to_string()
// }

/**
 * params: container: list of metrics (past 10s) of a container, node_ip: ip of the master node, node_port: port of the master node
 * returns: valuess of cpu and ram ratios
 * calculate the average of the usage of ram and cpu of 10s of usage
 */
fn calculate_metrics_cpu_ram(
    container: serde_json::Value,
    node_ip: String,
    node_port: String,
) -> (f64, f64) {
    let mut cpu_usage: f64 = 0.0;
    let mut ram_usage: f64 = 0.0;

    let stats = container["stats"].as_array();
    let mut i: f64 = 0.0;

    let global_ram = get_global_ram(node_ip, node_port);
    if let Some(s) = stats {
        for e in s.iter() {
            let cpu_usage_t = e["cpu"]["load_average"].as_f64();
            let ram_usage_t = e["memory"]["usage"].as_f64();
            if let Some(a) = cpu_usage_t {
                i += 1.0;
                cpu_usage += a;
            }
            if let Some(a) = ram_usage_t {
                ram_usage += a;
            }
        }
        cpu_usage /= i;
        ram_usage /= i;
    }

    (cpu_usage, ram_usage / global_ram)
}

/**
 * params: container: list of metrics (past 10s) of a container
 * returns: bandwidth values
 */
fn calculate_metrics_bandwidth(container: serde_json::Value) -> f64 {
    let mut rx_bytes_l: f64 = 0.0;
    let mut rx_bytes_f: f64 = 0.0;
    let mut tx_bytes_f: f64 = 0.0;
    let mut tx_bytes_l: f64 = 0.0;
    let mut time_l: DateTime<FixedOffset> =
        DateTime::parse_from_rfc2822("Wed, 18 Feb 2015 23:16:09 GMT").unwrap();
    let mut time_f: DateTime<FixedOffset> =
        DateTime::parse_from_rfc2822("Wed, 18 Feb 2015 23:16:09 GMT").unwrap();

    let stats = container["stats"].as_array();
    match stats {
        Some(s) => {
            let first = s.iter().next();
            let last = s.last();
            if let Some(first) = first {
                if let Some(s) = first["network"]["interfaces"].as_array() {
                    for e in s {
                        rx_bytes_f += e["rx_bytes"].as_f64().unwrap_or_default();
                    }
                }
                if let Some(s) = first["network"]["interfaces"].as_array() {
                    for e in s {
                        tx_bytes_f += e["tx_bytes"].as_f64().unwrap_or_default();
                    }
                }
                time_f =
                    DateTime::parse_from_rfc3339(first["timestamp"].as_str().get_or_insert("0"))
                        .unwrap()
            }

            if let Some(last) = last {
                if let Some(s) = last["network"]["interfaces"].as_array() {
                    for e in s {
                        rx_bytes_l += e["rx_bytes"].as_f64().unwrap_or_default();
                    }
                }
                if let Some(s) = last["network"]["interfaces"].as_array() {
                    for e in s {
                        tx_bytes_l += e["tx_bytes"].as_f64().unwrap_or_default();
                    }
                }
                time_l =
                    DateTime::parse_from_rfc3339(last["timestamp"].as_str().get_or_insert("0"))
                        .unwrap();
            }
        }
        None => println!("error"),
    }

    let delta_t = (time_l.timestamp() - time_f.timestamp()) as f64;
    let delta_rx = rx_bytes_l - rx_bytes_f;
    let delta_tx = tx_bytes_l - tx_bytes_f;

    ((delta_rx * 0.000001 / delta_t) + delta_tx * 0.000001 / delta_t) / 2.0
}

/**
 * params: cpu: cpu ratio, ram: ram ratio, net: bandwidth ratio, constants: A1,A2 and A3 to determine the importance of each ratio
 * params: the score assingned to a container based on an equation
 */
fn calculate_score(cpu: f64, ram: f64, net: f64, constants: (String, String, String)) -> f64 {
    let (a1, a2, a3) = constants;

    a1.parse::<f64>().unwrap() * cpu
        + ram * a2.parse::<f64>().unwrap()
        + a3.parse::<f64>().unwrap() * net
}

/**
 * params: vector of scores pods names and their calculated scores
 * returns: the pod which had the best score
 */
fn min(v: Vec<(String, f64)>) -> (String, f64) {
    let mini = v.get(0);
    match mini {
        Some(mini) => {
            let (mut ms, mut mf) = mini.clone();
            for (s, f) in v {
                if mf > f {
                    ms = s;
                    mf = f;
                }
            }

            (ms, mf)
        }
        None => ("".to_string(), 0.0),
    }
}

/**
 * params: conn: connection with redis db, node_ip: ip of the master node, node_port: port of the master node, constants: A1, A2 and A3 given as env variables
 * returns: the best scored pod of a replicaset with its score and ip (loop on every replicaset of the k8s cluster)
 * this function is practically the dynamic load balancer
 */
pub fn load_balance(
    conn: &mut Connection,
    node_ip: String,
    node_port: String,
    constants: (String, String, String),
) -> HashMap<String, (String, f64)> {
    let hosts: String = conn.get("hosts").unwrap(); // get list of deployments
    let a = deserialize_metrics(hosts);
    let arr = a.as_array().unwrap();
    let nodes: Vec<&str> = node_ip.split(",").collect();
    let ww = get_container_id(conn.borrow_mut(), node_ip.clone(), node_port.clone());

    let mut balancer: HashMap<String, (String, f64)> = HashMap::new();
    for e in arr.iter() {
        let replicaset = deserialize_metrics(conn.get(e.as_str().unwrap()).unwrap()); // get pods of the same deployment
        let mut a: Vec<(String, f64)> = vec![];
        for l in replicaset.as_array().iter() {
            for i in l.iter() {
                let pod = i.as_str().unwrap();
                let x: String = conn.get(pod).unwrap();
                let data = deserialize_metrics(x);
                // for w in data["containers"].as_array().iter() {
                let name = data["name"].as_str();
                if let Some(name) = name {
                    for node in nodes.iter() {
                        let container = ww.get(name);
                        if let Some(container) = container {
                            let metrics = get_metrics(
                                container.to_string(),
                                node.to_string(),
                                node_port.clone(),
                            );
                            if let Ok(metrics) = metrics {
                                let (cpu, ram) = calculate_metrics_cpu_ram(
                                    metrics.clone(),
                                    node.to_string(),
                                    node_port.clone(),
                                );
                                let net = calculate_metrics_bandwidth(metrics.clone());
                                let score = calculate_score(cpu, ram, net, constants.clone());

                                // println!("{} ===> cpu={}, ram={}, band={}", i, cpu, ram, net);
                                store_metrics(conn, format!("{}-metrics", name), (cpu, ram, net));
                                let ip = data["ip"].as_str();
                                if let Some(ip) = ip {
                                    a.push((ip.to_string(), score));
                                }
                            }
                        }
                    }
                }
                // }
            }
        }
        let m = min(a);
        balancer.insert(e.as_str().unwrap().to_string(), m);
    }

    balancer
}
