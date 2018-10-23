use qube::prelude::*;
use std::io::{self, Write};
use tokio::run;
use futures::{Future, Stream};
use rand::Rng;
use colored::*;
use rand::thread_rng;
use std::str::*;
use std::sync::{Arc, RwLock};
use carboxyl::*;

use core::poddata::*;

pub fn launch_logger(kube: Kubernetes, namespace: String, pod: Arc<RwLock<PodData>>, sink: Sink<PodData>) {
    let podread = pod.read().unwrap();

    let pod_name = podread.name;
    let container_name = podread.container;

    drop(podread); // drop read lock

    let colors: [String; 8] = [
        String::from("blue"),
        String::from("red"),
        String::from("green"),
        String::from("yellow"),
        String::from("cyan"),
        String::from("purple"),
        String::from("magenta"),
        String::from("white")
    ];

    if let Ok(res) = kube.pods().namespace(&*namespace).logs().fetch_container_future(pod_name, container_name) {
        let runfut = res
            .send()
            .and_then(move |mut res| {
                let name = format!("P={} C={} ::⇒ ", pod_name, container_name);
                let color = thread_rng().choose(&colors).unwrap();
                let resname = name.color(color.to_string());

                res
                    .into_body()
                    .for_each(move |chunk| {
                        let stdout = io::stdout();
                        let mut handle = stdout.lock();

                        let data = format!("{}{}\n", resname, from_utf8(&chunk).unwrap().trim_right());

                        handle
                            .write_all(&data.as_bytes())
                            .map_err(|e| {
                                panic!("stdout expected to be open, error={}", e)
                            })
                    })
            })
            .map_err(|err| println!("request error: {}", err));

        run(runfut); // Block like a boss!
    }

    let poddata = PodData { name: pod_name, container: container_name, status: PodStatus::PodUnreachable };
    sink.send(poddata)
}
