import { Before, After, type ITestCaseHookParameter, setDefaultTimeout, Status } from '@cucumber/cucumber';
import type { CustomWorld } from './world'; // Adjust path as necessary

// Set default timeout for Cucumber steps to 60 seconds (can be overridden by Playwright's own timeouts for its operations)
setDefaultTimeout(60 * 1000);

Before(async function (this: CustomWorld, scenario: ITestCaseHookParameter) {
    this.testName = scenario.pickle.name.replace(/ /g, "_");
    // The project name will be derived from PLAYWRIGHT_PROJECT_NAME env var or default in world.init()
    console.log(`\n--- Starting Scenario: ${scenario.pickle.name} (Project: ${process.env.PLAYWRIGHT_PROJECT_NAME || 'default'}) ---`);
    try {
        // Pass the project name from an environment variable or let init handle defaults
        await this.init({ project: process.env.PLAYWRIGHT_PROJECT_NAME });
        this.createNewUserData();
    } catch (error) {
        console.error(`Error in Before hook for scenario "${scenario.pickle.name}":`, error);
        throw error;
    }
});

After(async function (this: CustomWorld, scenario: ITestCaseHookParameter) {
    console.log(`--- Ending Scenario: ${scenario.pickle.name} (Project: ${this.projectName || 'default'}) ---`);
    console.log(`Status: ${scenario.result?.status}`);

    if (scenario.result?.status === Status.FAILED && this.page) {
        try {
            const screenshotDir = 'screenshots'; // Ensure this directory exists
            const screenshotPath = `${screenshotDir}/${this.projectName || 'default'}_${this.testName || 'failed_scenario'}_${Date.now()}.png`;
            await this.page.screenshot({ path: screenshotPath, fullPage: true });
            this.attach(await this.page.screenshot({ type: 'png' }), 'image/png');
            console.log(`Screenshot taken: ${screenshotPath}`);
        } catch (error) {
            console.error('Failed to take screenshot:', error);
        }
    }

    try {
        await this.close();
    } catch (error) {
        console.error('Error in After hook during close:', error);
    }
});
