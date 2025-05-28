import { Given, When, Then } from '@cucumber/cucumber';
import { expect, type Page } from '@playwright/test';
import type { CustomWorld } from '../support/world';

const throwIfNoPage = (page: Page | undefined) => {
    if (page === undefined) {
        throw new Error("Page is not initialized")
    }
}

const tinp = throwIfNoPage

Given('I am on the registration page', async function (this: CustomWorld) {
    if (!this.page) throw new Error("Page is not initialized")
    // Replace with the actual URL of your registration page
    await this.page.goto('http://wikipedia.com');
    await expect(this.page.locator('h1')).toContainText('Wikipedia'); // Example assertion
});

When('I fill in the registration form with my details', async function (this: CustomWorld) {
    if (!this.page || !this.userData) throw new Error('Page or user data not initialized');

    // Example: Fill form fields. Adjust selectors based on your application.
    // await this.page.locator('#firstName').fill(this.userData.firstName);
    // await this.page.locator('#lastName').fill(this.userData.lastName);
    // await this.page.locator('#email').fill(this.userData.email);
    // if (this.userData.password) {
    //     await this.page.locator('#password').fill(this.userData.password);
    //     await this.page.locator('#confirmPassword').fill(this.userData.password);
    // }
    await this.page.waitForTimeout(100);
    // Add other fields as necessary, e.g., terms and conditions checkbox
    // await this.page.locator('#termsAndConditions').check();

    console.log(`Filled form with: ${JSON.stringify(this.userData)}`);
});

When('I fill in the registration form with email {string} and other details', async function (this: CustomWorld, email: string) {
    if (!this.page || !this.userData) throw new Error('Page or user data not initialized');

    // Use the provided email, but other details from generated userData
    // await this.page.locator('#firstName').fill(this.userData.firstName);
    // await this.page.locator('#lastName').fill(this.userData.lastName);
    // await this.page.locator('#email').fill(email); // Use the specific email from the step
    // if (this.userData.password) { // Still generate and use a password for other fields
    //     await this.page.locator('#password').fill(this.userData.password);
    //     await this.page.locator('#confirmPassword').fill(this.userData.password);
    // }
    await this.page.waitForTimeout(100);
    console.log(`Filled form with email: ${email} and other details: ${JSON.stringify(this.userData)}`);
});


When('I submit the registration form', async function (this: CustomWorld) {
    if (!this.page) throw new Error('Page not initialized');
    // Example: Click the submit button. Adjust selector.
    // await this.page.locator('button[type="submit"]').click();
    await this.page.waitForTimeout(100);
});

Then('I should see a success message', async function (this: CustomWorld) {
    if (!this.page) throw new Error('Page not initialized');
    // // Example: Check for a success message. Adjust selector and text.
    // const successMessage = this.page.locator('.alert-success'); // Or whatever your success message selector is
    // await expect(successMessage).toBeVisible();
    // await expect(successMessage).toContainText('Registration successful');
    await this.page.waitForTimeout(100);
});

Then('I should be redirected to my dashboard', async function (this: CustomWorld) {
    if (!this.page) throw new Error('Page not initialized');
    // // Example: Check if the URL is the dashboard URL.
    // await expect(this.page).toHaveURL(/.*\/dashboard/); // Matches any URL ending with /dashboard
    // // And perhaps check for a unique element on the dashboard
    // await expect(this.page.locator('h1')).toContainText('My Dashboard');
    await this.page.waitForTimeout(100);
});

// This step is for setting up a precondition, actual implementation would depend on your test data strategy
Given('a user with email {string} already exists', async function (this: CustomWorld, email: string) {
    // This step would typically involve:
    // 1. Calling an API to create this user.
    // 2. Seeding a database.
    // 3. Or, if your UI allows, navigating and creating this user through the UI (less ideal for this specific precondition).
    // For this example, we'll just log it. In a real scenario, you'd implement the actual creation.
    console.log(`PRECONDITION: Ensuring user with email "${email}" exists.`);
    // If you have a way to quickly create a user (e.g., API endpoint):
    // await createUserViaApi({ ...this.userData, email: email });
    // For now, this step doesn't interact with the browser directly, it sets up a state.
});

Then('I should see an error message indicating the email is already taken', async function (this: CustomWorld) {
    if (!this.page) throw new Error('Page not initialized');
    // // Example: Check for an error message. Adjust selector and text.
    // const errorMessage = this.page.locator('.alert-danger'); // Or your error message selector for this specific case
    // await expect(errorMessage).toBeVisible();
    // await expect(errorMessage).toContainText('email is already taken'); // Or similar text
    await this.page.waitForTimeout(100);
});
