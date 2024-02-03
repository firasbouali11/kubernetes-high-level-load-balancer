use redis::{Commands, Connection};

/**
 * params: host: hostname of the redis service
 * returns: an object of the connections with redis db
 */
pub fn connect_db(host: &str) -> Connection {
    let client = redis::Client::open(format!("redis://{}", host)).unwrap();
    println!("db connected successfuly");
    client.get_connection().unwrap()
}

/**
 * params: cpu: cpu ratio, ram: ram ratio, bandwidth: bandwidth ratio
 * returns: a stringified json containing the given ratios
 */
fn serialize_metrics(pod: String, cpu: f64, ram: f64, bandwidth: f64) -> String {
    let data = format!(
        r#"{{
        "pod":"{}","cpu":{}, "ram":{}, "bandwidth":{}
    }}"#,
        pod, cpu, ram, bandwidth
    );
    data
}

/**
 * params: :conn: Connection with redis db, key: key for the redis object => pod name + "-metrics", value: (cpu,ram,bandwidth)
 * stores the metrics in the db
 */
pub fn store_metrics(conn: &mut Connection, key: String, value: (f64, f64, f64)) {
    let (cpu, ram, bandwidth) = value;
    let data = serialize_metrics(key.clone(), cpu, ram, bandwidth);
    let _: () = conn.set(key, data).unwrap();
}

/**
 * params: data: stringified json retreived from the db
 * returns: serde_json value
 */
pub fn deserialize_metrics(data: String) -> serde_json::Value {
    let json: serde_json::Value = serde_json::from_str(data.as_str()).unwrap_or_default();
    json
}
