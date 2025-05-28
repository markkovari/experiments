import { defineConfig, devices } from '@playwright/test';

/**
 * This configuration is used to define "projects" that specify browser,
 * viewport, device emulation, etc.
 * Our Cucumber setup will read from this to configure the browser.
 */
export default defineConfig({
    // Timeout for the entire test run if you were using Playwright's test runner.
    // For Cucumber, individual step timeouts are set in hooks.ts (setDefaultTimeout).
    // Global timeout for a Playwright operation (e.g., page.goto()) can be set here.
    timeout: 60 * 1000, // 60 seconds, affects Playwright operations

    // Global expect timeout
    expect: {
        timeout: 10 * 1000, // 10 seconds for expect() assertions
    },

    projects: [
        {
            name: 'chromium-desktop',
            use: {
                browserName: 'chromium',
                headless: process.env.HEADED !== 'true',
                viewport: { width: 1280, height: 720 },
                launchOptions: {
                    // args: ["--start-maximized"] // Example launch option
                },
                // trace: 'on-first-retry', // Example: record trace on first retry (if using PW runner)
            },
        },
        {
            name: 'firefox-desktop',
            use: {
                browserName: 'firefox',
                headless: process.env.HEADED !== 'true',
                viewport: { width: 1280, height: 720 },
            },
        },
        {
            name: 'webkit-desktop',
            use: {
                browserName: 'webkit',
                headless: process.env.HEADED !== 'true',
                viewport: { width: 1280, height: 720 },
            },
        },
        {
            name: 'pixel5-mobile',
            use: {
                browserName: 'chromium', // Typically use chromium for mobile emulation
                headless: process.env.HEADED !== 'true',
                ...devices['Pixel 5'], // Spreads device-specific viewport, userAgent, etc.
            },
        },
        {
            name: 'iphone13-mobile',
            use: {
                browserName: 'webkit', // Use WebKit for Safari/iPhone emulation
                headless: process.env.HEADED !== 'true',
                ...devices['iPhone 13'],
            },
        },
        {
            name: 'ipad-tablet-landscape',
            use: {
                browserName: 'webkit', // Safari for iPad
                headless: process.env.HEADED !== 'true',
                ...devices['iPad Pro 11 landscape'],
            }
        },
        // You can add more projects for different configurations
        // Example: A project specifically for running headed for debugging
        {
            name: 'chromium-debug',
            use: {
                browserName: 'chromium',
                headless: process.env.HEADED !== 'true',
                viewport: { width: 1600, height: 900 },
                launchOptions: {
                    slowMo: 50, // Slow down operations by 50ms
                },
            },
        },
    ],

    /* Optional: Shared settings for all projects */
    // use: {
    //   baseURL: 'http://localhost:3000', // Base URL for page.goto()
    //   trace: 'retain-on-failure',       // Record trace on failure
    //   screenshot: 'only-on-failure',    // Take screenshot only on failure
    //   video: 'retain-on-failure',       // Record video on failure
    // },

    // Folder for test artifacts such as screenshots, videos, traces, etc.
    // This is more for Playwright Test runner, but good to be aware of.
    // Our hooks.ts currently saves screenshots to 'screenshots/'.
    outputDir: 'test-results/',

    // Web server configuration (if you want Playwright to start a dev server)
    // webServer: {
    //   command: 'npm run start',
    //   url: 'http://localhost:3000',
    //   reuseExistingServer: !process.env.CI,
    // },
});
