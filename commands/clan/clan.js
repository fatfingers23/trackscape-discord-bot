const Discord = require('discord.js');
const webClient = require('../../services/WebClient.js');
const prefix = process.env.PREFIX;


const helpEmbed = new Discord.MessageEmbed()
	.setColor('#1a6ba1')
	.setTitle('Clan management')
	.setDescription(prefix + 'clan command help')
	.addFields(
		{ name: 'inactive', value: prefix + 'clan inactive, <how daus back>' },
	)
	.setTimestamp();

function sendHelp(message) {
	message.channel.send(helpEmbed);
}

// ??signup x, y
module.exports = {
	name: 'clan',
	description: 'Handles clan management',
	args: true,
	usage:'help',
	execute(message, args) {
		if(args[0] === '' || args[0] === 'help') {
			// Make help embed vvv
			sendHelp(message);
		}
		else {
			const trimmedArgs = args.map(arg => arg.trim());


			if(trimmedArgs[0] === 'inactive') {
				showInactiveClanMates(message, trimmedArgs[1]);
				return;
			}
			sendHelp(message);
		}
	},
};


function showInactiveClanMates(message, days) {

	webClient.axiosInstance.get('api/player/inactive/' + days, { headers: setAuthHeader(message) })
		.then(response => {
			let messageToSend = `Players who have not been on for ${days} days \nDate format is mm-dd-yyyy \n \n`;
			response.data.reverse().forEach(x => {
				messageToSend += `${x.username}: ${x['last_active']} \n`;
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
