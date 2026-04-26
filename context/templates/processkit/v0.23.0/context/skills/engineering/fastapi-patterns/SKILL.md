---
name: fastapi-patterns
description: |
  FastAPI patterns — dependency injection, Pydantic models, async endpoints, middleware, and testing. Use when building FastAPI applications, designing API endpoints, adding authentication, or reviewing FastAPI code.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-fastapi-patterns
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# FastAPI Patterns

## Intro

FastAPI leans on type hints, Pydantic models, and `Depends()` to make
endpoints declarative. Define schemas as `BaseModel` subclasses, inject
collaborators with `Depends`, and use `async def` only when the work is
genuinely I/O-bound.

## Overview

### Routes and routers

Use HTTP method decorators (`@app.get`, `@app.post`, `@app.put`,
`@app.delete`, `@app.patch`). Path parameters are typed function
arguments (`user_id: int`); query parameters are arguments with
defaults (`skip: int = 0, limit: int = 100`). Group related endpoints
with `APIRouter(prefix="/users", tags=["users"])` and set explicit
status codes via `status_code=201`.

### Pydantic models

Define request bodies and responses as `BaseModel` subclasses. Keep
`Create`, `Update`, and `Response` schemas distinct so PATCH semantics
and ORM round-trips don't bleed together. Set
`model_config = ConfigDict(from_attributes=True)` on response schemas
to allow construction from ORM instances. Use `response_model=...` on
the route to filter what leaks out, and `Field(min_length=...,
gt=...)` for inline validation.

### Dependency injection

Dependencies are plain functions or callables resolved by `Depends()`.
Yield-based dependencies (`yield session; session.close()`) handle
cleanup. Reuse them with `Annotated`:
`DbDep = Annotated[Session, Depends(get_db)]`. Apply app-wide
dependencies via `FastAPI(dependencies=[Depends(verify_api_key)])`.
Dependencies chain naturally — `get_current_user` depends on
`oauth2_scheme`, which extracts the header.

### Async vs sync handlers

Use `async def` for I/O-bound work (async DB drivers, `httpx`,
`aiofiles`). Use plain `def` for CPU-bound or sync-only libraries —
FastAPI runs them in a thread pool automatically. Never put blocking
calls inside an `async def` handler; they stall the event loop. Use
`asyncio.gather()` to fan out concurrent async calls inside a single
handler.

### Middleware and CORS

Register middleware with `app.add_middleware(...)`. Common ones:
`CORSMiddleware`, `TrustedHostMiddleware`, `GZipMiddleware`. Custom
middleware can be a function decorated with `@app.middleware("http")`
or a `BaseHTTPMiddleware` subclass. Order matters: the last
middleware added runs first (outermost).

### Error handling

Raise `HTTPException(status_code=..., detail=...)` for client errors.
Validation errors return 422 automatically with field-level detail.
Customize via `@app.exception_handler(...)` or by overriding
`RequestValidationError`. Aim for a consistent error envelope:
`{"detail": "...", "code": "..."}`.

### Testing

Use Starlette's `TestClient(app)` for sync tests and
`httpx.AsyncClient` for async ones. Override dependencies with
`app.dependency_overrides[get_db] = mock_db` and clear the overrides
in teardown. WebSocket tests use the `client.websocket_connect("/ws")`
context manager.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Calling blocking I/O from an `async def` handler.** A `requests.get(...)` or synchronous database call inside `async def` stalls the entire event loop and serializes all concurrent requests behind it. Use `async def` only with async-native libraries (`httpx`, async SQLAlchemy drivers, `aiofiles`); use plain `def` for synchronous work and let FastAPI run it in a thread pool.
- **Sharing one schema model for create, update, and response.** A single model leaks server-generated fields (`id`, `created_at`) into create payloads and loses PATCH semantics (all fields required vs. all optional). Define `XxxCreate`, `XxxUpdate`, and `XxxResponse` as separate Pydantic models.
- **Sharing a single database `Session` across requests.** A session shared between requests bleeds state from one request into another and causes race conditions in concurrent usage. Always inject a per-request session via a `Depends(get_db)` yield dependency.
- **Not setting `response_model=` on routes.** Without `response_model`, FastAPI returns the raw object including internal fields (passwords, internal IDs, server-only metadata) that were never meant to be serialized. Always set `response_model=` explicitly.
- **Forgetting to clear `app.dependency_overrides` between tests.** A dependency override set in one test leaks into subsequent tests, causing mysterious failures that depend on test execution order. Clear overrides in teardown.
- **Returning errors with status 200 and an error payload.** Embedding error information in a 200 response breaks client error handling, monitoring, and the framework's automatic error envelope. Raise `HTTPException` for client errors; let FastAPI format the response.
- **Not setting `model_config = ConfigDict(from_attributes=True)` on response schemas.** Without this, Pydantic V2 cannot construct a response model from a SQLAlchemy ORM instance, causing a `ValidationError` at runtime rather than a type error at authoring time.

## Full reference

### CRUD endpoint pattern

```python
from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.orm import Session
from typing import Annotated

router = APIRouter(prefix="/items", tags=["items"])
DbDep = Annotated[Session, Depends(get_db)]

@router.post("/", status_code=status.HTTP_201_CREATED, response_model=ItemResponse)
def create_item(item: ItemCreate, db: DbDep):
    db_item = Item(**item.model_dump())
    db.add(db_item)
    db.commit()
    db.refresh(db_item)
    return db_item

@router.get("/", response_model=list[ItemResponse])
def list_items(db: DbDep, skip: int = 0, limit: int = 20):
    return db.query(Item).offset(skip).limit(limit).all()

@router.get("/{item_id}", response_model=ItemResponse)
def get_item(item_id: int, db: DbDep):
    item = db.get(Item, item_id)
    if not item:
        raise HTTPException(status_code=404, detail="Item not found")
    return item

@router.put("/{item_id}", response_model=ItemResponse)
def update_item(item_id: int, data: ItemUpdate, db: DbDep):
    item = db.get(Item, item_id)
    if not item:
        raise HTTPException(status_code=404, detail="Item not found")
    for key, val in data.model_dump(exclude_unset=True).items():
        setattr(item, key, val)
    db.commit()
    db.refresh(item)
    return item

@router.delete("/{item_id}", status_code=status.HTTP_204_NO_CONTENT)
def delete_item(item_id: int, db: DbDep):
    item = db.get(Item, item_id)
    if not item:
        raise HTTPException(status_code=404, detail="Item not found")
    db.delete(item)
    db.commit()
```

### Schema separation

```python
from pydantic import BaseModel, ConfigDict, Field
from datetime import datetime

class ItemCreate(BaseModel):
    name: str = Field(min_length=1, max_length=200)
    description: str | None = None
    price: float = Field(gt=0)
    tags: list[str] = []

class ItemUpdate(BaseModel):
    name: str | None = None
    description: str | None = None
    price: float | None = Field(default=None, gt=0)

class ItemResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)
    id: int
    name: str
    description: str | None
    price: float
    created_at: datetime
```

### Cursor-based pagination

```python
class PaginatedResponse(BaseModel):
    items: list[ItemResponse]
    next_cursor: str | None
    has_more: bool

@router.get("/", response_model=PaginatedResponse)
def list_items(db: DbDep, cursor: str | None = None, limit: int = 20):
    query = db.query(Item).order_by(Item.id)
    if cursor:
        query = query.filter(Item.id > int(cursor))
    items = query.limit(limit + 1).all()
    has_more = len(items) > limit
    items = items[:limit]
    return PaginatedResponse(
        items=items,
        next_cursor=str(items[-1].id) if has_more else None,
        has_more=has_more,
    )
```

### File upload

```python
from fastapi import UploadFile, File

@router.post("/upload")
async def upload_file(file: UploadFile = File(...)):
    if file.size > 10 * 1024 * 1024:  # 10 MB limit
        raise HTTPException(413, "File too large")
    contents = await file.read()
    return {"filename": file.filename, "size": len(contents)}
```

### JWT authentication

```python
from fastapi import Security
from fastapi.security import OAuth2PasswordBearer, OAuth2PasswordRequestForm
from jose import jwt, JWTError

oauth2_scheme = OAuth2PasswordBearer(tokenUrl="token")

async def get_current_user(token: str = Depends(oauth2_scheme), db: DbDep):
    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=["HS256"])
        user_id: int = payload.get("sub")
    except JWTError:
        raise HTTPException(status_code=401, detail="Invalid token")
    user = db.get(User, user_id)
    if not user:
        raise HTTPException(status_code=401, detail="User not found")
    return user

CurrentUser = Annotated[User, Depends(get_current_user)]

@app.post("/token")
def login(form: OAuth2PasswordRequestForm = Depends(), db: DbDep):
    user = authenticate(db, form.username, form.password)
    if not user:
        raise HTTPException(400, "Incorrect credentials")
    token = jwt.encode({"sub": user.id}, SECRET_KEY, algorithm="HS256")
    return {"access_token": token, "token_type": "bearer"}
```

Always hash passwords with bcrypt; never store plaintext. Use
`Security(get_current_user, scopes=["admin"])` for scope-gated routes.

### WebSockets

```python
from fastapi import WebSocket, WebSocketDisconnect

class ConnectionManager:
    def __init__(self):
        self.connections: list[WebSocket] = []

    async def connect(self, ws: WebSocket):
        await ws.accept()
        self.connections.append(ws)

    def disconnect(self, ws: WebSocket):
        self.connections.remove(ws)

    async def broadcast(self, message: str):
        for conn in self.connections:
            await conn.send_text(message)

manager = ConnectionManager()

@app.websocket("/ws")
async def websocket_endpoint(ws: WebSocket):
    await manager.connect(ws)
    try:
        while True:
            data = await ws.receive_text()
            await manager.broadcast(f"Message: {data}")
    except WebSocketDisconnect:
        manager.disconnect(ws)
```

### Background tasks

Inject `BackgroundTasks` and call
`background_tasks.add_task(send_email, to=email)`. Tasks run after the
response is flushed, so they cannot block the user. They share the
same process as the request, so failures are isolated to the task.
For heavy or long-running work, hand off to Celery or ARQ instead.

### Streaming responses

```python
from fastapi.responses import StreamingResponse
import asyncio

@router.get("/stream")
async def stream_data():
    async def generate():
        for i in range(100):
            yield f"data: {i}\n\n"
            await asyncio.sleep(0.1)
    return StreamingResponse(generate(), media_type="text/event-stream")
```

### Test patterns

```python
from fastapi.testclient import TestClient

client = TestClient(app)
app.dependency_overrides[get_db] = lambda: test_session

def test_create_item():
    resp = client.post("/items/", json={"name": "Widget", "price": 9.99})
    assert resp.status_code == 201
    assert resp.json()["name"] == "Widget"

def test_item_not_found():
    resp = client.get("/items/9999")
    assert resp.status_code == 404

def test_validation_error():
    resp = client.post("/items/", json={"name": "", "price": -1})
    assert resp.status_code == 422
```

Always clear `app.dependency_overrides` between tests so state from
one test cannot bleed into another.

### Anti-patterns

- Calling blocking I/O from `async def` — stalls the event loop
- Mixing request and response schemas in one model — leaks server
  fields like `id` and `created_at` into create payloads
- Catching `Exception` in handlers and returning 500 manually —
  defeats FastAPI's error envelope
- Reading the request body twice — wrap with a custom middleware if
  you must
- Sharing a single `Session` across requests — always inject per
  request via `Depends`

### Further reading

- [FastAPI Documentation](https://fastapi.tiangolo.com/)
- [Pydantic V2 Documentation](https://docs.pydantic.dev/)
