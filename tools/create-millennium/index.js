#!/usr/bin/env node

const cli = require('@pyke/millennium-cli');
const arguments = process.argv.slice(2);
if (arguments.length > 0 && !arguments[0].startsWith('--'))
	arguments.unshift('-d');

cli.run([ 'init' ].concat(arguments), 'millennium').catch(err => {
	console.log(`Error running CLI: ${err.message}`);
	process.exit(1);
});
