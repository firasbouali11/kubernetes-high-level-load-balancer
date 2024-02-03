mod db;
mod load_balancer;
mod nginx_manipulator;

use db::connect_db;
use load_balancer::load_balance;
use nginx_manipulator::{add_location, generate_upstream, nginx_template_generator, trigger_nginx};
use redis::Commands;

use std::{borrow::BorrowMut, env, thread, time};
#[allow(non_snake_case)]
#[allow(unused_doc_comments)]
fn main() {
    thread::sleep(time::Duration::from_secs(10));

    /**
     * Get the necessary env variables from the os:
     * redis host name, nginx triggerer host name and port and constants for the equation
     */
    let mut REDIS_HOST = env::var("REDIS_HOST").unwrap_or_default();
    let mut SERVER_HOST = env::var("SERVER_TRIGGER_HOST").unwrap_or_default();
    let mut SERVER_PORT = env::var("SERVER_TRIGGER_PORT").unwrap_or_default();
    let mut NODE_IP = env::var("NODE_IP").unwrap_or_default();
    let mut NODE_PORT = env::var("NODE_PORT").unwrap_or_default();
    let mut A1 = env::var("A1").unwrap_or_default();
    let mut A2 = env::var("A2").unwrap_or_default();
    let mut A3 = env::var("A3").unwrap_or_default();

    /**
     * Default values if the env varialbles are not set
     */
    if NODE_IP.is_empty() {
        println!("no node ip !");
        NODE_IP = "localhost".to_string();
    }

    if A1.is_empty() && A2.is_empty() && A3.is_empty() {
        A1 = "0.33".to_string();
        A2 = "0.33".to_string();
        A3 = "0.33".to_string()
    }
    if NODE_PORT.is_empty() {
        NODE_PORT = "8080".to_string()
    }
    if REDIS_HOST.is_empty() {
        REDIS_HOST = "localhost".to_string()
    }

    if SERVER_HOST.is_empty() {
        SERVER_HOST = "127.0.0.1".to_string()
    }
    if SERVER_PORT.is_empty() {
        SERVER_PORT = "3333".to_string()
    }

    /**
     * Connection with the redis database
     */
    let mut conn = connect_db(REDIS_HOST.as_str());

    /**
     * Infinite loop for the agent to determine the best replica to forward the load to, every 5 seconds
     */
    loop {
        /**
         * Choose the best replica for each replicaset in the k8s cluster
         */
        let balancer = load_balance(
            conn.borrow_mut(),
            NODE_IP.clone(),
            NODE_PORT.clone(),
            (A1.clone(), A2.clone(), A3.clone()),
        );

        /**
         * Generate the nginx configuration file and and trigger nginx to hot reload
         */
        let mut locations: Vec<String> = Vec::new();
        let mut upstreams: Vec<String> = vec![];
        for (k, v) in balancer.iter() {
            let alg: String = conn.get(k.to_string() + "-algo").unwrap_or_default();
            if !v.0.is_empty() {
                if alg.is_empty() || alg.eq("custom") {
                    let s = add_location(
                        conn.borrow_mut(),
                        v.0.clone(),
                        k.to_string(),
                        "".to_string(),
                    );
                    if !s.is_empty() {
                        locations.push(s)
                    }
                } else {
                    let upstream = generate_upstream(conn.borrow_mut(), k.clone(), alg);
                    let s = add_location(
                        conn.borrow_mut(),
                        k.to_string() + "stream",
                        k.to_string(),
                        "upstream".to_string(),
                    );
                    if !upstream.is_empty() && !s.is_empty() {
                        upstreams.push(upstream);
                        locations.push(s)
                    }
                }
            }
        }

        // for u in upstreams {
        //     println!("{}",u);
        // }

        // println!("{:?}", locations.clone());
        // println!("{:?}", upstreams.clone());
        nginx_template_generator(locations, upstreams);

        trigger_nginx(&SERVER_HOST, &SERVER_PORT);

        // thread::sleep(time::Duration::from_secs(2));
    }
}
