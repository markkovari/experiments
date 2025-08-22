-- name: GetUserByID :one
SELECT * FROM users WHERE id = ?;

-- name: CreateUser :one
INSERT INTO users (name, email) VALUES (?, ?) RETURNING *;

-- name: ListUsers :many
SELECT * FROM users ORDER BY name;
