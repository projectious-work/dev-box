# Kubernetes Cluster Architecture

## Control Plane Components

### API Server (kube-apiserver)

Front door to the cluster. All kubectl commands, internal components, and external
integrations talk to the API server over HTTPS. It validates and persists resource
state to etcd.

```
Client (kubectl) --> API Server --> etcd
                         ^
                         |
          Scheduler, Controller Manager, Kubelet
```

Key flags to know:
- `--service-cluster-ip-range` -- CIDR for ClusterIP services
- `--etcd-servers` -- etcd endpoint(s)
- `--admission-plugins` -- ordered list of admission controllers

### etcd

Distributed key-value store holding all cluster state. Every resource you `kubectl get`
is read from etcd via the API server.

- Runs as a cluster (3 or 5 members for HA)
- Back up regularly: `etcdctl snapshot save backup.db`
- Restore: `etcdctl snapshot restore backup.db`
- Check health: `etcdctl endpoint health --cluster`

### Scheduler (kube-scheduler)

Watches for unscheduled Pods and assigns them to nodes based on:

1. **Filtering** -- which nodes can run the pod (resource requests, taints/tolerations,
   node selectors, affinity rules, PV availability)
2. **Scoring** -- rank remaining nodes (spread, resource balance)

Influence scheduling with:

```yaml
# Node selector (simple)
nodeSelector:
  disktype: ssd

# Node affinity (expressive)
affinity:
  nodeAffinity:
    requiredDuringSchedulingIgnoredDuringExecution:
      nodeSelectorTerms:
        - matchExpressions:
            - key: topology.kubernetes.io/zone
              operator: In
              values: ["us-east-1a"]

# Tolerations (allow scheduling on tainted nodes)
tolerations:
  - key: "dedicated"
    operator: "Equal"
    value: "gpu"
    effect: "NoSchedule"
```

### Controller Manager (kube-controller-manager)

Runs control loops that reconcile desired state with actual state:

- **ReplicaSet controller** -- ensures correct pod count
- **Deployment controller** -- manages rollouts and ReplicaSets
- **Node controller** -- monitors node health, evicts pods from unhealthy nodes
- **Job controller** -- tracks job completions
- **Endpoint controller** -- populates Endpoints objects for Services
- **ServiceAccount controller** -- creates default ServiceAccounts

Each controller watches specific resource types via the API server and takes action
when actual state drifts from desired state.

## Node Components

### kubelet

Agent running on every node. Responsibilities:

- Registers the node with the API server
- Watches for Pod specs assigned to its node
- Manages container lifecycle via the container runtime (CRI)
- Runs liveness, readiness, and startup probes
- Reports node status and resource capacity

The kubelet does NOT manage containers not created by Kubernetes.

### kube-proxy

Handles service networking on each node. Implements Services by programming
iptables or IPVS rules to route traffic to pod endpoints.

Modes:
- **iptables** (default) -- creates NAT rules, random backend selection
- **IPVS** -- kernel-level load balancing, supports round-robin/least-conn/etc.
- **nftables** -- newer alternative to iptables mode

### Container Runtime

Runs the actual containers. Kubernetes uses the Container Runtime Interface (CRI):

- **containerd** -- most common, used by Docker and standalone
- **CRI-O** -- lightweight, designed specifically for Kubernetes
- Docker Engine was removed as a direct runtime in v1.24

## How They Interact

```
kubectl apply -f deployment.yaml
    |
    v
API Server: validates, stores Deployment in etcd
    |
    v
Deployment Controller: sees new Deployment, creates ReplicaSet
    |
    v
ReplicaSet Controller: sees ReplicaSet, creates Pod objects (Pending)
    |
    v
Scheduler: sees unscheduled Pods, assigns to nodes
    |
    v
kubelet (on target node): sees Pod assigned, pulls image, starts containers
    |
    v
kube-proxy: updates iptables/IPVS rules when Pod endpoints change
```

## Inspecting the Control Plane

```bash
# Check component health
kubectl get componentstatuses          # deprecated but still works on some clusters
kubectl get --raw='/healthz?verbose'

# Control plane pods (kubeadm clusters)
kubectl get pods -n kube-system

# Node status and capacity
kubectl get nodes -o wide
kubectl describe node <name>

# Cluster info
kubectl cluster-info
kubectl version --short

# API resources available
kubectl api-resources
kubectl api-versions
```

## High Availability Layout

Production clusters run multiple control plane replicas:

```
         Load Balancer (:6443)
        /       |       \
  API Server  API Server  API Server
       \        |        /
        etcd    etcd    etcd   (or external etcd cluster)
```

Worker nodes point their kubelet `--kubeconfig` at the load balancer address.
etcd can run stacked (on control plane nodes) or external (dedicated hosts).
