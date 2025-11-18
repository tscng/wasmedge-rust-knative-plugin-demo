WasmEdge Rust KNative with Plugin Demonstration
===
This repository verifies a KNative setup for usage of Wasm container services that use WasmEdge plugins.
For CI/CD setup refer to [tscng/wasmedge-rust-plugin-demo](https://github.com/tscng/wasmedge-rust-plugin-demo).  
[tscng/wasmedge-rust-knative-demo](https://github.com/tscng/wasmedge-rust-knative-demo) contains a demo for using Wasm containers on MicroK8s + KWasm; however, this setup failed to load plugins.  
Furthermore, KWasm uses outdated versions of both the runwasi shim and wasmedge.  
Therefore, to achieve the goal of using plugins on Microk8s + KNative, a manual setup with crun is used.

## Microk8s + crun
This setup uses Ubuntu 22.04 in WSL. Check required dependencies/commands with the provided sources.
- Prepare: `sudo apt update`

### [Build and install WasmEdge](https://wasmedge.org/docs/contribute/source/os/linux) locally
This enables potential modification of the plugin source code as used later.
- Clone: `git clone https://github.com/WasmEdge/WasmEdge.git && cd WasmEdge && mkdir build`
- Dependencies: `sudo apt install -y software-properties-common cmake llvm-14-dev liblld-14-dev gcc g++`
- (Optional: Modify the plugin sources. Eg: Remove check for allow-command flag in process plugin.)
- Configure - select plugins to be used: `cmake -GNinja -Bbuild -DCMAKE_BUILD_TYPE=Release -DWASMEDGE_PLUGIN_PROCESS=On`
- Build: `cmake --build build`
- Install for all users: `sudo cmake --install build`

### [Build and install crun](https://wasmedge.org/docs/develop/deploy/oci-runtime/crun/) locally
Crun will be used as container runtime. It can be built to support wasm containers using wasmedge. 
- Clone: `git clone https://github.com/containers/crun && cd crun`
- Dependencies: `sudo apt install -y make git gcc build-essential pkgconf libtool libsystemd-dev libprotobuf-c-dev libcap-dev libseccomp-dev libyajl-dev go-md2man libtool autoconf python3 automake`
- Configure: `./autogen.sh && ./configure --with-wasmedge`
- Build: `make`
- Install: `sudo make install`

### Install and set up microk8s
- Install: `sudo snap install microk8s --classic`
- Join group: `sudo usermod -a -G microk8s $USER`
- Enable kube dir: `mkdir -p ~/.kube && chmod 0700 ~/.kube`
- Apply to session: `su - $USER`

### Patch containerd configuration
By default, microk8s uses containerd + runc runtime.  
To run Wasm containers, the previously installed crun will be used instead. 
[The wasmedge docs show](https://wasmedge.org/docs/develop/deploy/cri-runtime/containerd-crun/) how to patch the default containerd config.
The `containerd-patch.toml` contains a relevant snippet that will be used in the following.
- Edit the config template: `nano /var/snap/microk8s/current/args/containerd-template.toml`
- Remove all existing `[plugins."io.containerd.grpc.v1.cri".containerd]` values
- Paste the mentioned snippet instead
- Restart microk8s: `microk8s stop && microk8s start && microk8s status --wait-ready`

### Enable KNative and prepare plugins
Knative will be used to serve Wasm container services.  
Since Knative does not allow inline volumes, a configmap of the directory containing the plugins will be created.  
- Enable community addons: `microk8s enable community`
- Enable knative: `microk8s enable knative`
- Create configmap containing plugins: `microk8s kubectl create configmap wasmedge-plugins --from-file=/usr/local/lib/wasmedge`

### Deploy KNative service
The service uses the `wasmedge-plugins` configmap to mount the plugins in the container path `/plugin`.  
Correspondingly, the `WASMEDGE_PLUGIN_PATH` is set to the container path `/plugin`.
In order for crun to identify the wasm runtime, the `module.wasm.image/variant` annotation needs to be set at the pod.
- Apply service: `microk8s kubectl apply -f https://github.com/tscng/wasmedge-rust-knative-plugin-demo/releases/latest/download/service.yml`
- Test service with [correct address](https://github.com/tscng/wasmedge-rust-knative-demo): `curl -H "Host: <hostname>" http:/<ip>/service`  

This will return an error message, because the `echo` binary is not found on the wasm container.  
However, it verifies the WasmEdge plugin setup on microk8s and KNative.