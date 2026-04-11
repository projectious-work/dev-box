---
name: kubernetes-basics
description: |
  Kubernetes cluster management — resources, networking, storage, Helm, troubleshooting. Use when writing Kubernetes manifests, running kubectl or helm, debugging pods and services, or configuring ingress, DNS, storage, or network policies.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-kubernetes-basics
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: devops
---

# Kubernetes Basics

## Intro

Kubernetes is a declarative system: you write manifests, the API
server stores them in etcd, and controllers reconcile reality toward
that desired state. Confirm the active context, prefer declarative
`apply` over imperative commands, and always check events and logs
before poking at YAML.

## Overview

### Cluster context

Always confirm the active cluster and namespace before making changes:

```bash
kubectl config current-context
kubectl config get-contexts
kubectl get ns
```

Switch context/namespace explicitly rather than relying on defaults:

```bash
kubectl config use-context <name>
kubectl config set-context --current --namespace=<ns>
```

### Core resources

Use Deployments for stateless workloads and StatefulSets for stateful
ones. Never create bare Pods in production — they are not rescheduled
on failure.

- **Pod** — smallest deployable unit, one or more containers
- **Deployment** — manages ReplicaSets, handles rolling updates
- **StatefulSet** — stable network identity and persistent storage
  per replica
- **DaemonSet** — one pod per node (logging, monitoring agents)
- **Job / CronJob** — run-to-completion and scheduled tasks

### Networking

Services expose pods internally or externally:

- **ClusterIP** (default) — internal only
- **NodePort** — exposes on each node's IP at a static port
- **LoadBalancer** — provisions an external LB (cloud providers)

Ingress routes external HTTP/HTTPS to services by host/path. Always
set `ingressClassName`. Pods resolve services via DNS at
`<svc>.<ns>.svc.cluster.local`.

Network policies default to allow-all. Define explicit ingress/egress
rules to restrict traffic between namespaces or pods.

### Storage

- **PersistentVolume (PV)** — cluster-level storage resource
- **PersistentVolumeClaim (PVC)** — namespace-scoped request for
  storage
- **StorageClass** — dynamic provisioning template

Use `storageClassName` in PVCs. For StatefulSets, use
`volumeClaimTemplates` to create per-replica PVCs automatically.
Always set `accessModes` and `resources.requests.storage`.

### Configuration

- **ConfigMap** — non-sensitive key-value config; mount as files or
  env vars
- **Secret** — base64-encoded sensitive data; same mount options

Prefer mounting as volumes over env vars for config files. Use
`kubectl create secret generic` for quick creation. In production, use
an external secrets operator or sealed-secrets.

### Helm

Helm manages templated releases:

```bash
helm repo add <name> <url>
helm repo update
helm search repo <chart>
helm install <release> <chart> -n <ns> --create-namespace -f values.yaml
helm upgrade <release> <chart> -n <ns> -f values.yaml
helm rollback <release> <revision> -n <ns>
helm list -n <ns>
helm uninstall <release> -n <ns>
```

Inspect before installing: `helm template` to render, `helm show
values` for defaults. Pin chart versions in CI with `--version`.

### Troubleshooting order

When debugging, start broad and narrow down:

1. `kubectl get events --sort-by=.lastTimestamp` — cluster events
2. `kubectl describe pod <pod>` — scheduling, image pull, probes
3. `kubectl logs <pod> [-c container] [--previous]` — app logs
4. `kubectl exec -it <pod> -- sh` — interactive shell
5. `kubectl top pod` / `kubectl top node` — resource usage

### Applying changes safely

Always `kubectl diff` before `kubectl apply` to preview changes.
Prefer declarative `apply` over imperative `create`/`edit`. Use
`--dry-run=client -o yaml` to validate without submitting. For
production rollouts, watch with `kubectl rollout status` and reach
for `kubectl rollout undo` if something goes wrong.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Not confirming the active context before making changes.** `kubectl apply -f` operates against whichever cluster context is currently active. Running a destructive operation in production because the context was left pointing at prod from a previous debugging session is among the most common Kubernetes incidents. Always run `kubectl config current-context` before any write operation, and use a tool or alias that shows the context in the prompt.
- **Bare Pods in production instead of Deployments.** A bare Pod that crashes is not rescheduled — it stays in `Error` or `CrashLoopBackOff` until manually deleted and recreated. A Deployment's ReplicaSet controller reschedules it automatically and supports rolling updates and rollbacks. Never create bare Pods for any workload that must stay up.
- **Missing readiness probes on Services.** A Service will send traffic to all pods that match its selector, including pods that are starting, initializing, or currently unhealthy but have not yet been killed by the liveness probe. Without a readiness probe, traffic hits unready pods and users see errors. The liveness probe kills bad pods; the readiness probe keeps traffic away from them until they are ready.
- **No resource requests or limits.** Without `resources.requests`, the scheduler has no signal for node placement and a noisy pod can be scheduled on an already-overloaded node. Without `resources.limits`, a memory-leaking pod can OOMKill neighboring pods on the same node. Set both on every container; start with generous values, observe with `kubectl top`, then tighten.
- **`latest` image tags in Deployment manifests.** A Deployment with `image: myapp:latest` will pull whatever `latest` resolves to at each restart — silently deploying a different version than the one you tested. Always pin to a specific immutable tag (typically a Git SHA or build number) so rollouts are deterministic and `kubectl rollout undo` restores the exact previous version.
- **Blind `kubectl apply` without reviewing `kubectl diff` first.** `kubectl apply` without a preceding `kubectl diff` is a production change made without preview. Fields set out-of-band (via `kubectl edit`, autoscalers, or admission webhooks) may be silently overwritten. Always `kubectl diff -f manifest.yaml` before `kubectl apply -f manifest.yaml` in production.
- **Using `-target` or imperative `kubectl edit` as a normal workflow.** Imperative edits (`kubectl edit`, `kubectl scale`) and `-target` applies produce configuration that diverges from the version-controlled manifests. The next declarative apply will overwrite the change without warning. All production changes must go through version-controlled manifests applied via CI/CD.

## Full reference

### Control plane architecture

```
kubectl apply -f deployment.yaml
    |
    v
API Server: validates, stores in etcd
    |
    v
Deployment controller -> ReplicaSet -> Pod objects (Pending)
    |
    v
Scheduler: assigns Pods to nodes (filtering + scoring)
    |
    v
kubelet: pulls image, starts containers on the node
    |
    v
kube-proxy: updates iptables/IPVS rules for Service endpoints
```

Components:

- **API server** — front door; validates and persists to etcd
- **etcd** — distributed KV store holding all cluster state; HA with
  3 or 5 members; back up with `etcdctl snapshot save`
- **Scheduler** — filters then scores nodes for unscheduled pods
- **Controller manager** — runs reconcile loops (ReplicaSet,
  Deployment, Node, Job, Endpoint, ServiceAccount controllers)
- **kubelet** — node agent; manages Pod lifecycle via the CRI
- **kube-proxy** — programs iptables/IPVS/nftables for Services
- **Container runtime** — containerd or CRI-O; Docker Engine was
  removed as a direct runtime in v1.24

### Resource YAML cheatsheet

**Deployment with resources, probes, config, storage:**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp
spec:
  replicas: 3
  selector:
    matchLabels:
      app: myapp
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  template:
    metadata:
      labels:
        app: myapp
    spec:
      containers:
        - name: app
          image: myapp:1.0.0
          ports:
            - containerPort: 8080
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 256Mi
          livenessProbe:
            httpGet:
              path: /healthz
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /ready
              port: 8080
            initialDelaySeconds: 3
            periodSeconds: 5
          envFrom:
            - configMapRef:
                name: myapp-config
          volumeMounts:
            - name: data
              mountPath: /data
      volumes:
        - name: data
          persistentVolumeClaim:
            claimName: myapp-data
```

**Service types:**

```yaml
apiVersion: v1
kind: Service
metadata:
  name: myapp
spec:
  selector:
    app: myapp
  ports:
    - port: 80
      targetPort: 8080
---
apiVersion: v1
kind: Service
metadata:
  name: myapp-lb
spec:
  type: LoadBalancer
  selector:
    app: myapp
  ports:
    - port: 443
      targetPort: 8080
```

**Ingress with TLS:**

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: myapp
spec:
  ingressClassName: nginx
  tls:
    - hosts: [myapp.example.com]
      secretName: myapp-tls
  rules:
    - host: myapp.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: myapp
                port:
                  number: 80
```

**ConfigMap, Secret, PVC:**

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: myapp-config
data:
  APP_ENV: production
  LOG_LEVEL: info
---
apiVersion: v1
kind: Secret
metadata:
  name: myapp-secret
type: Opaque
stringData:
  DB_PASSWORD: s3cret
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: myapp-data
spec:
  accessModes: [ReadWriteOnce]
  storageClassName: standard
  resources:
    requests:
      storage: 10Gi
```

**Job and CronJob:**

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: nightly-backup
spec:
  schedule: "0 2 * * *"
  concurrencyPolicy: Forbid
  successfulJobsHistoryLimit: 3
  failedJobsHistoryLimit: 3
  jobTemplate:
    spec:
      template:
        spec:
          restartPolicy: OnFailure
          containers:
            - name: backup
              image: myapp:1.0.0
              command: ["./backup.sh"]
```

### kubectl cheatsheet

```bash
# Viewing
kubectl get pods -n <ns> -o wide
kubectl get all -n <ns>
kubectl describe pod <pod> -n <ns>
kubectl get events --sort-by=.lastTimestamp

# Applying
kubectl apply -f manifest.yaml
kubectl apply -f ./manifests/ -R
kubectl diff -f manifest.yaml
kubectl create secret generic db \
  --from-literal=password=s3cret -n <ns>

# Debugging
kubectl logs <pod> -n <ns>
kubectl logs <pod> -c <container> --previous
kubectl logs -l app=myapp -n <ns> --tail=100
kubectl exec -it <pod> -n <ns> -- sh
kubectl port-forward svc/myapp 8080:80 -n <ns>
kubectl top pod -n <ns>
kubectl top node

# Rollouts
kubectl scale deploy myapp --replicas=5 -n <ns>
kubectl rollout status deploy/myapp -n <ns>
kubectl rollout history deploy/myapp -n <ns>
kubectl rollout undo deploy/myapp -n <ns>
kubectl rollout restart deploy/myapp -n <ns>

# Deleting
kubectl delete -f manifest.yaml
kubectl delete pod <pod> --grace-period=0 --force
```

### Troubleshooting patterns

**Pod Pending (not scheduled).** Read the events:

```bash
kubectl describe pod <pod> -n <ns>   # look at Events section
```

| Event message | Cause | Fix |
|---|---|---|
| `Insufficient cpu/memory` | Node full | Scale cluster, reduce requests |
| `node(s) had taint` | Taint/toleration mismatch | Add toleration or remove taint |
| `no persistent volumes available` | PVC cannot bind | Check PV, StorageClass, access modes |
| `didn't match Pod's node affinity/selector` | Scheduling constraint | Fix selector or label nodes |

**CrashLoopBackOff.** Get logs from the crashed container:

```bash
kubectl logs <pod> -n <ns> --previous
kubectl get pod <pod> -n <ns> \
  -o jsonpath='{.status.containerStatuses[0].lastState.terminated}'
```

Exit code 137 is OOMKilled — raise the memory limit. Exit code 1 with
no logs usually means a bad `command`/`args`. A failing liveness
probe restarts the container on a loop.

**ImagePullBackOff.** Almost always a wrong tag, a private registry
without `imagePullSecrets`, or Docker Hub rate limiting:

```bash
kubectl create secret docker-registry regcred \
  --docker-server=ghcr.io \
  --docker-username=<user> \
  --docker-password=<token> \
  -n <ns>
```

Reference it in the pod spec via `spec.imagePullSecrets`.

**Service not reachable.** Endpoints tell you if the selector matches
any pods:

```bash
kubectl get svc <svc> -n <ns>
kubectl get endpoints <svc> -n <ns>
# Empty endpoints = selector mismatch OR no pods are Ready
kubectl get pods -n <ns> --show-labels
kubectl run curl --image=curlimages/curl --restart=Never --rm -it -- \
  curl -v http://<svc>.<ns>.svc.cluster.local:<port>
```

Check: selector labels match pod labels exactly, `targetPort` matches
the container's listening port, readiness probe is passing,
NetworkPolicy is not blocking.

**DNS issues inside the cluster.**

```bash
kubectl run dnstest --image=busybox:1.36 --restart=Never --rm -it -- \
  nslookup <svc>.<ns>.svc.cluster.local
kubectl get pods -n kube-system -l k8s-app=kube-dns
kubectl logs -n kube-system -l k8s-app=kube-dns --tail=50
```

`ndots:5` can cause slow external lookups — use a fully-qualified name
with a trailing dot to force a direct lookup.

**Resource limits and OOM.**

- `requests` = typical usage, used for scheduling
- `limits` = maximum allowed; memory overrun is OOMKill, CPU overrun
  is throttling
- Start with generous limits, monitor with `kubectl top`, then tighten
- CPU limits are often best omitted — throttling is less harmful than
  OOMKill

```bash
kubectl top pod -n <ns> --containers
kubectl get pods -n <ns> -o json | \
  jq '.items[] | select(.status.containerStatuses[]?.lastState.terminated.reason=="OOMKilled") | .metadata.name'
```

**Node pressure and eviction.**

```bash
kubectl describe node <node> | grep -A 5 Conditions
# Common: MemoryPressure, DiskPressure, PIDPressure
kubectl get pods -n <ns> --field-selector=status.phase=Failed | grep Evicted
```

### Helm upgrade with rollback safety

```bash
helm upgrade myrelease mychart/app -n prod -f prod-values.yaml \
  --atomic --timeout 5m
# --atomic auto-rolls back on failure
helm history myrelease -n prod
```

### Debug containers and RBAC checks

```bash
# Ephemeral debug container (K8s 1.23+)
kubectl debug -it <pod> -n <ns> --image=busybox:1.36 --target=<container>

# Copy a running pod
kubectl debug <pod> -n <ns> --copy-to=debug-pod --container=app -- sh

# RBAC checks
kubectl auth can-i create deployments -n <ns>
kubectl auth can-i '*' '*' --all-namespaces
kubectl auth can-i get pods --as=system:serviceaccount:<ns>:<sa>
```

### Anti-patterns

- **Bare Pods in production** — use Deployments or StatefulSets
- **Missing readiness probes** — Services will send traffic to pods
  that aren't ready
- **No resource requests/limits** — scheduler has nothing to go on,
  noisy neighbors can starve you
- **Hardcoded namespace in manifests** — pass via `-n` or kustomize
- **`latest` image tags** — rollouts become non-deterministic
- **Ignoring `kubectl diff`** — blind `apply` can silently remove
  fields that were set out-of-band
