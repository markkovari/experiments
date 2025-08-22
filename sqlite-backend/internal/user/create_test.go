package user

import (
	tests "sqlite-backend/internal/tests"
	"testing"
)

func TestUserCreateShouldCreateUser(t *testing.T) {
	db := tests.SetupInMemoryDB(t)

	defer db.Close()

	useCase := CreateUser{db: db}
	_, err := useCase.Create(t.Context(), "some-name", "some@email.com")
	if err != nil {
		t.Fatal("user cannot be create in empty database")
	}
}

func TestUserCreateShoulFailWithUniqueEmail(t *testing.T) {
	db := tests.SetupInMemoryDB(t)

	defer db.Close()

	useCase := CreateUser{db: db}
	_, err := useCase.Create(t.Context(), "some-name", "some@email.com")
	if err != nil {
		t.Fatal("user cannot be create in empty database")
	}
	_, err = useCase.Create(t.Context(), "some-name", "some@email.com")
	if err == nil {
		t.Fatal("user should not be able to storw with the same email")
	}
}
