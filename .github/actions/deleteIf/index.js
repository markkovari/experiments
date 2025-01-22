const core = require('@actions/core');
const github = require('@actions/github');

try {
    const nameToGreet = core.getInput('who-to-greet');
    console.log(`Hello ${nameToGreet}`);

    const time = new Date().toTimeString();
    core.setOutput('time', time); // Set output for later use

    const payload = JSON.stringify(github.context.payload, undefined, 2);
    console.log(`The event payload: ${payload}`);
} catch (error) {
    core.setFailed(error.message);
}