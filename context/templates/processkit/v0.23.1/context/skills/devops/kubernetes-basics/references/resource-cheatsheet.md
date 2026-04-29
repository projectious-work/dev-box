# Kubernetes Resource Cheatsheet

## Pod

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: myapp
  labels:
    app: myapp
spec:
  containers:
    - name: app
      image: nginx:1.27
      ports:
        - containerPort: 80
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
          port: 80
        initialDelaySeconds: 5
        periodSeconds: 10
      readinessProbe:
        httpGet:
          path: /ready
          port: 80
        initialDelaySeconds: 3
        periodSeconds: 5
```

## Deployment

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

## Service

```yaml
# ClusterIP (internal)
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
# NodePort (external via node IP)
apiVersion: v1
kind: Service
metadata:
  name: myapp-nodeport
spec:
  type: NodePort
  selector:
    app: myapp
  ports:
    - port: 80
      targetPort: 8080
      nodePort: 30080
---
# LoadBalancer (cloud external LB)
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

## Ingress

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: myapp
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
spec:
  ingressClassName: nginx
  tls:
    - hosts:
        - myapp.example.com
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

## ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: myapp-config
data:
  APP_ENV: production
  LOG_LEVEL: info
  config.yaml: |
    database:
      host: db.default.svc.cluster.local
      port: 5432
```

## Secret

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: myapp-secret
type: Opaque
stringData:           # plain text, encoded on apply
  DB_PASSWORD: s3cret
  API_KEY: abc123
```

## PersistentVolumeClaim

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: myapp-data
spec:
  accessModes:
    - ReadWriteOnce
  storageClassName: standard
  resources:
    requests:
      storage: 10Gi
```

## Job

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: db-migrate
spec:
  backoffLimit: 3
  activeDeadlineSeconds: 300
  template:
    spec:
      restartPolicy: Never
      containers:
        - name: migrate
          image: myapp:1.0.0
          command: ["./migrate", "--up"]
          envFrom:
            - secretRef:
                name: myapp-secret
```

## CronJob

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

---

## kubectl Command Cheatsheet

### Viewing Resources

```bash
kubectl get pods -n <ns> -o wide              # pods with node/IP info
kubectl get all -n <ns>                        # pods, services, deployments, etc.
kubectl get deploy,svc,ing -n <ns>             # specific resource types
kubectl describe pod <pod> -n <ns>             # detailed status and events
kubectl get pod <pod> -o yaml                  # full spec as YAML
kubectl get events --sort-by=.lastTimestamp    # recent cluster events
```

### Creating and Applying

```bash
kubectl apply -f manifest.yaml                 # declarative create/update
kubectl apply -f ./manifests/ -R               # apply directory recursively
kubectl diff -f manifest.yaml                  # preview changes before apply
kubectl create namespace myns                  # imperative namespace creation
kubectl create secret generic db \
  --from-literal=password=s3cret -n <ns>       # quick secret creation
```

### Debugging

```bash
kubectl logs <pod> -n <ns>                     # container logs
kubectl logs <pod> -c <container> --previous   # previous crashed container
kubectl logs -l app=myapp -n <ns> --tail=100   # logs by label
kubectl exec -it <pod> -n <ns> -- sh           # interactive shell
kubectl port-forward svc/myapp 8080:80 -n <ns> # local port forwarding
kubectl top pod -n <ns>                        # resource usage
kubectl top node                               # node resource usage
```

### Scaling and Rollouts

```bash
kubectl scale deploy myapp --replicas=5 -n <ns>
kubectl rollout status deploy/myapp -n <ns>
kubectl rollout history deploy/myapp -n <ns>
kubectl rollout undo deploy/myapp -n <ns>
kubectl rollout restart deploy/myapp -n <ns>   # restart all pods
```

### Deleting

```bash
kubectl delete -f manifest.yaml                # delete by manifest
kubectl delete pod <pod> -n <ns>               # delete specific pod
kubectl delete pod <pod> --grace-period=0 --force  # force delete stuck pod
kubectl delete ns myns                         # delete namespace and everything in it
```

### Context and Config

```bash
kubectl config get-contexts                    # list contexts
kubectl config use-context <name>              # switch context
kubectl config set-context --current --namespace=<ns>  # set default namespace
kubectl config view --minify                   # show current context config
```
