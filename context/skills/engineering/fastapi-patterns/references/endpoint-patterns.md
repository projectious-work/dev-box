# FastAPI Endpoint Patterns

## CRUD Endpoints

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

## Pydantic Schemas

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

## Cursor-Based Pagination

```python
from pydantic import BaseModel

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

## File Upload

```python
from fastapi import UploadFile, File

@router.post("/upload")
async def upload_file(file: UploadFile = File(...)):
    if file.size > 10 * 1024 * 1024:  # 10 MB limit
        raise HTTPException(413, "File too large")
    contents = await file.read()
    # Process or save file
    return {"filename": file.filename, "size": len(contents)}

@router.post("/upload-multiple")
async def upload_multiple(files: list[UploadFile] = File(...)):
    return [{"name": f.filename, "size": f.size} for f in files]
```

## Streaming Response

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

## WebSocket Pattern

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

## Auth Middleware

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

@router.get("/me", response_model=UserResponse)
def read_profile(user: CurrentUser):
    return user

@app.post("/token")
def login(form: OAuth2PasswordRequestForm = Depends(), db: DbDep):
    user = authenticate(db, form.username, form.password)
    if not user:
        raise HTTPException(400, "Incorrect credentials")
    token = jwt.encode({"sub": user.id}, SECRET_KEY, algorithm="HS256")
    return {"access_token": token, "token_type": "bearer"}
```

## Testing

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
