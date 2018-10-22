extern crate dirs;
extern crate qube;
extern crate backtrace;
extern crate carboxyl;
extern crate clap;

use qube::clients::*;
use qube::prelude::*;
use qube::errors::*;
use std::env;
use std::{mem, thread, time};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use carboxyl::*;
use clap::{Arg, App};


mod poddata;

use self::poddata::*;

fn push_pods(kube: Kubernetes, sink: Sink<PodData>) -> Result<i32> {
    let poll_int = time::Duration::from_millis(50);

    loop {
        if kube.healthy()? {
            for pod in kube.pods().namespace("default").list(None)? {
                let name = pod.metadata.name.unwrap();
                let poddata = PodData {
                    name: Box::leak(name.into_boxed_str()),
                    status: PodStatus::PodReachable
                };
                sink.send(poddata);
            }
        }

        thread::sleep(poll_int);
    }
}

fn launch_logger(kube: Kubernetes, namespace: String, pod: Arc<RwLock<PodData>>, sink: Sink<PodData>) {
    let podread = pod.read().unwrap();
    let podname = podread.name;
    drop(podread);
    let _res = kube.pods().namespace(&*namespace).logs().fetch(podname);
    let poddata = PodData { name: podname, status: PodStatus::PodUnreachable };
    sink.send(poddata)
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
                          .version("0.1.0")
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
                          .get_matches();

    let filter = matches.value_of("filter").unwrap_or("");

    let matches = matches.clone();
    let namespace = matches.value_of("namespace").unwrap_or("default").to_owned();

    let kube: Kubernetes = register_client()
        .expect("Client couldn't instantiated!");

    let mut registry = Vec::<&'static str>::new();

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

    let poller = thread::spawn(|| { push_pods(kube, sink) });

    let mut events = stream.events().filter(|&p| {
        p.name.contains(filter)
    });

    let global_pod = Arc::new(RwLock::new(PodData{name: "", status: PodStatus::PodUnreachable}));

    while let Some(event) = events.next() {
        let kb = obs_kube.clone();
        let si = obs_sink.clone();
        let ns = namespace.clone();

        let watched_pod = global_pod.clone();

        let podname = event.name;
        {
            let mut wpwriter = watched_pod.write().unwrap();
            mem::replace(&mut *wpwriter, event.clone());
        }
        let wpreader = watched_pod.clone();

        match event.status {
            PodStatus::PodReachable => {
                if registry.iter().position(|x| *x == podname).is_none() == true {
                    registry.push(podname);
                    thread::spawn(|| { launch_logger(kb, ns, wpreader, si) });
                }
            }
            PodStatus::PodUnreachable => {
                let kname = podname.clone();
                if let Some(index) = registry.iter().position(|x| *x == kname.to_owned()) {
                    registry.remove(index);
                }
            }
        }
    }

    if poller.join().is_err() {
        panic!("Long polling failure");
    }
}
