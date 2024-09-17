# API V1 Documentation

Admin panel documentation: [Here](admin/)

**All endpoints are prefixed with /api/v1/**

Table of contents:

1. [Leagues](#leagues)
2. [Users](#users)

### Type `permissions`

| Name           |  Value   |                                          Comment |
| :------------- | :------: | -----------------------------------------------: |
| ADMIN          |   `1`    |                     All permissions (superadmin) |
| SetPermissions | `1 << 1` |                   Can set permissions for others |
| CreateLeague   | `1 << 2` | Can create new Leagues, and modify existing ones |
| CreateGame     | `1 << 3` |           Can create new Games between two teams |

A user's permissions are represented through an i64 bitfield of the above values.

i.e. permission of `0` is nothing, permission of `1` is admin, permission of (binary) `110 (6)` is SetPermissions and CreateLeague.

## Leagues

### Type `League`

| Key  |     Type |
| :--- | -------: |
| id   |    `int` |
| name | `string` |

### GET `/leagues`

Get all current registered leagues. Array can be empty.

**Response:**

| Key        |            Type |
| :--------- | --------------: |
| (response) | `array[League]` |

## Users

### Type `User`

| Key         |          Type |
| :---------- | ------------: |
| id          |         `int` |
| permissions | `permissions` |
| avatarurl   |      `string` |
| steamid     |      `string` |
| username    |      `string` |

A **`MiniUser`** is the same as above, but without an `id` parameter. This is to facilitate easily creating new users.

### GET `/users`

Get all currently registered users.

**Response:**

| Key      |          Type |
| :------- | ------------: |
| response | `array[User]` |

### GET `/user/steamid/{steamid}`

where {steamid} is a valid steamid.

### GET `/user/authtoken/{authtoken}`

where {authtoken} is a valid held authorization token.

**Response:**

404s if user is not found.

| Key        | Type |
| :--------- | ---: |
| (response) | User |
