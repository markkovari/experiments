// cucumber.js
module.exports = {
    default: {
        paths: ['src/features/**/*.feature'],
        requireModule: ['ts-node/register'],
        require: ['src/step-definitions/**/*.ts', 'src/support/**/*.ts'],
        format: ['summary', 'progress-bar'],
        formatOptions: { snippetInterface: 'async-await' },
        publishQuiet: true,
        worldParameters: {
            // You can pass global parameters to your world here
        }
    }
};
