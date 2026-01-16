# Migration Scripts

These scripts automate the UUID to Auto-Generated ID migration.

## Quick Start

```bash
cd /Users/markkovari/DEV/markkovari/experiments/surreal-backend

# Run all automated fixes
./scripts/run_all_fixes.sh
```

## Individual Scripts

### 1. `fix_repositories.sh`
Updates database repository files:
- Removes Uuid imports
- Updates Pet repository tests to use String IDs
- Prepares Doctor and Check repositories for manual updates

### 2. `fix_api_handlers.sh`
Updates API handler files:
- Changes `Path<Uuid>` to `Path<String>`
- Updates repository method calls to pass `&id` instead of `id`
- Updates all path parameter extractions

### 3. `fix_dtos.sh`
Updates Data Transfer Objects:
- Removes Uuid imports
- Changes all Uuid fields to String fields

### 4. `run_all_fixes.sh`
Master script that:
- Runs all automated fix scripts
- Provides checklist for manual updates
- Shows verification commands

## After Running Scripts

1. Review the changes: `git diff`
2. Check compilation: `cargo check --workspace`
3. Read `MIGRATION_GUIDE.md` for manual update patterns
4. Complete manual updates for repositories, migrations, and tests
5. Run tests: `cargo nextest run`

## Safety

All scripts use `sed -i ''` for in-place editing with backup support on macOS.
Review changes before committing!
