# 🦀 RealWorld Rust (Axum + SQLx)

A high-performance, strictly typed implementation of the [RealWorld](https://github.com) backend specification.

## Tech Stack
- **Axum** - Modern, ergonomic web framework
- **SQLx** - Compile-time verified SQL queries
- **PostgreSQL** - Reliable relational database
- **Tokio** - Async runtime for high-performance networking
- **Bcrypt/Argon2** - Secure password hashing
- **JWT** - JSON Web Token implementation
- **Envy** - Loading and validating configuration from environment variables
- **Testcontainers** - Isolated database testing

## Prerequisites
- **Rust 1.75+** ([install](https://rust-lang.org))
- **Docker** (for running PostgreSQL and integration tests)

## Getting Started

### 1. Configure Environment
```bash
cp .env.example .env
```
Edit .env to match your setup:
DATABASE_URL=postgres://postgres:password@localhost:5432/conduit
JWT_SECRET=your-secret-key
SERVER_PORT=8080
### 2. Run with Docker (Recommended)

The easiest way to start the server and database:

```bash
docker-compose up --build
```

The server will start at http://localhost:8080.

