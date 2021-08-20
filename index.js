const Discord = require('discord.js');
const { Intents } = require('discord.js');
const intents = new Intents([
	Intents.NON_PRIVILEGED, // include all non-privileged intents, would be better to specify which ones you actually need
	'GUILD_MEMBERS', // lets you request guild members
	'GUILD_PRESENCES',
]);

const client = new Discord.Client({ intents:intents });
const dotenv = require('dotenv');
const fs = require('fs');
dotenv.config();
const prefix = process.env.PREFIX;


client.commands = new Discord.Collection();
const commandFolders = fs.readdirSync('./commands');

for (const folder of commandFolders) {
	const commandFiles = fs.readdirSync(`./commands/${folder}`).filter(file => file.endsWith('.js'));
	for (const file of commandFiles) {
		const command = require(`./commands/${folder}/${file}`);
		client.commands.set(command.name, command);
	}
}


client.on('message', message => {
	if (!message.content.startsWith(prefix) || message.author.bot) return;

	const messageSplit = message.content.slice(prefix.length).trim().split(/ +/);
	const commandName = messageSplit.shift().toLowerCase();

	// HACK ¯\_(ツ)_/¯
	const justArgsString = message.content.split(' ').splice(1).join(' ');
	let args = justArgsString.split(',');

	if(args[0] === '') {
		args = [];
	}

	if (!client.commands.has(commandName)) return;


	const command = client.commands.get(commandName);

	if (command.args && !args.length) {
		let reply = `You didn't provide any arguments, ${message.author}!`;

		if(command.usage) {
			reply += `\nThe proper usage would be: \`${prefix}${command.name} ${command.usage}\``;
		}

		return message.channel.send(reply);
	}

	try {
		command.execute(message, args);
	}
	catch (error) {
		console.error(error);
		message.reply('there was an error trying to execute that command!');
	}
});


client.login(process.env.TOKEN);

