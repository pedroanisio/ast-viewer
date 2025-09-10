# ADR-011: Adopt SQLx Migration Framework for Database Schema Management

## Status
**Accepted** ✅

## Context

The project currently uses a custom, non-standard approach for database schema management that creates significant operational and maintenance risks:

### Current Problems
1. **No Migration Versioning**: SQL files exist but aren't tracked in database state
2. **Schema Drift Risk**: `initialize_schema()` code differs from migration files
3. **No Rollback Capability**: Cannot safely revert schema changes
4. **Manual Execution**: Requires manual CLI commands for schema updates
5. **No Transaction Safety**: Inconsistent error handling across migrations
6. **Production Deployment Risk**: No guarantee of schema consistency

### Technical Analysis
- Project already uses SQLx 0.7 with PostgreSQL features
- Migration files exist but aren't integrated with SQLx framework
- Custom `Database::initialize_schema()` creates parallel schema definition
- No compile-time query verification (`sqlx-data.json` missing)

## Decision

**ADOPT SQLx Migration Framework** as the standard database schema management approach for Block-Migrate.

### Rationale

#### Why SQLx Migrations
1. **Industry Standard**: Widely adopted in Rust ecosystem
2. **Built-in Versioning**: Automatic migration state tracking
3. **Transaction Safety**: Atomic migration execution
4. **Compile-time Verification**: Query checking at build time
5. **Production Ready**: Battle-tested in production environments
6. **Rollback Support**: Safe schema change reversal
7. **Embedded Execution**: Automatic migrations on application startup

#### Alternative Considered: Keep Custom Approach
- **Rejected**: High maintenance burden, operational risk, no rollback capability

#### Alternative Considered: Diesel Migrations
- **Rejected**: Would require major dependency changes, SQLx already integrated

## Implementation Plan

### Phase 1: SQLx CLI Setup
- Install SQLx CLI tool
- Restructure existing migration files to SQLx format
- Generate timestamp-based migration naming

### Phase 2: Embedded Migration Integration
- Add `sqlx::migrate!()` macro to codebase
- Create migration runner in `Database` struct
- Update application initialization to run migrations automatically

### Phase 3: Remove Custom Schema Code
- Delete `initialize_schema()` method
- Remove `migrate_to_hierarchical_schema()` method
- Consolidate all schema logic into migration files

### Phase 4: Production Readiness
- Add compile-time query verification
- Implement transaction boundaries for all migrations
- Add migration validation and error handling

## Implementation Details

### Directory Structure
```
migrations/
├── 20240101000001_initial_schema.sql
├── 20240101000002_eliminate_source_code.sql
└── 20240101000003_schema_alignment.sql
```

### Code Integration
```rust
// src/database/mod.rs
use sqlx::migrate::Migrator;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

impl Database {
    pub async fn run_migrations(&self) -> Result<()> {
        MIGRATOR.run(&self.pool).await?;
        Ok(())
    }
}
```

### Application Startup
```rust
// Automatic migration execution on startup
pub async fn setup_database() -> Result<Database> {
    let db = Database::new(&database_url).await?;
    db.run_migrations().await?;
    Ok(db)
}
```

## Migration Strategy

### Existing Data Safety
- Current database schemas will be preserved
- SQLx migration table (`_sqlx_migrations`) will be created
- Existing migration files will be converted to SQLx format
- No data loss during transition

### Rollback Plan
- Keep backup of current custom schema code (temporarily)
- Document manual rollback procedure if issues arise
- Test migration rollback capabilities thoroughly

## Success Criteria

1. ✅ SQLx CLI successfully installed and configured
2. ✅ All existing migrations converted to SQLx format
3. ✅ Embedded migrations working in application
4. ✅ Custom schema initialization code removed
5. ✅ Compile-time query verification enabled (`sqlx-data.json`)
6. ✅ All tests passing with new migration system
7. ✅ Production deployment using SQLx migrations

## Risks and Mitigations

### Risk: Migration File Conflicts
- **Mitigation**: Careful timestamp ordering of existing migrations
- **Mitigation**: Test migration order in clean database

### Risk: Production Deployment Issues
- **Mitigation**: Thorough testing in staging environment
- **Mitigation**: Database backup before migration deployment
- **Mitigation**: Rollback procedure documented

### Risk: Compile-time Query Verification Failures
- **Mitigation**: Generate `sqlx-data.json` with current schema
- **Mitigation**: Update CI/CD to maintain query verification

## Compliance

- **Rules-101 v1.2**: ✅ Production-ready database management
- **Rules-102 v1.2**: ✅ Industry standard tooling adoption
- **Rules-103 v1.2**: ✅ Risk mitigation through proper versioning

## Timeline

- **Phase 1**: 1 hour - SQLx CLI setup and file restructuring
- **Phase 2**: 1 hour - Embedded migration integration
- **Phase 3**: 30 minutes - Remove custom schema code
- **Phase 4**: 30 minutes - Production readiness features

**Total Estimated Time**: 3 hours

## Decision Maker
Architecture Team

## Implementation Date
2024-01-XX (Today)

## References
- [SQLx Migration Documentation](https://docs.rs/sqlx/latest/sqlx/migrate/index.html)
- [Current Migration Files](/migrations/)
- [Database Schema Module](/src/database/schema.rs)
