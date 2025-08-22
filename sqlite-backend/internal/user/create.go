package user

import (
	"context"
	"database/sql"
	gen "sqlite-backend/db/gen"
)

type CreateUser struct {
	db *sql.DB
}

func (c CreateUser) Create(ctx context.Context, name string, email string) (*gen.User, error) {
	queries := gen.New(c.db)

	userParams := gen.CreateUserParams{
		Name:  name,
		Email: email,
	}

	user, err := queries.CreateUser(ctx, userParams)
	if err != nil {
		return nil, err
	}
	return &user, nil
}
