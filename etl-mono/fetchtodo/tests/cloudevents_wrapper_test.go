package natscloudevents_nats

import (
	"context"
	"encoding/json"
	"testing"
	"time"

	cloudevents "github.com/cloudevents/sdk-go/v2"
	ceevent "github.com/cloudevents/sdk-go/v2/event"
	"github.com/google/uuid"
	"github.com/stretchr/testify/require"
	"github.com/testcontainers/testcontainers-go"
	"github.com/testcontainers/testcontainers-go/wait"
)

func TestCloudEventsOverNATS(t *testing.T) {
	ctx := context.Background()

	// Start NATS container
	req := testcontainers.ContainerRequest{
		Image:        "nats:latest",
		ExposedPorts: []string{"4222/tcp"},
		Cmd:          []string{"-js"},
		WaitingFor:   wait.ForLog("Server is ready"),
	}
	natsC, err := testcontainers.GenericContainer(ctx, testcontainers.GenericContainerRequest{
		ContainerRequest: req,
		Started:          true,
	})
	require.NoError(t, err)
	defer natsC.Terminate(ctx)

	// Get the host and port
	host, err := natsC.Host(ctx)
	require.NoError(t, err)

	port, err := natsC.MappedPort(ctx, "4222")
	require.NoError(t, err)

	url := "nats://" + host + ":" + port.Port()

	// Setup your wrapper
	client, err := NewCloudEventNATS(url)
	require.NoError(t, err)
	defer client.Close()

	err = client.EnsureStream("CLOUDEVENTS", []string{"test.>"})
	require.NoError(t, err)

	// Setup receiver
	received := make(chan ceevent.Event, 1)

	err = client.Subscribe("test.subject", "test-consumer", func(evt ceevent.Event) {
		received <- evt
	})
	require.NoError(t, err)

	// Publish an event
	event := ceevent.New()
	event.SetID(uuid.NewString())
	event.SetSource("testcase")
	event.SetType("example.test")
	event.SetTime(time.Now())
	err = event.SetData(cloudevents.ApplicationJSON, map[string]string{"message": "from test"})
	require.NoError(t, err)

	err = client.Publish("test.subject", event)
	require.NoError(t, err)

	// Wait and assert
	select {
	case e := <-received:
		require.Equal(t, "example.test", e.Type())
		require.Equal(t, "testcase", e.Source())
		var payload map[string]string
		err := json.Unmarshal(e.Data(), &payload)
		require.NoError(t, err)
		require.Equal(t, "from test", payload["message"])
	case <-time.After(5 * time.Second):
		t.Fatal("did not receive event")
	}
}
