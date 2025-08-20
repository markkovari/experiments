# Event-Driven NATS Example

This project is an example of an event-driven application using NATS for messaging. It consists of a TypeScript publisher and two Go subscribers. The events are documented using EventCatalog.

## Prerequisites

- Docker
- Just

## Getting Started

1.  **Start the NATS server:**
    ```bash
    just up
    ```

2.  **Build and run the services:**
    Due to issues with the development environment, the `npm install` and `go get` commands could not be run successfully. Therefore, the services cannot be built or run at the moment.

    Once the environment issues are resolved, you would typically run the following commands:

    -   **Build the publisher:**
        ```bash
        just build-publisher
        ```
    -   **Run the publisher:**
        ```bash
        just publish
        ```
    -   **Build the subscribers:**
        ```bash
        just build-subscribers
        ```
    -   **Run the subscribers:**
        ```bash
        just run-subscriber1
        just run-subscriber2
        ```

3.  **View the EventCatalog:**
    The EventCatalog documentation has been set up manually. To view it, you would typically run a command like `npm run start` from the `eventcatalog` directory, after installing the dependencies.

## Project Structure

-   `publisher/`: The TypeScript publisher application.
-   `subscriber1/`: The first Go subscriber application.
-   `subscriber2/`: The second Go subscriber application.
-   `eventcatalog/`: The EventCatalog documentation.
-   `docker-compose.yaml`: Defines the NATS service.
-   `justfile`: Provides commands for managing the project.

## Environment Issues

During the development of this project, we faced persistent issues with the development environment that prevented the installation of dependencies for both the Node.js and Go projects. This means that the services are not currently buildable or runnable. The issue seems to be related to the environment's ability to resolve the current working directory.
