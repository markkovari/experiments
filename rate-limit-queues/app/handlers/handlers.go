package handlers

import (
	"asdasd/database" // Import the database package to access DB
	"asdasd/events"
	"asdasd/models" // Import the models package
	"encoding/json"
	"fmt"
	"log"
	"log/slog"
	"net/http"
	"strconv"

	"github.com/gorilla/mux" // Router for handling HTTP requests
	"gorm.io/gorm"           // GORM for database operations
)

// respondWithJSON is a helper function to send JSON responses.
func respondWithJSON(w http.ResponseWriter, code int, payload any) {
	response, err := json.Marshal(payload) // Marshal the payload into JSON bytes
	if err != nil {
		log.Printf("Error marshalling JSON: %v", err)
		w.WriteHeader(http.StatusInternalServerError) // Send 500 if marshalling fails
		return
	}
	w.Header().Set("Content-Type", "application/json") // Set Content-Type header
	w.WriteHeader(code)                                // Set HTTP status code
	_, err = w.Write(response)                         // Write the JSON response body
	if err != nil {
		println("YIKES")
	}
}

// respondWithError is a helper function to send JSON error responses.
func respondWithError(w http.ResponseWriter, code int, message string) {
	respondWithJSON(w, code, map[string]string{"error": message}) // Send a JSON object with an "error" field
}

// CreateUser handles POST /users to create a new user.
func CreateUser(w http.ResponseWriter, r *http.Request) {
	var user models.User
	// Decode the JSON request body into the User struct.
	if err := json.NewDecoder(r.Body).Decode(&user); err != nil {
		respondWithError(w, http.StatusBadRequest, "Invalid request payload")
		return
	}

	// Create the user record in the database.
	if result := database.DB.Create(&user); result.Error != nil {
		// Handle database errors (e.g., unique constraint violation for email).
		respondWithError(w, http.StatusInternalServerError, result.Error.Error())
		return
	}

	err := events.Send([]byte("hello"))
	if err != nil {
		slog.Warn("was not able to send message")
	}
	respondWithJSON(w, http.StatusCreated, user) // Respond with the created user and 201 status
}

// GetUsers handles GET /users to retrieve all users.
func GetUsers(w http.ResponseWriter, r *http.Request) {
	var users []models.User
	// Preload "Properties" to include the associated properties in the response
	// when fetching users. This avoids N+1 query problems.
	if result := database.DB.Preload("Properties").Find(&users); result.Error != nil {
		respondWithError(w, http.StatusInternalServerError, result.Error.Error())
		return
	}
	respondWithJSON(w, http.StatusOK, users) // Respond with the list of users
}

// GetUserByID handles GET /users/{id} to retrieve a single user by ID.
func GetUserByID(w http.ResponseWriter, r *http.Request) {
	vars := mux.Vars(r)                              // Get path variables from the request
	id, err := strconv.ParseUint(vars["id"], 10, 64) // Parse the ID string to uint64
	if err != nil {
		respondWithError(w, http.StatusBadRequest, "Invalid user ID")
		return
	}

	var user models.User
	// Preload "Properties" and find the user by ID.
	if result := database.DB.Preload("Properties").First(&user, id); result.Error != nil {
		if result.Error == gorm.ErrRecordNotFound {
			respondWithError(w, http.StatusNotFound, "User not found") // 404 if user not found
		} else {
			respondWithError(w, http.StatusInternalServerError, result.Error.Error()) // 500 for other DB errors
		}
		return
	}
	respondWithJSON(w, http.StatusOK, user) // Respond with the found user
}

// CreateProperty handles POST /properties to create a new property.
func CreateProperty(w http.ResponseWriter, r *http.Request) {
	var property models.Property
	// Decode the JSON request body into the Property struct.
	if err := json.NewDecoder(r.Body).Decode(&property); err != nil {
		respondWithError(w, http.StatusBadRequest, "Invalid request payload")
		return
	}

	// Create the property record in the database.
	if result := database.DB.Create(&property); result.Error != nil {
		// Handle database errors (e.g., unique constraint violation for address).
		respondWithError(w, http.StatusInternalServerError, result.Error.Error())
		return
	}
	respondWithJSON(w, http.StatusCreated, property) // Respond with the created property and 201 status
}

// GetProperties handles GET /properties to retrieve all properties.
func GetProperties(w http.ResponseWriter, r *http.Request) {
	var properties []models.Property
	// Preload "Users" to include the associated users in the response.
	if result := database.DB.Preload("Users").Find(&properties); result.Error != nil {
		respondWithError(w, http.StatusInternalServerError, result.Error.Error())
		return
	}
	respondWithJSON(w, http.StatusOK, properties) // Respond with the list of properties
}

// GetPropertyByID handles GET /properties/{id} to retrieve a single property by ID.
func GetPropertyByID(w http.ResponseWriter, r *http.Request) {
	vars := mux.Vars(r)                              // Get path variables
	id, err := strconv.ParseUint(vars["id"], 10, 64) // Parse ID
	if err != nil {
		respondWithError(w, http.StatusBadRequest, "Invalid property ID")
		return
	}

	var property models.Property
	// Preload "Users" and find the property by ID.
	if result := database.DB.Preload("Users").First(&property, id); result.Error != nil {
		if result.Error == gorm.ErrRecordNotFound {
			respondWithError(w, http.StatusNotFound, "Property not found") // 404 if property not found
		} else {
			respondWithError(w, http.StatusInternalServerError, result.Error.Error()) // 500 for other DB errors
		}
		return
	}
	respondWithJSON(w, http.StatusOK, property) // Respond with the found property
}

// AssociateUserProperty handles POST /users/{userId}/properties/{propertyId}
// This creates a many-to-many association between an existing user and an existing property.
func AssociateUserProperty(w http.ResponseWriter, r *http.Request) {
	vars := mux.Vars(r)
	userID, err := strconv.ParseUint(vars["userId"], 10, 64)
	if err != nil {
		respondWithError(w, http.StatusBadRequest, "Invalid user ID")
		return
	}
	propertyID, err := strconv.ParseUint(vars["propertyId"], 10, 64)
	if err != nil {
		respondWithError(w, http.StatusBadRequest, "Invalid property ID")
		return
	}

	var user models.User
	// Find the user by ID.
	if result := database.DB.First(&user, userID); result.Error != nil {
		if result.Error == gorm.ErrRecordNotFound {
			respondWithError(w, http.StatusNotFound, "User not found")
		} else {
			respondWithError(w, http.StatusInternalServerError, result.Error.Error())
		}
		return
	}

	var property models.Property
	// Find the property by ID.
	if result := database.DB.First(&property, propertyID); result.Error != nil {
		if result.Error == gorm.ErrRecordNotFound {
			respondWithError(w, http.StatusNotFound, "Property not found")
		} else {
			respondWithError(w, http.StatusInternalServerError, result.Error.Error())
		}
		return
	}

	// Append the property to the user's properties. GORM handles inserting into the join table.
	// GORM's `Append` method will automatically check for existing associations and avoid duplicates.
	if result := database.DB.Model(&user).Association("Properties").Append(&property); result != nil {
		// If `Append` returns an error (which it does not for duplicate, it just doesn't add),
		// it means there was a database issue.
		respondWithError(w, http.StatusInternalServerError, fmt.Sprintf("Error associating user and property: %v", result))
		return
	}

	respondWithJSON(w, http.StatusOK, map[string]string{"message": "User associated with property successfully"})
}

// DisassociateUserProperty handles DELETE /users/{userId}/properties/{propertyId}
// This removes a many-to-many association between a user and a property.
func DisassociateUserProperty(w http.ResponseWriter, r *http.Request) {
	vars := mux.Vars(r)
	userID, err := strconv.ParseUint(vars["userId"], 10, 64)
	if err != nil {
		respondWithError(w, http.StatusBadRequest, "Invalid user ID")
		return
	}
	propertyID, err := strconv.ParseUint(vars["propertyId"], 10, 64)
	if err != nil {
		respondWithError(w, http.StatusBadRequest, "Invalid property ID")
		return
	}

	var user models.User
	// Find the user.
	if result := database.DB.First(&user, userID); result.Error != nil {
		if result.Error == gorm.ErrRecordNotFound {
			respondWithError(w, http.StatusNotFound, "User not found")
		} else {
			respondWithError(w, http.StatusInternalServerError, result.Error.Error())
		}
		return
	}

	var property models.Property
	// Find the property.
	if result := database.DB.First(&property, propertyID); result.Error != nil {
		if result.Error == gorm.ErrRecordNotFound {
			respondWithError(w, http.StatusNotFound, "Property not found")
		} else {
			respondWithError(w, http.StatusInternalServerError, result.Error.Error())
		}
		return
	}

	// Delete the association from the join table.
	if result := database.DB.Model(&user).Association("Properties").Delete(&property); result != nil {
		// If `Delete` returns an error, it indicates a database issue.
		respondWithError(w, http.StatusInternalServerError, fmt.Sprintf("Error disassociating user and property: %v", result))
		return
	}

	respondWithJSON(w, http.StatusOK, map[string]string{"message": "User disassociated from property successfully"})
}
