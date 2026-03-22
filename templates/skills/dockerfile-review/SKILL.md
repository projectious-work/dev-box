---
name: dockerfile-review
description: Dockerfile best practices review — layer optimization, caching, security, image size. Use when writing or reviewing Dockerfiles.
---

# Dockerfile Review

## When to Use

When the user asks to review a Dockerfile, optimize an image, or says "why is my image so big?", "is this Dockerfile correct?", or "help me with Docker".

## Instructions

1. **Layer optimization:**
   - Combine related `RUN` commands with `&&` to reduce layers
   - Order from least to most frequently changing (dependencies before source code)
   - Use `.dockerignore` to exclude unnecessary files
2. **Caching:**
   - Copy dependency manifests first, install, then copy source:
     ```dockerfile
     COPY requirements.txt .
     RUN pip install -r requirements.txt
     COPY . .
     ```
   - Pin dependency versions for reproducible builds
3. **Security:**
   - Don't run as root — add `USER nonroot` after setup
   - Never `COPY` secrets (use build secrets or runtime env vars)
   - Pin base image versions with digest: `FROM python:3.12-slim@sha256:...`
   - Remove package manager caches: `rm -rf /var/lib/apt/lists/*`
4. **Size reduction:**
   - Use slim or alpine base images
   - Multi-stage builds for compiled languages
   - Remove build tools after compilation
   - Use `--no-install-recommends` with apt-get
5. **Correctness:**
   - Use `COPY` over `ADD` (unless extracting archives)
   - Set `WORKDIR` instead of `cd` in `RUN`
   - Use `exec form` for `CMD` and `ENTRYPOINT`: `["executable", "arg"]`

## Examples

**User:** "Review my Dockerfile"
**Agent:** Reads the Dockerfile, identifies that dependency installation and source copy are in the same layer (cache-busting), apt lists aren't cleaned up, and the container runs as root. Provides specific fixes for each issue.
