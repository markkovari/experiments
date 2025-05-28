import { type IWorldOptions, World, setWorldConstructor } from '@cucumber/cucumber';
import { type Browser, type BrowserContext, type Page, chromium, firefox, webkit, devices, type LaunchOptions, type BrowserContextOptions } from 'playwright';
import { faker } from '@faker-js/faker';
import playwrightConfig from '../../playwright.config';

export interface UserData {
    email: string;
    password?: string;
    firstName: string;
    lastName: string;
}

export interface CustomWorld extends World {
    browser?: Browser;
    context?: BrowserContext;
    page?: Page;
    userData?: UserData;
    testName?: string;
    projectName?: string; // Store the selected project name

    init: (options?: { project?: string }) => Promise<void>;
    close: () => Promise<void>;
    createNewUserData: (generatePassword?: boolean) => void;
}

class PlaywrightWorld extends World implements CustomWorld {
    public browser?: Browser;
    public context?: BrowserContext;
    public page?: Page;
    public userData?: UserData;
    public testName?: string;
    public projectName?: string;

    async init(options: { project?: string } = {}): Promise<void> {
        this.projectName = options.project || process.env.PLAYWRIGHT_PROJECT_NAME || playwrightConfig.projects?.[0]?.name || 'chromium-desktop';

        const project = playwrightConfig.projects?.find(p => p.name === this.projectName);

        if (!project || !project.use) {
            throw new Error(
                `Playwright project "${this.projectName}" not found or 'use' configuration is missing in playwright.config.ts. ` +
                `Available projects: ${playwrightConfig.projects?.map(p => p.name).join(', ')}`
            );
        }

        const { browserName, headless: projectHeadless, viewport, launchOptions, ...contextOptionsFromConfig } = project.use;
        const effectiveBrowserName = browserName || 'chromium';
        const effectiveHeadless = projectHeadless !== undefined ? projectHeadless : process.env.HEADED !== 'true';

        const playwrightLaunchOptions: LaunchOptions = {
            headless: effectiveHeadless,
            ...(launchOptions || {}), // Spread any launchOptions from the project config
        };

        // Launch browser
        switch (effectiveBrowserName.toLowerCase()) {
            case 'firefox':
                this.browser = await firefox.launch(playwrightLaunchOptions);
                break;
            case 'webkit':
                this.browser = await webkit.launch(playwrightLaunchOptions);
                break;
            //   case 'chromium':
            default:
                this.browser = await chromium.launch(playwrightLaunchOptions);
                break;
        }

        // Prepare context options
        // Start with options from project.use (excluding browserName, headless, launchOptions)
        const contextOptions: BrowserContextOptions = { ...contextOptionsFromConfig };

        // If a device is specified in project.use (like from ...devices['Pixel 5']),
        // it spreads viewport, userAgent etc. into contextOptions.
        // If not, and viewport is in project.use, apply it.
        if (!contextOptions.viewport && viewport) {
            contextOptions.viewport = viewport;
        }
        // Ensure default viewport if nothing is specified
        if (!contextOptions.viewport && !Object.values(devices).some(d => d.userAgent === contextOptions.userAgent)) { // Check if it's not a device
            contextOptions.viewport = { width: 1280, height: 720 };
        }


        // Add other common context options
        contextOptions.acceptDownloads = true;
        if (playwrightConfig.use?.baseURL) { // Example: picking up global baseURL
            contextOptions.baseURL = playwrightConfig.use.baseURL;
        }

        // Apply tracing or video recording options if defined globally or per-project
        // Example: (you'd need to manage dirs and when to enable these)
        // if (project.use.trace || playwrightConfig.use?.trace) {
        //   contextOptions.recordVideo = { dir: `test-results/videos/${this.testName || 'unknown'}-${this.projectName}` };
        // }

        this.context = await this.browser.newContext(contextOptions);

        // Set global timeout for Playwright operations if defined in playwright.config.ts
        if (playwrightConfig.timeout) {
            this.context.setDefaultTimeout(playwrightConfig.timeout);
            this.page?.setDefaultTimeout(playwrightConfig.timeout); // For page specific timeouts as well
        }


        this.page = await this.context.newPage();
    }

    async close(): Promise<void> {
        // ... (same close method)
        if (this.page) {
            await this.page.close();
        }
        if (this.context) {
            await this.context.close();
        }
        if (this.browser) {
            await this.browser.close();
        }
    }

    createNewUserData(generatePassword = true): void {
        // ... (same createNewUserData method)
        const baseFirstName = faker.person.firstName();
        const baseLastName = faker.person.lastName();
        this.userData = {
            firstName: baseFirstName,
            lastName: baseLastName,
            email: faker.internet.email({
                firstName: baseFirstName,
                lastName: baseLastName,
                provider: 'testmail.com'
            }),
            ...(generatePassword && { password: faker.internet.password({ length: 12, memorable: false, prefix: 'P@ss' }) })
        };
        console.log('Generated User Data:', this.userData);
    }
}

setWorldConstructor(PlaywrightWorld);
