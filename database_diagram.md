# Database Diagram

```text
┌─────────────────────────────────┐
│  newsletter.subscription        │
├─────────────────────────────────┤
│ id: uuid (PK)                   │
│ email: citext                   │
│ first_name:  char varying(126)  │
│ last_name:  char varying(126)   │
│ subscription_date: timestamptz  │
└─────────────────────────────────┘
```

```text
┌─────────────────────────────────┐
│  _initialization_migrations     │
├─────────────────────────────────┤
│ version: serial (PK)            │
│ filename: char varying(255)     │
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
