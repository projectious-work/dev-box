# Kubernetes Troubleshooting Guide

## General Debugging Flow

Always start broad and narrow down:

```bash
# 1. Cluster-level overview
kubectl get nodes
kubectl get events --sort-by=.lastTimestamp -A | head -30

# 2. Namespace-level overview
kubectl get all -n <ns>
kubectl get events --sort-by=.lastTimestamp -n <ns>

# 3. Resource-level detail
kubectl describe <resource> <name> -n <ns>
kubectl logs <pod> -n <ns>
```

---

## Pod Not Starting (Pending)

The pod is created but not scheduled to a node.

```bash
kubectl describe pod <pod> -n <ns>   # look at Events section
```

**Common causes and fixes:**

| Event message | Cause | Fix |
|---|---|---|
| `Insufficient cpu/memory` | Node resources exhausted | Scale cluster, reduce requests, or delete unused pods |
| `0/N nodes are available: N node(s) had taint` | Taint/toleration mismatch | Add matching toleration or remove taint |
| `no persistent volumes available` | PVC cannot bind | Check PV availability, storage class, access modes |
| `didn't match Pod's node affinity/selector` | Scheduling constraints | Fix nodeSelector/affinity or label nodes |

```bash
# Check node capacity vs allocated
kubectl describe node <node> | grep -A 5 "Allocated resources"
kubectl top node

# Check PVC status
kubectl get pvc -n <ns>
kubectl describe pvc <pvc> -n <ns>
```

---

## CrashLoopBackOff

Container starts, crashes, restarts with exponential backoff.

```bash
# Get logs from the crashed container
kubectl logs <pod> -n <ns> --previous
kubectl logs <pod> -c <container> -n <ns> --previous

# Check exit code
kubectl get pod <pod> -n <ns> -o jsonpath='{.status.containerStatuses[0].lastState.terminated}'
```

**Common causes:**

- **Exit code 1** -- Application error. Read logs for stack trace.
- **Exit code 137 (SIGKILL)** -- OOMKilled. Increase memory limits.
- **Exit code 1 with no logs** -- Bad command/entrypoint. Check `command` and `args` in spec.
- **Failing liveness probe** -- Container restarts when probe fails.

```bash
# Check if OOMKilled
kubectl get pod <pod> -n <ns> -o jsonpath='{.status.containerStatuses[0].lastState.terminated.reason}'

# Debug by overriding the entrypoint
kubectl run debug --image=<image> --restart=Never -it --rm -- sh
```

Fix OOMKilled:

```yaml
resources:
  limits:
    memory: 512Mi   # increase from previous value
  requests:
    memory: 256Mi
```

---

## ImagePullBackOff

Kubernetes cannot pull the container image.

```bash
kubectl describe pod <pod> -n <ns>   # look for "Failed to pull image"
```

**Common causes:**

- **Wrong image name/tag** -- Typo or tag does not exist
- **Private registry without credentials** -- Missing imagePullSecrets
- **Rate limiting** -- Docker Hub anonymous pull limits

```bash
# Create registry secret
kubectl create secret docker-registry regcred \
  --docker-server=ghcr.io \
  --docker-username=<user> \
  --docker-password=<token> \
  -n <ns>
```

Reference it in the pod spec:

```yaml
spec:
  imagePullSecrets:
    - name: regcred
```

---

## Service Not Reachable

Traffic is not reaching the pods behind a Service.

```bash
# 1. Verify service exists and has endpoints
kubectl get svc <svc> -n <ns>
kubectl get endpoints <svc> -n <ns>

# 2. Empty endpoints = selector mismatch
kubectl get svc <svc> -n <ns> -o jsonpath='{.spec.selector}'
kubectl get pods -n <ns> --show-labels

# 3. Test connectivity from inside the cluster
kubectl run curl --image=curlimages/curl --restart=Never --rm -it -- \
  curl -v http://<svc>.<ns>.svc.cluster.local:<port>
```

**Checklist:**

- Service selector labels match pod labels exactly
- `targetPort` matches the container's actual listening port
- Pod readiness probe is passing (unready pods are removed from endpoints)
- NetworkPolicy is not blocking traffic

```bash
# Check readiness
kubectl get pods -n <ns> -o wide   # READY column shows x/y

# Check network policies
kubectl get networkpolicy -n <ns>
kubectl describe networkpolicy <name> -n <ns>
```

---

## DNS Issues

Pods cannot resolve service names or external domains.

```bash
# Test DNS from inside a pod
kubectl run dnstest --image=busybox:1.36 --restart=Never --rm -it -- \
  nslookup <svc>.<ns>.svc.cluster.local

# Check CoreDNS is running
kubectl get pods -n kube-system -l k8s-app=kube-dns
kubectl logs -n kube-system -l k8s-app=kube-dns --tail=50

# Check resolv.conf inside a pod
kubectl exec <pod> -n <ns> -- cat /etc/resolv.conf
```

**Common fixes:**

- CoreDNS pods crashed -- check logs and resource limits
- `ndots:5` causing slow external lookups -- use FQDN with trailing dot (`api.example.com.`)
- Pod DNS policy set to `None` or `Default` -- should usually be `ClusterFirst`

---

## Resource Limits and OOM

```bash
# Check resource usage vs limits
kubectl top pod -n <ns> --containers
kubectl top node

# Find OOMKilled pods
kubectl get pods -n <ns> -o json | \
  jq '.items[] | select(.status.containerStatuses[]?.lastState.terminated.reason=="OOMKilled") | .metadata.name'

# Check resource quotas
kubectl get resourcequota -n <ns>
kubectl describe resourcequota -n <ns>

# Check limit ranges
kubectl get limitrange -n <ns>
kubectl describe limitrange -n <ns>
```

**Guidelines for setting resources:**

- `requests` = typical usage (used for scheduling)
- `limits` = maximum allowed (OOMKill if exceeded for memory, throttled for CPU)
- Start with generous limits, monitor with `kubectl top`, then tighten
- CPU limits are optional (throttling is less harmful than OOMKill)

---

## Node Pressure and Eviction

```bash
# Check node conditions
kubectl describe node <node> | grep -A 5 Conditions

# Common pressure conditions:
#   MemoryPressure    -- node running low on memory
#   DiskPressure      -- node running low on disk
#   PIDPressure       -- too many processes

# Check what is using resources
kubectl top pod --all-namespaces --sort-by=memory | head -20
kubectl top pod --all-namespaces --sort-by=cpu | head -20

# Check for evicted pods
kubectl get pods -n <ns> --field-selector=status.phase=Failed | grep Evicted
kubectl delete pods -n <ns> --field-selector=status.phase=Failed  # clean up
```

---

## Useful Debug Patterns

### Ephemeral debug container (K8s 1.23+)

```bash
kubectl debug -it <pod> -n <ns> --image=busybox:1.36 --target=<container>
```

### Copy a running pod for debugging

```bash
kubectl debug <pod> -n <ns> --copy-to=debug-pod --container=app -- sh
```

### Check RBAC permissions

```bash
kubectl auth can-i create deployments -n <ns>
kubectl auth can-i '*' '*' --all-namespaces    # am I cluster-admin?
kubectl auth can-i get pods --as=system:serviceaccount:<ns>:<sa>
```

### Dump all pod info for offline analysis

```bash
kubectl get pods -n <ns> -o yaml > pods-dump.yaml
kubectl get events -n <ns> --sort-by=.lastTimestamp > events.txt
```
