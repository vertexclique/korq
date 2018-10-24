![K∅RQ](https://i.imgur.com/VjF6Oz0.png)

**Kubernetes Dynamic Log Tailing Utility**

[![crates.io](https://meritbadge.herokuapp.com/korq)](https://crates.io/crates/korq)
[![Build Status](https://travis-ci.org/vertexclique/korq.svg?branch=master)](https://travis-ci.org/vertexclique/korq)
![Crates.io](https://img.shields.io/crates/l/korq.svg)
![Crates.io](https://img.shields.io/crates/d/korq.svg)
[![Master API docs](https://img.shields.io/badge/docs-master-green.svg)](https://docs.rs/korq)

K∅RQ is used for tailing pod logs concurrently and following groups at once. It was basically a need to follow logs during deployment and see how instances behave during and after deployment. This is the main motive behind K∅RQ.

## Installation

Start by installing K∅RQ with Cargo.

```
cargo install korq
```

Or download it from the release tag!

Check that cargo bin path is in your PATH.

K∅RQ first looks for Kubernetes configuration file after that it will look for either CA certificates, cluster side certificates or an auth provider token for client initialization. Before that you might want to set your environment variable for configuration file which can be done via environment variable. By default it is using: `$HOME/.kube/config`.

```bash
$ KUBECONFIG=$HOME/somepath/admin.conf
```

## Usage

After these steps you need to set your default project if you are going to use token. Access Token must be valid during execution. OFC. yep!

By default K∅RQ's namespace is `default`. You can pass this argument as a parameter to the command with `--namespace` flag.

For filtering the pods by name you can pass pods' base name to the `--filter` parameter.

Then you can invoke K∅RQ with:

```bash
korq --context <CONTEXT> --namespace <NAMESPACE> --filter <FILTER>
```

If you want to tail a specific container in pod group you can use:

```bash
korq --context <CONTEXT> --namespace <NAMESPACE> --filter <FILTER> --container <CONTAINER_FILTER>
```

**Both commands in short:**

```bash
korq -k <CONTEXT> -n <NAMESPACE> -f <FILTER>
```

```bash
korq -k <CONTEXT> -n <NAMESPACE> -f <FILTER> -c <CONTAINER_FILTER>
```

**Enjoy the ride!**
