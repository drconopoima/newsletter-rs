# Database Diagram

┌─────────────────────────────────┐
│  newsletter.subscription        │
├─────────────────────────────────┤
│  id: uuid (PK)                  │
│  email: citext                  │
│  name: text                     │
│  subscription_date: timestamptz │
└─────────────────────────────────┘

┌─────────────────────────────────┐
│  _initialization_migrations     │
├─────────────────────────────────┤
│  version: serial (PK)           │
│  filename: text                 │
│  installed_on: timestamptz      │
|  md5_hash: uuid                 |
└─────────────────────────────────┘
