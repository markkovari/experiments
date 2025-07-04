package database

import (
	"asdasd/models" // Import the models package
	"log"
	"os"

	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

// DB is a global variable that holds the GORM database connection instance.
var DB *gorm.DB

// InitDB initializes the database connection and performs schema migrations.
func InitDB() {
	var err error
	// Retrieve the database connection string from environment variables.
	// This makes the application configurable without recompilation.
	dsn := os.Getenv("DATABASE_URL")
	if dsn == "" {
		// Log a fatal error if the environment variable is not set,
		// as the application cannot function without a database.
		log.Fatal("DATABASE_URL environment variable not set")
	}

	// Open a connection to the PostgreSQL database using the DSN.
	// gorm.Config{} can be used to pass custom GORM configurations.
	DB, err = gorm.Open(postgres.Open(dsn), &gorm.Config{})
	if err != nil {
		// Log a fatal error if the connection fails.
		log.Fatalf("Failed to connect to database: %v", err)
	}

	log.Println("Database connection established successfully.")

	// AutoMigrate will automatically create or update database tables
	// based on the defined GORM models (User and Property).
	// It will create `users`, `properties`, and the `user_properties` join table.
	err = DB.AutoMigrate(&models.User{}, &models.Property{})
	if err != nil {
		// Log a fatal error if migrations fail.
		log.Fatalf("Failed to auto migrate database: %v", err)
	}
	log.Println("Database migrations completed successfully.")
}

// CloseDB closes the underlying SQL database connection.
// This should be called when the application shuts down.
func CloseDB() {
	sqlDB, err := DB.DB() // Get the underlying *sql.DB from GORM
	if err != nil {
		log.Printf("Error getting underlying SQL DB: %v", err)
		return
	}
	err = sqlDB.Close() // Close the connection
	if err != nil {
		log.Printf("Error closing database connection: %v", err)
	}
	log.Println("Database connection closed.")
}
