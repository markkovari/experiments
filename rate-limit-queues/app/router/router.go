package router

import (
	"asdasd/handlers" // Import the handlers package
	"log"

	"github.com/gorilla/mux" // Import Gorilla Mux router
)

// SetupRouter creates and configures the Gorilla Mux router with all API routes.
func SetupRouter() *mux.Router {
	r := mux.NewRouter() // Create a new router instance

	// User routes
	r.HandleFunc("/users", handlers.CreateUser).Methods("POST")      // Create a new user
	r.HandleFunc("/users", handlers.GetUsers).Methods("GET")         // Get all users
	r.HandleFunc("/users/{id}", handlers.GetUserByID).Methods("GET") // Get a user by ID

	// Property routes
	r.HandleFunc("/properties", handlers.CreateProperty).Methods("POST")      // Create a new property
	r.HandleFunc("/properties", handlers.GetProperties).Methods("GET")        // Get all properties
	r.HandleFunc("/properties/{id}", handlers.GetPropertyByID).Methods("GET") // Get a property by ID

	// Many-to-many association routes
	// Associate a user with a property
	r.HandleFunc("/users/{userId}/properties/{propertyId}", handlers.AssociateUserProperty).Methods("POST")
	// Disassociate a user from a property
	r.HandleFunc("/users/{userId}/properties/{propertyId}", handlers.DisassociateUserProperty).Methods("DELETE")

	log.Println("API routes configured.")
	return r // Return the configured router
}
