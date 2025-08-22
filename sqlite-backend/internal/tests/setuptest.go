package tests

import (
	"database/sql"
	"log"
	"testing"

	"github.com/pressly/goose/v3"
	_ "github.com/tursodatabase/go-libsql"
)

func SetupInMemoryDB(t *testing.T) *sql.DB {
	db, err := sql.Open("libsql", ":memory:")
	if err != nil {
		t.Fatalf("failed to open in-memory database: %v", err)
	}

	// Defer the closing of the connection. This automatically destroys the database.
	t.Cleanup(func() {
		db.Close()
	})

	// Set the database dialect for Goose
	if err := goose.SetDialect("sqlite3"); err != nil {
		t.Fatalf("failed to set database dialect: %v", err)
	}

	// Run migrations from the migrations directory
	if err := goose.Up(db, "../../db/migrations"); err != nil {
		t.Fatalf("failed to run migrations: %v", err)
	}

	log.Println("Migrations applied to in-memory database.")

	return db
}
