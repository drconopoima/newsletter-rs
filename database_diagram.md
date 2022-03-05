# Database Diagram

```text
┌─────────────────────────────────┐
│  newsletter.subscription        │
├─────────────────────────────────┤
│  id: uuid (PK)                  │
│  email: citext                  │
│  name: text                     │
│  subscription_date: timestamptz │
└─────────────────────────────────┘
```

```text
┌─────────────────────────────────┐
│  _initialization_migrations     │
├─────────────────────────────────┤
│  version: serial (PK)           │
│  filename: text                 │
│  installed_on: timestamptz      │
|  md5_hash: uuid                 |
└─────────────────────────────────┘
```

```text
┌─────────────────────────────┐
│  _healthcheck               │
├─────────────────────────────┤
|  id: bool (PK, check)       |
│  datetime: timestamptz      │
└─────────────────────────────┘
```
