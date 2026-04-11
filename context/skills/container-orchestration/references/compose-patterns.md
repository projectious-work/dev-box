# Compose Patterns

Common multi-service architecture patterns with complete YAML examples.

## Web + Database

The most basic production pattern. Always include health checks, named volumes for data persistence, and explicit networking.

```yaml
services:
  web:
    build: .
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgres://app:${DB_PASSWORD}@db:5432/myapp
    depends_on:
      db:
        condition: service_healthy
    restart: unless-stopped

  db:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: app
      POSTGRES_PASSWORD: ${DB_PASSWORD}
      POSTGRES_DB: myapp
    volumes:
      - pg-data:/var/lib/postgresql/data
      - ./init.sql:/docker-entrypoint-initdb.d/init.sql:ro
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U app -d myapp"]
      interval: 5s
      timeout: 3s
      retries: 5
    restart: unless-stopped

volumes:
  pg-data:
```

## Web + Database + Cache

Add Redis as a session store or application cache. The web service waits for both dependencies.

```yaml
services:
  web:
    build: .
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgres://app:${DB_PASSWORD}@db:5432/myapp
      REDIS_URL: redis://cache:6379/0
    depends_on:
      db:
        condition: service_healthy
      cache:
        condition: service_healthy
    restart: unless-stopped

  db:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: app
      POSTGRES_PASSWORD: ${DB_PASSWORD}
      POSTGRES_DB: myapp
    volumes:
      - pg-data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U app -d myapp"]
      interval: 5s
      timeout: 3s
      retries: 5
    restart: unless-stopped

  cache:
    image: redis:7-alpine
    volumes:
      - redis-data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5
    command: redis-server --appendonly yes --maxmemory 256mb --maxmemory-policy allkeys-lru
    restart: unless-stopped

volumes:
  pg-data:
  redis-data:
```

## Worker Queue Pattern

Background job processing with a message broker. The web service enqueues jobs, workers process them asynchronously.

```yaml
services:
  web:
    build: .
    ports:
      - "8080:8080"
    environment:
      BROKER_URL: amqp://guest:guest@rabbitmq:5672/
      DATABASE_URL: postgres://app:${DB_PASSWORD}@db:5432/myapp
    depends_on:
      rabbitmq:
        condition: service_healthy
      db:
        condition: service_healthy
    restart: unless-stopped

  worker:
    build: .
    command: celery -A app worker --loglevel=info --concurrency=4
    environment:
      BROKER_URL: amqp://guest:guest@rabbitmq:5672/
      DATABASE_URL: postgres://app:${DB_PASSWORD}@db:5432/myapp
    depends_on:
      rabbitmq:
        condition: service_healthy
      db:
        condition: service_healthy
    deploy:
      replicas: 2
    restart: unless-stopped

  rabbitmq:
    image: rabbitmq:3-management-alpine
    ports:
      - "15672:15672"    # management UI (debug profile recommended)
    volumes:
      - rabbitmq-data:/var/lib/rabbitmq
    healthcheck:
      test: ["CMD", "rabbitmq-diagnostics", "check_port_connectivity"]
      interval: 10s
      timeout: 5s
      retries: 5
      start_period: 15s
    restart: unless-stopped

  db:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: app
      POSTGRES_PASSWORD: ${DB_PASSWORD}
      POSTGRES_DB: myapp
    volumes:
      - pg-data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U app -d myapp"]
      interval: 5s
      timeout: 3s
      retries: 5
    restart: unless-stopped

volumes:
  pg-data:
  rabbitmq-data:
```

## Reverse Proxy Pattern

Nginx or Traefik in front of application services. Useful for TLS termination, load balancing, and serving static assets.

```yaml
services:
  proxy:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./certs:/etc/nginx/certs:ro
    depends_on:
      app:
        condition: service_healthy
    networks:
      - frontend
    restart: unless-stopped

  app:
    build: .
    expose:
      - "8080"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 10s
      timeout: 3s
      retries: 3
    networks:
      - frontend
      - backend
    restart: unless-stopped

  db:
    image: postgres:16-alpine
    volumes:
      - pg-data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 3s
      retries: 5
    networks:
      - backend
    restart: unless-stopped

networks:
  frontend:
  backend:

volumes:
  pg-data:
```
