# API V1 Documentation

**All endpoints are prefixed with /api/v1/**

## Permissions

| Name           |  Value   |                                          Comment |
| :------------- | :------: | -----------------------------------------------: |
| ADMIN          |   `1`    |                                  All permissions |
| SetPermissions | `1 << 1` |                   Can set permissions for others |
| CreateLeague   | `1 << 2` | Can create new leagues, and modify existing ones |
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

### POST `/leagues`

Add a new league. Required permission: CreateLeague

**Body:**

| Key  |     Type |
| :--- | -------: |
| name | `string` |

## Users

### Type `User`

| Key         |          Type |
| :---------- | ------------: |
| id          |         `int` |
| permissions | `permissions` |
| avatarurl   |      `string` |
| steamid     |      `string` |
| username    |      `string` |

### GET `/user/steamid/{steamid}`

where {steamid} is a valid steamid.

### GET `/user/authtoken/{authtoken}`

where {authtoken} is a valid held authorization token.

**Response:**

404s if user is not found.

| Key        | Type |
| :--------- | ---: |
| (response) | User |
