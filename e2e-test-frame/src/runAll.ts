import { execSync } from 'node:child_process';
// Adjust the import path if your playwright.config.ts is not in the root
// or if this script is placed in a sub-directory (e.g., ./scripts/run-cucumber-projects.ts)
import playwrightConfig from '../playwright.config';

async function runAllConfiguredProjects() {
    const projects = playwrightConfig.projects;

    if (!projects || projects.length === 0) {
        console.warn('No projects found in playwright.config.ts. Exiting.');
        return;
    }

    const projectNames = projects.map(p => p.name);
    console.log(`Found projects to run: ${projectNames.join(', ')}`);
    console.log('----------------------------------------------------');

    const resultsSummary = [];

    for (const projectName of projectNames) {
        if (!projectName) {
            console.warn('Found a project without a name. Skipping.');
            continue;
        }
        console.log(`\n🚀 Starting tests for project: ${projectName}`);
        console.log('----------------------------------------------------');
        try {
            // Construct the command to run Cucumber tests for the current project.
            // This uses npx to ensure the local cucumber-js is used.
            // `stdio: 'inherit'` ensures that the output of cucumber-js is displayed in real-time.
            const command = `PLAYWRIGHT_PROJECT_NAME=${projectName} npx cucumber-js`;
            console.log(`🔩 Executing: ${command}`);

            execSync(command, { stdio: 'inherit' });

            console.log(`✅ Successfully completed tests for project: ${projectName}`);
            resultsSummary.push({ project: projectName, status: 'PASSED' });
        } catch (error) {
            // execSync throws an error if the command exits with a non-zero status code (i.e., tests failed)
            console.error(`❌ Tests FAILED for project: ${projectName}`);
            resultsSummary.push({ project: projectName, status: 'FAILED' });
            // Optional: Decide if you want to stop on the first failure or continue with other projects.
            // To stop on first failure, you could re-throw the error or process.exit(1) here.
            // This script currently continues with other projects.
        }
        console.log('----------------------------------------------------');
    }

    // Print a summary of all project runs
    console.log('\n\n📋 === Test Run Summary ===');
    for (const result of resultsSummary) {
        console.log(`Project: ${result.project.padEnd(25)} Status: ${result.status}`);
    }

    console.log('===========================');

    const failedCount = resultsSummary.filter(r => r.status === 'FAILED').length;
    if (failedCount > 0) {
        console.error(`\n🚨 Total failed project configurations: ${failedCount}`);
        process.exit(1); // Exit with a non-zero code to indicate failure in CI environments
    } else {
        console.log('\n🎉 All project configurations passed successfully!');
        process.exit(0);
    }
}

runAllConfiguredProjects().catch(error => {
    console.error('\n🛑 An unexpected error occurred during the test run orchestration:', error);
    process.exit(1);
});
