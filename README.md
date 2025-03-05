# OxideGate - Cloud Native Proxy

## Overview
OxideGate is a lightweight, high-performance proxy server. It is built using Rust and
leverages modern asynchronous programming techniques to provide efficient and scalable proxying capabilities.

## Features
- High performance and low latency with hyper
- Asynchronous I/O using Tokio
- HTTP/2 and HTTP/1.1 support
- Load balancing and failover
- Easy configuration with TOML files

## Docker Image

```bash
docker push alexspx/oxidegate:latest
```

## Configuration File

The config file defines the settings for the proxy service, including the server configuration, 
frontends, and backends. The config file path must be set by an enviornment variable `CONFIG_FILE`.
Below is an overview of its structure and expected values.

### Configuration Sections

#### `server` (Global Server Settings)

| Key            | Type    | Default | Description |
|---------------|--------|---------|-------------|
| `enable_https` | `bool`  | `false` | Enables HTTPS mode. Requires `cert_path` and `key_path` if `true`. |
| `port`        | `u16`   | `3000`  | The port on which the server listens. |
| `cert_path`   | `string` | `None`  | Path to the TLS certificate file (required if `enable_https: true`). |
| `key_path`    | `string` | `None`  | Path to the TLS key file (required if `enable_https: true`). |

#### `frontends` (Routing Rules)
Defines how incoming requests are mapped to backend services.

| Key            | Type     | Description |
|---------------|---------|-------------|
| `path_prefixes` | `Vec<String>` | List of path prefixes that should be routed to a specific backend. |
| `backend`     | `string` | The name of the backend to route the requests to. |

#### `backends` (Load Balancing Configuration)
Defines backend services and their load balancing strategies.

| Key            | Type             | Description |
|---------------|----------------|-------------|
| `name`        | `string`         | Identifier for the backend. |
| `lb_algorithm` | `string` (optional) | Load balancing algorithm (`RoundRobin`, `LeastConnections`, `WeightedRoundRobin`). Defaults to `RoundRobin`. |
| `servers`     | `Vec<BackendServer>` | List of backend servers. |

##### `servers` (Backend Server Instances)
Each backend can have multiple server instances with optional weighting.

| Key     | Type     | Description |
|---------|---------|-------------|
| `server` | `string` | The backend server URL (e.g., `http://host:port`). |
| `weight` | `u32` (optional) | Weight for weighted load balancing. |

---

### Notes
- **HTTPS Mode:** If `enable_https` is set to `true`, `cert_path` and `key_path` must be provided.
- **Load Balancing Algorithms:**
  - `RoundRobin`: Requests are distributed evenly in a cyclic manner.
  - `LeastConnections`: Requests are sent to the backend with the fewest active connections.
  - `WeightedRoundRobin`: Requests are distributed based on server weight.
- **Path Prefix Matching:** Requests matching any `path_prefix` in `frontends` are forwarded to the corresponding `backend`.

---



### Example `config.yml`
```yml
server:
  enable_https: true
  port: 3000
  key_path: "certs/key.pem"
  cert_path: "certs/cert.pem"

frontends:
  - path_prefixes: 
      - "/hello_world"
      - "/hello_world/*"
      - "/cant"
    backend: "hello-world-backend"
  - path_prefixes:
      - "/test"
    backend: "test-backend"

backends:
  - name: "test-backend"
    servers:
      - server: "http://localhost:3003"
  - name: "hello-world-backend"
    servers:
      - server: "http://localhost:3001"
        weight: 2
      - server: "http://localhost:3002"
        weight: 1
    lb_algorithm: WeightedRoundRobin

```

## Contributing
Contributions are welcome! Please open an issue or submit a pull request on GitHub.