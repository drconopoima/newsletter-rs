# Database Diagram

```text
┌─────────────────────────────────┐
│  newsletter.subscription        │
├─────────────────────────────────┤
│ id: uuid (PK)                   │
│ email: citext                   │
│ name: VARCHAR(254)              │
│ subscription_date: timestamptz  │
└─────────────────────────────────┘
```

```text
┌─────────────────────────────────┐
│  _initialization_migrations     │
├─────────────────────────────────┤
│ version: serial (PK)            │
│ filename: text                  │
│ installed_on: timestamptz       │
| md5_hash: uuid                  |
└─────────────────────────────────┘
```

```text
┌────────────────────────────────┐
│  _healthcheck                  │
├────────────────────────────────┤
| id: bool (PK)                  |
│ datetime: timestamptz          │
| updated_by: char varying(126)  |
└────────────────────────────────┘
```
