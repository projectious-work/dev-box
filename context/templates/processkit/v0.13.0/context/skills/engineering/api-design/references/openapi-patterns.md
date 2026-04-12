# OpenAPI 3.1 Patterns Reference

## Document Structure

```yaml
openapi: "3.1.0"
info:
  title: My API
  version: "1.0.0"
  description: API description with **Markdown** support.
  contact:
    name: API Support
    email: api@example.com
  license:
    name: MIT
servers:
  - url: https://api.example.com/v1
    description: Production
  - url: https://staging-api.example.com/v1
    description: Staging
paths:
  /users:
    $ref: "./paths/users.yaml"
components:
  schemas: {}
  securitySchemes: {}
security:
  - BearerAuth: []
```

## Reusable Components with $ref

### Schema Components

```yaml
components:
  schemas:
    User:
      type: object
      required: [id, email, created_at]
      properties:
        id:
          type: string
          format: uuid
          readOnly: true
        email:
          type: string
          format: email
        name:
          type: string
          maxLength: 255
        created_at:
          type: string
          format: date-time
          readOnly: true

    UserCreate:
      type: object
      required: [email, name]
      properties:
        email:
          type: string
          format: email
        name:
          type: string
          maxLength: 255

    PaginatedResponse:
      type: object
      properties:
        data:
          type: array
          items: {}
        pagination:
          $ref: "#/components/schemas/Pagination"

    Pagination:
      type: object
      properties:
        next_cursor:
          type: ["string", "null"]
        has_more:
          type: boolean

    ErrorResponse:
      type: object
      required: [error]
      properties:
        error:
          type: object
          required: [code, message]
          properties:
            code:
              type: string
            message:
              type: string
            details:
              type: array
              items:
                $ref: "#/components/schemas/ErrorDetail"
            request_id:
              type: string

    ErrorDetail:
      type: object
      properties:
        field:
          type: string
        reason:
          type: string
        message:
          type: string
```

### Referencing Components

```yaml
paths:
  /users:
    get:
      summary: List users
      operationId: listUsers
      parameters:
        - $ref: "#/components/parameters/CursorParam"
        - $ref: "#/components/parameters/LimitParam"
      responses:
        "200":
          description: Paginated list of users.
          content:
            application/json:
              schema:
                allOf:
                  - $ref: "#/components/schemas/PaginatedResponse"
                  - type: object
                    properties:
                      data:
                        type: array
                        items:
                          $ref: "#/components/schemas/User"
        "401":
          $ref: "#/components/responses/Unauthorized"
    post:
      summary: Create user
      operationId: createUser
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/UserCreate"
      responses:
        "201":
          description: User created.
          headers:
            Location:
              schema:
                type: string
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/User"
        "422":
          $ref: "#/components/responses/ValidationError"
```

## Reusable Parameters

```yaml
components:
  parameters:
    CursorParam:
      name: cursor
      in: query
      required: false
      schema:
        type: string
      description: Opaque pagination cursor.

    LimitParam:
      name: limit
      in: query
      required: false
      schema:
        type: integer
        minimum: 1
        maximum: 100
        default: 25
      description: Number of items per page.

    PathId:
      name: id
      in: path
      required: true
      schema:
        type: string
        format: uuid
      description: Resource identifier.
```

## Reusable Responses

```yaml
components:
  responses:
    Unauthorized:
      description: Authentication required.
      content:
        application/json:
          schema:
            $ref: "#/components/schemas/ErrorResponse"
          example:
            error:
              code: AUTHENTICATION_ERROR
              message: "Invalid or missing access token."

    ValidationError:
      description: Input validation failed.
      content:
        application/json:
          schema:
            $ref: "#/components/schemas/ErrorResponse"
          example:
            error:
              code: VALIDATION_ERROR
              message: "One or more fields failed validation."
              details:
                - field: email
                  reason: required
                  message: "Email is required."

    NotFound:
      description: Resource not found.
      content:
        application/json:
          schema:
            $ref: "#/components/schemas/ErrorResponse"
```

## Security Schemes

```yaml
components:
  securitySchemes:
    BearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT
      description: JWT access token in Authorization header.

    ApiKeyAuth:
      type: apiKey
      in: header
      name: X-API-Key
      description: API key for service-to-service calls.

    OAuth2:
      type: oauth2
      flows:
        authorizationCode:
          authorizationUrl: https://auth.example.com/authorize
          tokenUrl: https://auth.example.com/token
          scopes:
            read:users: Read user profiles
            write:users: Create and update users
            admin: Full administrative access
```

### Applying Security

```yaml
# Global default — applies to all endpoints
security:
  - BearerAuth: []

# Per-endpoint override
paths:
  /public/health:
    get:
      security: []  # No auth required

  /admin/users:
    get:
      security:
        - BearerAuth: []
        - OAuth2: [admin]
```

## Request/Response Examples

```yaml
paths:
  /users:
    post:
      requestBody:
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/UserCreate"
            examples:
              basic:
                summary: Basic user creation
                value:
                  email: alice@example.com
                  name: Alice Smith
              minimal:
                summary: Minimal required fields
                value:
                  email: bob@example.com
                  name: Bob
```
