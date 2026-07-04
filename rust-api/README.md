# Dependencies

- [rust](https://rust-lang.org/) ([cargo](https://doc.rust-lang.org/cargo/))
- [docker](https://www.docker.com/products/docker-desktop/)

# Installation

```shell
cp .env.example .env
docker compose up -d
cargo build
```

# Run

```shell
cargo run
```

# Routes

## Auth — `/auth`

**POST `/auth/`** — Create credentials for a user.
Body (JSON): `{ "user_id": i32, "email": string, "password": string }`
Password is hashed (Argon2id) before storage. Returns `200` empty.

**POST `/auth/login`** — Authenticate and open a session.
Body (JSON): `{ "email": string, "password": string }`
On success, sets a `session-id` cookie (path `/`) and returns `200` with JSON `expires_on` (RFC3339 datetime). Wrong email/password → `401`.

## Sessions — `/sessions`

**POST `/sessions/`** — Create a new session unconditionally.
No body. Sets a `session-id` cookie and returns `200` with JSON `expires_on` (datetime).

**GET `/sessions/`** — Validate the current session.
Requires a `cookie` header containing the session id. Returns `204 No Content` if valid; `401` if expired.

## Users — `/users`

**GET `/users/`** — List users with optional filtering/sorting.
Query params (all optional): `field` (`id|name|login|created_on`), `order`, `id` (i32), `name`, `login`, `created_on`, `limit` (u8).
Note: if `field` is provided, the matching value (e.g. `id` when `field=id`) is required → otherwise `400`. Returns JSON array of `User`.

**POST `/users/`** — Create a user (also creates its auth record).
Body (JSON): `{ "name": string, "login": string, "email": string, "password": string }`
Returns `201 Created` with the created `User`.

**GET `/users/:id_or_login`** — Fetch one user.
Path param is parsed as i32 (lookup by id); if not numeric, lookup by login. Returns JSON `User`.

**PUT `/users/:id`** — Update a user.
Path param: i32 id. Body (JSON): same shape as create (`name`, `login`, `email`, `password`). Returns updated JSON `User`.

**DELETE `/users/:id`** — Delete a user.
Path param: i32 id. Returns the deleted JSON `User`.

---

`User` response shape: `{ "id": i32, "name": string, "login": string, "created_on": datetime }`.

Note: the `Project` and `Task` structs exist in `structs_and_enums.rs` but have no routes wired up yet.
