use redis::{Commands, Connection};
use std::io::Write;
use std::net::TcpStream;

use crate::db::deserialize_metrics;

pub fn generate_upstream(conn: &mut Connection, host: String, algo: String) -> String {
    let servers = deserialize_metrics(conn.get(host.clone()).unwrap());
    let port: String = conn.get(host.clone() + "-port").unwrap_or_default();
    if !port.is_empty() {
        let mut list_servers: Vec<String> = vec![];
        if !algo.eq("rr") {
            list_servers.push(algo + ";")
        }
        servers.as_array().into_iter().for_each(|server| {
            for s in server {
                let data = deserialize_metrics(conn.get(s.as_str().unwrap()).unwrap());
                let ip = data["ip"].as_str().unwrap();
                let a = format!("server {}:{};", ip, port);
                list_servers.push(a);
            }
        });
        let upstream = format!(
            r#"
                upstream {}stream{{
                    {}
                }}
            "#,
            host,
            list_servers.join("\n")
        );
        upstream
    } else {
        String::from("")
    }
}

/**
 * params: conn: Connection with redis db, ip: ip of the pod, host: hostname of the application (name of the deployment by default)
 * returns: string containing all the locations for the nginx to forward the request to
 */
pub fn add_location(conn: &mut Connection, ip: String, host: String, from_where: String) -> String {
    if !from_where.eq("upstream") {
        let port: String = conn.get(host.clone() + "-port").unwrap_or_default();
        if !port.is_empty() {
            let location_template = format!(
                r#"
                location /{}/ {{
                    proxy_pass http://{}:{}/;
                }}
            "#,
                host, ip, port
            );
            location_template
        } else {
            "".to_string()
        }
    } else {
        let location_template = format!(
            r#"
            location /{}/ {{
                proxy_pass http://{}/;
            }}
        "#,
            host, ip
        );
        location_template
    }
}

/**
 * params: locations: list of the locations for the nginx to serve the request
 * this function generates the content of the configuration file for nginx to read and store the content in a file called hello.conf
 */

pub fn nginx_template_generator(locations: Vec<String>, upstreams: Vec<String>) {
    let l = locations.join("\n");
    let u = upstreams.join("\n");
    let template = format!(
        r#"
        {}
        server {{
            listen       443 ssl;
            listen  [::]:443 ssl;
            server_name  localhost;

            {}

            location / {{
                root   /usr/share/nginx/html;
                index  index.html index.htm;
            }}

            error_page   500 502 503 504  /50x.html;
            location = /50x.html {{
                root   /usr/share/nginx/html;
            }}

        }}
    "#,
        u, l
    );

    match std::fs::write("/conf/hello.conf", template) {
        Ok(_) => println!("write done"),
        Err(e) => println!("{:?}", e),
    }
}

/**
 * params: host: hostname of the sverer that triggers nginx to reload, port: port of the server
 * this function opens a tcp connection between the agent and a server inside nginx pod to trigger it to hot reload the configuration by sending a message
 */
pub fn trigger_nginx(host: &str, port: &str) {
    match TcpStream::connect(format!("{}:{}", host, port)) {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 3333");

            let msg = b"Hello!";

            stream.write(msg).unwrap_or_default();
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
}
