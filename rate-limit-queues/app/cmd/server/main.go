package main

import (
	"asdasd/database" // Import the database package for connection management
	"asdasd/events"
	"asdasd/router" // Import the router package for API route setup
	"log"
	"net/http"
	"os" // Used to access environment variables
)

func main() {
	_, err := events.InitQueue(events.QueueName)
	if err != nil {
		log.Fatal("cannot connect to events")
	}
	// Initialize the database connection and perform migrations.
	database.InitDB()
	// Ensure the database connection is closed when the main function exits.
	// This is important for resource management.
	defer database.CloseDB()

	// Setup the HTTP router with all defined API endpoints.
	r := router.SetupRouter()

	// Get the port from the environment variable "PORT".
	// If not set, default to "8080".
	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	log.Printf("Server starting on port %s...", port)
	// Start the HTTP server.
	// ListenAndServe blocks until the server is shut down or an error occurs.
	if err := http.ListenAndServe(":"+port, r); err != nil {
		// Log a fatal error if the server fails to start.
		log.Fatalf("Server failed to start: %v", err)
	}
}
