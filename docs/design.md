# Database

## Configuration

| Name  | Type | Comment |
| ----- | ---- | ------- |
| id    | TEXT | Primary |
| key   | TEXT | index   |
| value | TEXT |         |
| order | int  |         |

AvaliableKey:

- ula_prefix
- host_domain

## Proxy

| Name          | Type      | Comment |
| ------------- | --------- | ------- |
| id            | int       | Primary |
| name          | TEXT      | unique  |
| protocol      | TEXT      |         |
| nameserver    | jsonb     |         |
| configuration | jsonb     |         |
| set           | TEXT      | index   |
| create_at     | timestamp |         |
| update_at     | timestamp |         |

## Rule

| Field     | Type      | Comment |
| --------- | --------- | ------- |
| id        | int       |         |
| mode      | TEXT      | index   |
| value     | TEXT      |         |
| set       | TEXT      | index   |
| create_at | timestamp |         |
| update_at | timestamp |         |

## Match

| Name      | Type      | Comment |
| --------- | --------- | ------- |
| id        | int       |         |
| proxy_set | TEXT      | index   |
| rule_set  | TEXT      | index   |
| create_at | timestamp |         |
| update_at | timestamp |         |

## DNS Record

| Name      | Type      | Comment |
| --------- | --------- | ------- |
| id        | int       |         |
| domain    | TEXT      | index   |
| type      | TEXT      | index   |
| value     | TEXT      |         |
| create_at | timestamp |         |
| update_at | timestamp |         |

## Host

| Name      | Type      | Comment |
| --------- | --------- | ------- |
| id        | int       |         |
| mac_addr  | TEXT      | index   |
| name      | TEXT      | index   |
| hostname  |           |         |
| create_at | timestamp |         |
| update_at | timestamp |         |

## Host ip

| Name      | Type      | Comment |
| --------- | --------- | ------- |
| id        | int       |         |
| host_id   | TEXT      | index   |
| ip        | TEXT      | index   |
| create_at | timestamp |         |
| update_at | timestamp |         |
