## Permissions

### Authorizing to endpoints

If you want to perform a protected action, e.g. create a new league, game, or similar, you _must_ attach an Authorization header bearing a session token which corresponds to a user who is allowed to perform that action.

EXAMPLE:

```
HTTP/1.1 POST /api/v1/leagues
Accept: application/json;*/*
Authorization: Bearer (token)

{body}
```

### POST `/admin/users`

Forcibly add a new User.

| Key    |       Type |
| :----- | ---------: |
| (body) | `MiniUser` |

**Response:**

| Key        |   Type |
| :--------- | -----: |
| (response) | `User` |

### POST `/admin/leagues`

Add a new league. Required permission: CreateLeague

**Body:**

| Key             |     Type |
| :-------------- | -------: |
| name            | `string` |
| accepting_teams |   `bool` |

**Response:**

| Key        |     Type |
| :--------- | -------: |
| (response) | `League` |
