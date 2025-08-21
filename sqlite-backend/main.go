package main

import (
	"context"
	"database/sql"
	"fmt"
	"log"
	"math/rand/v2"
	gen "sqlite-backend/db/gen"

	_ "github.com/mattn/go-sqlite3"
)

func main() {
	// Connect to the database
	// For libSQL, you might use a different driver.
	db, err := sql.Open("sqlite3", "./sqlite.db")
	if err != nil {
		log.Fatal(err)
	}
	defer db.Close()

	// Create a new sqlc queries object
	queries := gen.New(db)
	ctx := context.Background()

	rand := rand.Uint32()

	// Use the generated typesafe functions
	userParams := gen.CreateUserParams{
		Name:  "John Doe",
		Email: fmt.Sprintf("john.doe%d@example.com", rand),
	}
	user, err := queries.CreateUser(ctx, userParams)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Println("Created user:", user.Name)

	// Retrieve the user by ID
	retrievedUser, err := queries.GetUserByID(ctx, user.ID)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Println("Retrieved user:", retrievedUser.Email)
}
