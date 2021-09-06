const Discord = require('discord.js');
const webClient = require('../../services/WebClient.js');
const prefix = process.env.PREFIX;


const helpEmbed = new Discord.MessageEmbed()
	.setColor('#1a6ba1')
	.setTitle('View in game chat logs! ')
	.setDescription(prefix + 'chat log command help')
	.addFields(
		{ name: 'list', value: prefix + 'donations list, <how many messages>' },
	)
	.setTimestamp();

function sendHelp(message) {
	message.channel.send(helpEmbed);
}

// ??signup x, y
module.exports = {
	name: 'chatlog',
	description: 'Handles chat logs',
	args: true,
	usage:'list, <count>',
	execute(message, args) {
		if(args[0] === '' || args[0] === 'help') {
			// Make help embed vvv
			sendHelp(message);
		}
		else {
			const trimmedArgs = args.map(arg => arg.trim());


			if(trimmedArgs[0] === 'list') {
				listMessages(message, trimmedArgs[1]);
				return;
			}
			sendHelp(message);
		}
	},
};

function handleErrors(values, errorObj, message) {
	Object.keys(values).forEach(key => {
		if (values[key] === undefined) {
			if (errorObj[key] !== undefined) {
				message.channel.send(`${errorObj[key]} was not provided`);
			}
			else {
				message.channel.send('Sorry! Something went wrong!');
			}
			return true;
		}
	});
	return false;
}


function listMessages(message, count) {

	webClient.axiosInstance.get('api/clan/chatlog/' + count, { headers: setAuthHeader(message) })
		.then(response => {
			let messageToSend = '';
			response.data.reverse().forEach(x => {
				messageToSend += `${x.time_sent} ${x.sender}: ${x.message} \n`;
			});


			for(let i = 0; i < messageToSend.length; i += 1994) {
				let toSend = messageToSend.substring(i, Math.min(messageToSend.length, i + 1994));
				toSend = '```' + toSend;
				toSend += '```';
				message.channel.send(toSend, { spilt: true });
			}

		})
		.catch(errorFromCall => {
			message.channel.send(errorFromCall.message);
		});
}


function setAuthHeader(message) {
	const server = message.guild.id;
	const user = message.author.id;

	return {
		'userdiscordid': user,
		'discordserverid' : server,
	};
}
