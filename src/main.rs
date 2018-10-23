extern crate dirs;
extern crate qube;
extern crate backtrace;
extern crate carboxyl;
extern crate clap;
extern crate tokio;
extern crate futures;
extern crate reqwest;
extern crate rand;
extern crate colored;

use qube::clients::*;
use qube::errors::*;
use std::env;
use std::{mem, thread, time};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use carboxyl::*;
use clap::{Arg, App};


mod core;

use core::poddata::*;
use core::logtailer::*;

fn upstream_generator(kube: Kubernetes, namespace: String, sink: Sink<PodData>) -> Result<i32> {
    let poll_int = time::Duration::from_millis(50);
    let emit_int = time::Duration::from_millis(5);
    const RUN_PHASE: &'static str = "Running";

    loop {
        if kube.healthy()? {
            for pod in kube.pods().namespace(&*namespace).list(None)? {
                let name = pod.metadata.name.unwrap();
                // println!("STATUS {:?}", pod.status.clone().unwrap().phase);
                if let Some(pod_status) = pod.status {
                    if let Some(phase) = pod_status.phase {
                        if phase == RUN_PHASE {
                            for c_status in pod_status.container_statuses.unwrap() {
                                // println!("STATE {:?}", c_status.state.unwrap().running);
                                if let Some(_running) = c_status.state.unwrap().running {
                                    let pod_name = name.clone();
                                    let container_name = c_status.name;

                                    let poddata = PodData {
                                        name: Box::leak(pod_name.into_boxed_str()),
                                        container: Box::leak(container_name.into_boxed_str()),
                                        status: PodStatus::PodReachable
                                    };

                                    thread::sleep(emit_int);
                                    sink.send(poddata);
                                }
                            }
                        }
                    }
                }
            }
        }

        thread::sleep(poll_int);
    }
}


fn register_client() -> Result<qube::Kubernetes> {
    // filename is set to $KUBECONFIG if the env var is available.
    let filename = env::var("KUBECONFIG").ok();
    let empty_buf = PathBuf::new();
    let home_dir: PathBuf = dirs::home_dir().unwrap_or(empty_buf);
    let default_config_path = format!("{}/.kube/config", home_dir.to_str().unwrap());
    let default_config = Path::new(default_config_path.as_str()).to_str().unwrap();
    let filename = filename
        .as_ref()
        .map(String::as_str)
        .and_then(|s| if s.is_empty() { None } else { Some(s) })
        .unwrap_or(default_config);
    Ok(Kubernetes::load_conf(filename)?)
}


fn main() {
    let matches = App::new("K∅RQ")
                          .version("0.2.0")
                          .author("Mahmut Bulut <vertexclique@gmail.com>")
                          .about("Kubernetes Dynamic Log Tailing Utility")
                          .arg(Arg::with_name("namespace")
                               .short("n")
                               .long("namespace")
                               .help("Namespace for the cluster")
                               .takes_value(true))
                          .arg(Arg::with_name("filter")
                               .short("f")
                               .long("filter")
                               .required(true)
                               .help("Name parameter to filter for pods")
                               .takes_value(true))
                          .arg(Arg::with_name("container")
                               .short("c")
                               .long("container")
                               .help("Filter for container name in pods")
                               .takes_value(true))
                          .get_matches();

    let filter = matches.value_of("filter").unwrap_or("");

    let matches = matches.clone();
    let namespace = matches.value_of("namespace").unwrap_or("default").to_owned();

    let matches = matches.clone();
    let cfilter = matches.value_of("container").unwrap_or("");

    let kube: Kubernetes = register_client()
        .expect("Client couldn't instantiated!");

    let mut registry = Vec::<PodData>::new();

    if let Err(e) = kube.healthy() {
        panic!("Client authorization failure :: {:?}", e);
    }

    // Start event stream
    let sink = carboxyl::Sink::new();
    let stream = sink.stream();

    // Make this shitty shallow clone...
    // Shit code, use something feasible...
    let obs_kube = kube.clone();
    let obs_sink = sink.clone();
    let obs_ns = namespace.clone();

    let poller = thread::spawn(|| { upstream_generator(kube, obs_ns, sink) });

    let mut events = stream.events().filter(|&p| {
        p.name.contains(filter) && p.container.contains(cfilter)
    });

    let global_pod = Arc::new(RwLock::new(PodDataImpl::new()));

    while let Some(event) = events.next() {
        let kb = obs_kube.clone();
        let si = obs_sink.clone();
        let ns = namespace.clone();

        let watched_pod = global_pod.clone();

        {
            let mut wpwriter = watched_pod.write().unwrap();
            mem::replace(&mut *wpwriter, event.clone());
        }
        let wpreader = watched_pod.clone();

        // println!("EVENT {:?}", event);

        match event.status {
            PodStatus::PodReachable => {
                if registry.iter().position(|x| *x.name == *event.name && *x.container == *event.container).is_none() == true {
                    registry.push(event);
                    thread::spawn(|| { launch_logger(kb, ns, wpreader, si) });
                }
            }
            PodStatus::PodUnreachable => {
                let ec = event.clone();
                // println!("EC {:?}", ec);
                if let Some(index) = registry.iter().position(|x| *x.name == *ec.name && *x.container == *ec.container) {
                    registry.remove(index);
                }
            }
        }
    }

    if poller.join().is_err() {
        panic!("Long polling failure");
    }
}
