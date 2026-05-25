# Inamute

Indonesian commute schedule API for LRT Jabodebek data. Inamute serves a small homepage plus JSON API endpoints for stations, routes, schedules, and service health.

## Features

- Actix Web HTTP server
- PostgreSQL storage with sqlx migrations
- Seed data for LRT Jabodebek stations, routes, and schedules
- Simple API response envelope with metadata
- Database-backed API key lookup and per-minute rate limiting
- Astro account app with Better Auth Google login and API key management
- Browser API documentation with Swagger UI and OpenAPI
- Docker and Kubernetes deployment files

## Requirements

- Rust 2021 toolchain
- PostgreSQL
- Docker, optional
- Kubernetes with CloudNativePG, optional

## Configuration

The service reads configuration from environment variables, with `.env` support through `dotenvy`.

| Variable | Default | Description |
| --- | --- | --- |
| `DATABASE_URL` | `postgresql://inamute:inamute_password@localhost:5432/inamute` | PostgreSQL connection string |
| `BIND_ADDR` | `0.0.0.0:8080` | Address and port for the HTTP server |
| `RUST_LOG` | `info` | Rust logging filter |
| `BETTER_AUTH_SECRET` | none | Secret for the Astro Better Auth account app |
| `API_KEY_HASH_SECRET` | none | HMAC secret shared by Astro and Actix for hashing API keys |
| `BETTER_AUTH_URL` | `http://localhost:4321` | Public base URL for the Astro account app |
| `GOOGLE_CLIENT_ID` | none | Google OAuth client ID for Better Auth |
| `GOOGLE_CLIENT_SECRET` | none | Google OAuth client secret for Better Auth |

Example `.env`:

```env
DATABASE_URL=postgresql://inamute:inamute_password@localhost:5432/inamute
BIND_ADDR=127.0.0.1:8080
RUST_LOG=info
BETTER_AUTH_SECRET=replace-with-a-32-byte-random-secret
API_KEY_HASH_SECRET=replace-with-a-different-32-byte-random-secret
BETTER_AUTH_URL=http://localhost:4321
GOOGLE_CLIENT_ID=replace-with-google-client-id
GOOGLE_CLIENT_SECRET=replace-with-google-client-secret
```

## Local Development

Start PostgreSQL:

```sh
docker run --name inamute-postgres \
  -e POSTGRES_USER=inamute \
  -e POSTGRES_PASSWORD=inamute_password \
  -e POSTGRES_DB=inamute \
  -p 5432:5432 \
  -d postgres:16
```

Run the API:

```sh
cargo run
```

The application runs migrations automatically on startup. Open `http://localhost:8080` for the homepage, `http://localhost:8080/docs` for browser API documentation, or `http://localhost:8080/api` for the JSON health endpoint.

Run the account app in a second terminal:

```sh
cd account
pnpm install
pnpm dev
```

Open `http://localhost:4321/account` to sign in and manage API keys. In Google Cloud Console, configure the OAuth redirect URI as `http://localhost:4321/api/auth/callback/google` for local development and `https://your-account-domain/api/auth/callback/google` for production.

The account app root at `http://localhost:4321/` is public. Only `/account` requires login because that page creates and tracks API keys.

For the production Google client shown in Google Cloud Console with redirect URI `https://inamute.mris.dev/docs/api/auth/callback/google`, set `BETTER_AUTH_URL=https://inamute.mris.dev/docs`. Do not commit Google client secrets; put local values in `account/.env`, which is ignored by git.

## API

Interactive browser documentation is available at `http://localhost:8080/docs`. The OpenAPI document is served at `http://localhost:8080/api-docs/openapi.json`.

The docs can run API requests directly in the browser. Click `Authorize`, paste an API key into `ApiKeyAuth`, and Swagger UI will send it as the `X-API-Key` header for try-it-out requests. Leaving authorization empty still works for public requests, subject to the unauthenticated rate limit.

All API responses use this shape:

```json
{
  "success": true,
  "data": {},
  "message": null,
  "meta": null
}
```

### `GET /api`

Returns service health and database connectivity.

```sh
curl http://localhost:8080/api
```

### `GET /api/stations`

Returns all stations.

```sh
curl http://localhost:8080/api/stations
```

### `GET /api/routes`

Returns all routes.

```sh
curl http://localhost:8080/api/routes
```

### `GET /api/schedules`

Returns schedules. Optional query parameters:

- `station`: station code, for example `DA` or `JM`
- `type`: `weekday`, `weekend`, or `holiday`; defaults to `weekday`
- `direction`: destination direction, for example `Dukuh Atas` or `Jati Mulya`

```sh
curl "http://localhost:8080/api/schedules?station=JM&type=weekday&direction=Dukuh%20Atas"
```

## Rate Limiting

Requests are rate limited per peer address when unauthenticated. Unauthenticated requests default to 10 requests per minute.

You can pass an API key with the `X-API-Key` header:

```sh
curl -H "X-API-Key: inamute_live_your_generated_key" \
  http://localhost:8080/api/stations
```

API keys are created from the Astro account app and stored in Postgres as HMAC-SHA256 digests using `API_KEY_HASH_SECRET`. Invalid, inactive, expired, or malformed keys return `401 Unauthorized`.

## Docker

Build the multi-stage image:

```sh
docker build -t inamute .
```

Run it:

```sh
docker run --rm \
  -e DATABASE_URL=postgresql://inamute:inamute_password@host.docker.internal:5432/inamute \
  -p 8080:8080 \
  inamute
```

`Dockerfile.deploy` is provided for deployments that already have a local `target/release/inamute` binary and only need to package the runtime image.

## Kubernetes

Kubernetes manifests live in `k8s/`:

- `namespace.yaml`: application namespace
- `cnpg-cluster.yaml`: CloudNativePG PostgreSQL cluster
- `db-credentials.yaml`: database credentials secret
- `app-secrets.yaml`: runtime secrets for the Actix API and Astro account app
- `deployment.yaml`: API deployment
- `service.yaml`: cluster service
- `ingress.yaml`: Traefik ingress for `inamute.mris.dev`

Apply the manifests with:

```sh
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/db-credentials.yaml
kubectl apply -f k8s/cnpg-cluster.yaml
kubectl apply -f k8s/app-secrets.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/ingress.yaml
```

## Project Structure

```text
.
├── src/main.rs              # Actix Web app and API handlers
├── account/                 # Astro Better Auth account app
├── static/index.html        # Homepage
├── migrations/              # PostgreSQL schema and seed data
├── k8s/                     # Kubernetes deployment manifests
├── Dockerfile               # Multi-stage build image
├── Dockerfile.deploy        # Runtime image for pre-built binaries
├── Cargo.toml
└── LICENSE
```

## License

GPL-3.0. See `LICENSE`.
