# Playwright Cucumber TypeScript E2E Framework

This project provides a basic structure for End-to-End (E2E) testing using Playwright, Cucumber, and TypeScript.

## Project Structure


.
├── cucumber.js                 # Cucumber.js configuration
├── package.json                # Project dependencies and scripts
├── tsconfig.json               # TypeScript compiler options
├── src/
│   ├── features/               # Cucumber feature files
│   │   └── user_registration.feature
│   ├── step-definitions/       # Step definition files
│   │   └── user_registration.steps.ts
│   └── support/                # Support files (hooks, world)
│       ├── world.ts
│       └── hooks.ts
├── screenshots/                # Will be created automatically for failed tests (if enabled in hooks)
└── README.md


## Prerequisites

* Node.js (v18 or higher recommended)
* npm or yarn

## Setup

1.  **Clone the repository (or create these files in your project):**
    ```bash
    # If you were cloning:
    # git clone <repository-url>
    # cd playwright-cucumber-ts-framework
    ```

2.  **Install dependencies:**
    ```bash
    npm install
    ```
    This will also run `npx playwright install --with-deps` (due to the `postinstall` script in `package.json`) to download the necessary browser binaries. If it doesn't, you can run it manually:
    ```bash
    npx playwright install --with-deps
    ```

3.  **Install Faker.js for generating mock data:**
    ```bash
    npm install --save-dev @faker-js/faker
    ```

## Running Tests

1.  **Execute all tests:**
    By default, tests run in headless mode using Chromium.
    ```bash
    npm test
    ```

2.  **Run tests in headed mode:**
    To see the browser interactions, you can run tests in headed mode:
    ```bash
    npm run test:headed
    ```
    This sets the `HEADED=true` environment variable, which is picked up by the `hooks.ts` file.

3.  **Run tests with a specific browser:**
    You can specify the browser by setting the `BROWSER` environment variable. Supported values are `chromium`, `firefox`, `webkit`.
    ```bash
    # Example: Run with Firefox in headed mode
    HEADED=true BROWSER=firefox npm test
    ```
    The `world.ts` file reads this environment variable.

4.  **Run specific features or scenarios:**
    Cucumber.js allows you to run specific features or scenarios using tags or file paths.
    ```bash
    # Run a specific feature file
    npm test -- src/features/user_registration.feature

    # Run scenarios with a specific tag (add @yourtag to your .feature file)
    # npm test -- --tags "@yourtag"
    ```

## Configuration

* **Cucumber:** Configuration is in `cucumber.js`. You can modify paths, formatting options, etc.
* **TypeScript:** Configuration is in `tsconfig.json`.
* **Playwright/World:** Browser launch options and user data generation are in `src/support/world.ts` and `src/support/hooks.ts`.

## Key Components

* **`src/support/world.ts`:**
    * Manages Playwright `browser`, `context`, and `page` instances.
    * Includes `createNewUserData()` method using `@faker-js/faker` to generate random user details (first name, last name, email, and an optional password). This data is refreshed for each scenario.
* **`src/support/hooks.ts`:**
    * `Before` hook: Initializes a new browser page and generates user data before each scenario.
    * `After` hook: Closes the browser page after each scenario. It also includes logic to take a screenshot if a scenario fails and save it to the `screenshots/` directory.
* **`src/features/`:** Contains `.feature` files written in Gherkin.
* **`src/step-definitions/`:** Contains TypeScript files that implement the steps defined in the feature files.

## Customizing User Data

The `UserData` interface and `createNewUserData` method in `src/support/world.ts` can be customized to fit the specific needs of your application's user model.

## Screenshots on Failure

The `After` hook in `src/support/hooks.ts` is configured to automatically take a screenshot if a scenario fails. Screenshots are saved in the `screenshots/` directory. Ensure this directory exists or can be created by the script.

## Further Development

* **Page Object Model (POM):** For larger applications, consider implementing the Page Object Model to create a more maintainable and scalable test suite.
* **Reporting:** Integrate more advanced HTML reporters like `cucumber-html-reporter`.
* **Environment Configuration:** Manage different test environments (dev, staging, prod) with separate configuration files or environment variables.
* **API Helpers:** Add helper functions for interacting with your application's API for test setup or verification.
