package models

import "gorm.io/gorm"

// User represents a user in the system.
// gorm.Model provides default fields like ID, CreatedAt, UpdatedAt, DeletedAt.
type User struct {
	gorm.Model
	Name       string     `json:"name"`                                         // User's name
	Email      string     `json:"email" gorm:"unique"`                          // User's email, must be unique
	Properties []Property `json:"properties" gorm:"many2many:user_properties;"` // Many-to-many relationship with Property.
	// `gorm:"many2many:user_properties;"` tells GORM to create a join table named `user_properties`
	// for this relationship.
}

// Property represents a property detail.
type Property struct {
	gorm.Model
	Address string  `json:"address" gorm:"unique"`                   // Property address, must be unique
	City    string  `json:"city"`                                    // City where the property is located
	Price   float64 `json:"price"`                                   // Price of the property
	Users   []User  `json:"users" gorm:"many2many:user_properties;"` // Many-to-many relationship with User.
	// This also uses the same `user_properties` join table.
}
