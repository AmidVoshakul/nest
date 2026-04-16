---
name: golang-backend-patterns
description: "Master Go backend development with Gin, Echo, Fiber, and Chi. Production patterns for REST APIs, middleware, authentication, and deployment."
---

## Use this skill when

- Building REST APIs in Go
- Choosing between Gin/Echo/Fiber/Chi
- Implementing Go authentication, middleware, or routing
- Optimizing Go performance

## Do not use this skill when

- Need microservices (use backend-architect)
- Need database ORM (use prisma-expert/postgresql)
- Need gRPC (use separate skill)

## Instructions

1. Identify framework need (API complexity, performance)
2. Choose appropriate framework
3. Implement with quality gates
4. Verify with tests

---

# Go Backend Patterns

## Framework Decision

| Framework | Best For | Performance | Ecosystem |
|-----------|---------|------------|-----------|
| **Gin** | Most Go APIs | High | Best |
| **Echo** | Elegant APIs | High | Good |
| **Fiber** | High-throughput | Highest | Good |
| **Chi** | stdlib-style | High | Minimal |

**Default choice:** Gin (best ecosystem, most examples)

## Quality Gates

```bash
# 1. Lint
go vet ./...
golint -set_exit_status=1 ./...  # or staticcheck

# 2. Security
gosec -fmt=json -out=gosec.json ./...

# 3. Test
go test -v -race -coverprofile=coverage.out ./...

# 4. Build
go build -o bin/server ./cmd/server

# Full gate: go vet && gosec && go test && go build
```

## Project Structure

```
myapi/
├── cmd/
│   └── server/main.go
├── internal/
│   ├── handler/
│   ├── middleware/
│   ├── model/
│   └── repository/
├── pkg/
│   └── response/
├── go.mod
└── go.sum
```

## Handlers

```go
// Gin example
func GetUser(c *gin.Context) {
    id := c.Param("id")
    user, err := repo.GetUser(c.Request.Context(), id)
    if err != nil {
        c.JSON(404, response.Error("user not found"))
        return
    }
    c.JSON(200, response.Success(user))
}

// Echo example
func GetUser(c echo.Context) error {
    id := c.Param("id")
    user, err := repo.GetUser(c.Request().Context(), id)
    if err != nil {
        return c.JSON(404, response.Error("user not found"))
    }
    return c.JSON(200, response.Success(user))
}
```

## Middleware

```go
func Logger() gin.HandlerFunc {
    return func(c *gin.Context) {
        start := time.Now()
        c.Next()
        log.Printf("%s %s %d %v", c.Method(), c.Path(), c.Writer.Status(), time.Since(start))
    }
}

func Auth() gin.HandlerFunc {
    return func(c *gin.Context) {
        token := c.GetHeader("Authorization")
        if token == "" {
            c.AbortWithStatus(401)
            return
        }
        c.Next()
    }
}
```

## Authentication

```go
func CreateToken(userID string) (string, error) {
    claims := jwt.MapClaims{
        "user_id": userID,
        "exp":     time.Now().Add(time.Hour).Unix(),
    }
    token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
    return token.SignedString([]byte(os.Getenv("JWT_SECRET")))
}

func ValidateToken(tokenStr string) (string, error) {
    token, err := jwt.Parse(tokenStr, func(token *jwt.Token) (interface{}, error) {
        return []byte(os.Getenv("JWT_SECRET")), nil
    })
    if err != nil || !token.Valid {
        return "", errors.New("invalid token")
    }
    claims := token.Claims.(jwt.MapClaims)
    return claims["user_id"].(string), nil
}
```

## Error Response

```go
type ErrorResponse struct {
    Error   string `json:"error"`
    Code    int    `json:"-"`
}

type SuccessResponse struct {
    Data    interface{} `json:"data"`
}

// Usage
c.JSON(200, SuccessResponse{Data: user})
c.JSON(404, ErrorResponse{Error: "not found", Code: 404})
```

## Request Validation

```go
type CreateUserRequest struct {
    Email    string `json:"email" validate:"required,email"`
    Password string `json:"password" validate:"required,min=8"`
    Name     string `json:"name" validate:"required"`
}

func CreateUser(c *gin.Context) {
    var req CreateUserRequest
    if err := c.ShouldBindJSON(&req); err != nil {
        c.JSON(400, response.Error(err.Error()))
        return
    }
    // validate manually or use go-playground/validator
}
```

## Configuration

```go
type Config struct {
    Server   ServerConfig
    Database DatabaseConfig
    JWT      JWTConfig
}

type ServerConfig struct {
    Port string `env:"SERVER_PORT" default:"8080"`
}

func LoadConfig() (*Config, error) {
    err := envconfig.Process("myapi", &cfg)
    return &cfg, err
}
```

## Database

```go
// With sqlx
var user User
err := db.GetContext(ctx, &user, "SELECT * FROM users WHERE id = $1", id)

// With GORM
result := db.First(&user, id)
```

## Testing

```go
func TestGetUser(t *testing.T) {
    w := httptest.NewRecorder()
    c, _ := gin.CreateTestContext(w)
    c.Params = gin.Params{{Key: "id", Value: "1"}}
    
    GetUser(c)
    
    assert.Equal(t, 200, w.Code)
}
```

## Deployment

```bash
# Build
CGO_ENABLED=0 GOOS=linux go build -a -ldflags '-s -w' -o bin/server

# Docker
FROM golang:1.22-alpine AS builder
WORKDIR /app
COPY . .
RUN CGO_ENABLED=0 go build -o server

FROM alpine:latest
COPY --from=builder /app/server /server
EXPOSE 8080
ENTRYPOINT ["/server"]
```

## Common Commands

| Command | Purpose |
|---------|---------|
| `go mod init` | Initialize module |
| `go mod tidy` | Clean dependencies |
| `go vet ./...` | Check errors |
| `go test ./...` | Run tests |
| `go run cmd/server` | Run server |
| `go build -o bin/server` | Binary build |

## Response

1. **Analyze requirements** - API type, performance needs
2. **Choose framework** - Gin default, Fiber for high-throughput
3. **Implement handlers** - Follow structure above
4. **Add middleware** - Logging, auth as needed
5. **Test and verify** - Quality gates pass