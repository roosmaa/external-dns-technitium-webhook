# Repository Guidelines

## Project Structure

```
src/
  main.rs          — Axum router setup, startup zone-check logic, token auto-renewal
  app.rs           — AppState, AppError, ensure_ready() guard
  config.rs        — Config::from_env() (panics on missing required vars)
  handlers.rs      — HTTP endpoint handlers; ExtDnsJson custom content-type wrapper
  models.rs        — ExternalDNS wire types: Endpoint, Changes, Filters
  technitium.rs    — Module re-export file (not logic)
  technitium/
    client.rs      — TechnitiumClient, TechnitiumError; all API calls use form-encoded POST
    models.rs      — Technitium API request/response types with serde rename annotations
```

No `tests/` directory exists yet; all tests are inline `mod tests` blocks.

## Build, Test, and Development Commands

```sh
cargo fmt -- --check          # format check (not in CI, but expected before PR)
cargo clippy -- -D warnings   # lint; warnings are failures
cargo build --verbose         # what CI runs
cargo test --verbose          # what CI runs
cargo test zone::             # run tests matching "zone::" (useful for focused runs)
cargo run                     # requires env vars below; defaults to port 3000
```

CI (`.github/workflows/build-and-test.yml`) only runs `cargo build` and `cargo test`. It does **not** run `cargo fmt` or `cargo clippy`, but PRs should still pass both.

## Required Environment Variables

`Config::from_env()` panics at startup if required vars are absent — there is no graceful fallback.

| Variable              | Required | Notes                                                        |
|-----------------------|----------|--------------------------------------------------------------|
| `TECHNITIUM_URL`      | yes      | Base URL of Technitium DNS server                            |
| `TECHNITIUM_USERNAME` | yes      |                                                              |
| `TECHNITIUM_PASSWORD` | one of   | Used for login + auto-renew every 20 min                     |
| `TECHNITIUM_TOKEN`    | one of   | Static token; skips login entirely                           |
| `ZONE`                | yes      | Zone to manage (e.g. `example.com`)                          |
| `LISTEN_ADDRESS`      | no       | Defaults to `0.0.0.0`                                        |
| `LISTEN_PORT`         | no       | Defaults to `3000`                                           |
| `DOMAIN_FILTERS`      | no       | Semicolon-separated (e.g. `foo.example.com;bar.example.com`) |
| `RUST_LOG`            | no       | e.g. `external_dns_technitium_webhook=debug,tower_http=debug`|

Exactly one of `TECHNITIUM_PASSWORD` or `TECHNITIUM_TOKEN` must be set; both missing causes a panic.

## Key Quirks

- **`mockito` is in `[dependencies]`** (not `[dev-dependencies]`). This is intentional — tests live inline rather than in a separate test binary.
- **`/health` returns 503** until the async zone-setup task finishes setting `is_ready = true`. All handlers call `ensure_ready()`.
- **Token auto-renewal**: when using password auth, a background task re-authenticates every 20 min (or retries every 60 s on failure). `TechnitiumClient` is stored behind a `RwLock` and replaced atomically.
- **All Technitium API calls are form-encoded POSTs** (`application/x-www-form-urlencoded`). No JSON is sent to Technitium; only received from ExternalDNS.
- **Zone auto-creation**: if `ZONE` doesn't exist at startup, it is created as a `Forwarder` zone pointing to `this-server` with DNSSEC validation. The "zone already exists" API error is swallowed silently.
- **`src/technitium.rs`** is a thin re-export shim; logic lives in `src/technitium/client.rs` and `src/technitium/models.rs`.
- **Serde field renames**: Technitium API fields use camelCase; Rust structs use snake_case. Always check existing rename annotations before adding new model fields.

## Testing Guidelines

- Use `#[tokio::test]` for async tests; use `mockito::Server::new_async().await` to create per-test mock servers.
- Tests assert on HTTP status codes, JSON/form payloads, and `mock.assert()` call counts — not on log output.
- Cover both success and error paths: `TechnitiumError::ApiError` (e.g. `"zone already exists"`), `InvalidToken`, and HTTP non-2xx responses.
- No integration test directory exists yet; add one under `tests/` for full HTTP lifecycle tests.

## Commit & PR Guidelines

- Use Conventional Commits: `feat:`, `fix:`, `docs:`, `refactor:`, etc.
- Every PR must: describe the change, list any new env vars, and paste one-line summaries from `cargo fmt`, `cargo clippy`, and `cargo test`.
- Docker image is published to `ghcr.io/james-gonzalez/external-dns-technitium-webhook:latest` on every push to `main`.

## Security

- Config is env-var only; never hardcode secrets or commit `.env` files.
- Document any new env var in `README.md` (the env var table there is the canonical reference).
- Prefer `TECHNITIUM_TOKEN` over `TECHNITIUM_PASSWORD` in test environments.
