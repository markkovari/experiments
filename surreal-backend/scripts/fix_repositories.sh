#!/bin/bash
set -e

echo "🔧 Fixing Pet Repository Tests..."

# Update pet repository tests
sed -i '' 's/let owner_id = Uuid::new_v4();/let owner_id = "users:test123".to_string();/g' \
  crates/db/src/repository/pet.rs

sed -i '' 's/assert_eq!(created.name, pet.name);/assert_eq!(created.name, pet.name);\n        assert!(created.id.is_some());\n\n        let id = created.id.as_ref().unwrap();/g' \
  crates/db/src/repository/pet.rs

echo "✅ Pet repository tests updated"

echo "🔧 Removing Uuid imports from repositories..."

# Remove uuid imports from doctor and check repositories
sed -i '' '/^use uuid::Uuid;$/d' crates/db/src/repository/doctor.rs
sed -i '' '/^use uuid::Uuid;$/d' crates/db/src/repository/check.rs

echo "✅ Uuid imports removed"
echo ""
echo "⚠️  Manual updates still needed for:"
echo "   - Doctor repository implementation and tests"
echo "   - Check repository implementation and tests"
echo "   - Pet repository test assertions"
echo ""
echo "See MIGRATION_GUIDE.md for details"
