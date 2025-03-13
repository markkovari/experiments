package main

import (
	"context"
	"fmt"
	"log"
	"os"
	"testing"
	"time"

	"github.com/cucumber/godog"
	"github.com/cucumber/godog/colors"
	"github.com/testcontainers/testcontainers-go"
	"github.com/testcontainers/testcontainers-go/wait"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"

	_ "github.com/jackc/pgx/v5/stdlib"
)

type User struct {
	ID    int `gorm:"primaryKey"`
	Name  string
	Email string
}

type TestContext struct {
	container testcontainers.Container
	db        *gorm.DB
}

func setupTestContainer() (*TestContext, error) {
	ctx := context.Background()
	postgresUser := "testuser"
	postgresPassword := "testpass"
	postgresDatabase := "testdb"

	// Create PostgreSQL container
	req := testcontainers.ContainerRequest{
		Image:        "postgres:16-alpine",
		ExposedPorts: []string{"5432/tcp"},
		Env: map[string]string{
			"POSTGRES_USER":     postgresUser,
			"POSTGRES_PASSWORD": postgresPassword,
			"POSTGRES_DB":       postgresDatabase,
		},
		WaitingFor: wait.ForLog("database system is ready to accept connections").WithPollInterval(time.Second * 5),
	}

	container, err := testcontainers.GenericContainer(ctx, testcontainers.GenericContainerRequest{
		ContainerRequest: req,
		Started:          true,
	})
	if err != nil {
		return nil, err
	}

	host, err := container.Host(ctx)
	if err != nil {
		return nil, err
	}

	port, err := container.MappedPort(ctx, "5432")
	if err != nil {
		return nil, err
	}

	dsn := fmt.Sprintf("host=%s port=%s user=%s password=%s dbname=%s sslmode=disable",
		host, port.Port(), postgresUser, postgresPassword, postgresDatabase)

	db, err := gorm.Open(postgres.New(postgres.Config{
		DSN:                  dsn,
		PreferSimpleProtocol: true,
	}), &gorm.Config{})

	if err != nil {
		fmt.Println(err.Error())
		return nil, err
	}

	// Migrate schema
	err = db.AutoMigrate(&User{})
	if err != nil {
		return nil, err
	}
	return &TestContext{
		db:        db,
		container: container,
	}, nil
}

func teardownTestContainer(t TestContext) {
	if t.container != nil {
		_ = t.container.Terminate(context.Background())
	}
}

// Step Definitions
func iHaveAUserWithNameAndEmail(context *TestContext, name, email string) error {
	user := User{Name: name, Email: email}
	return context.db.Create(&user).Error
}

func iShouldFindAUserWithName(context *TestContext, name string) error {
	var user User
	result := context.db.First(&user, "name = ?", name)
	if result.Error != nil {
		return result.Error
	}
	if user.Name != name {
		return fmt.Errorf("expected name %s, but got %s", name, user.Name)
	}
	return nil
}

func InitializeScenario(s *godog.ScenarioContext, tc *TestContext) {
	s.Given(`^I have a user with name "([^"]*)" and email "([^"]*)"$`, func(name, email string) {
		_ = iHaveAUserWithNameAndEmail(tc, name, email)
	})
	s.Then(`^I should find a user with name "([^"]*)"$`, func(name string) {
		_ = iShouldFindAUserWithName(tc, name)
	})
}

func TestMain(m *testing.M) {

	ctx, err := setupTestContainer()
	if err != nil {
		log.Fatalf("Could not set up test container: %v", err)
	}

	defer teardownTestContainer(*ctx)
	status := godog.TestSuite{
		Name: "godogs",
		ScenarioInitializer: func(sc *godog.ScenarioContext) {
			InitializeScenario(sc, ctx)
		},
		Options: &godog.Options{
			Format:      "pretty",
			Output:      colors.Colored(os.Stdout),
			Concurrency: 10,
		},
	}.Run()

	os.Exit(status)
}
