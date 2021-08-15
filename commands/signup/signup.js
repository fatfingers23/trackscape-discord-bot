const Discord = require('discord.js');
const webClient = require('../../services/WebClient.js');

const errorMessagesCleanNames = {
	name: 'Clan Name',
	runescapeUserName: 'Runescape username',
};


const helpEmbed = new Discord.MessageEmbed()
	.setColor('#1a6ba1')
	.setTitle('Sign up your clan with Runelite!')
	.setDescription('??signup command help')
	.addFields(
		{ name: '\u200b', value: '\u200b' },
		{ name: 'Example usage', value: '??signup <clan name>, <clan leader>' },
		{ name: '\u200b', value: '\u200b' },
		{ value: 'We will handle the rest!' },
	)
	.setImage('https://i.imgur.com/dHamuVH.jpeg')
	.setTimestamp();

function sendHelp(message) {
	message.channel.send(helpEmbed);
}

// ??signup x, y
module.exports = {
	name: 'signup',
	description: 'sign up to start tracking',
	args: true,
	usage:'<clan Name>, <clan Leader>',
	execute(message, args) {
		if(args[0] === '' || args[0] === 'help') {
			// Make help embed vvv
			sendHelp(message);
		}
		else {
			const trimmedArgs = args.map(arg => arg.trim());

			const server = message.guild.id;
			const channel = message.channel.id;
			const user = message.author.id;

			const webRequest = {
				name: trimmedArgs[0],
				discordId: server,
				discordIdOfCreator: user,
				runescapeUserName: trimmedArgs[1],
			};

			let error = false;
			Object.keys(webRequest).forEach(key => {
				if(webRequest[key] === undefined) {
					if(errorMessagesCleanNames[key] !== undefined) {
						message.channel.send(`${errorMessagesCleanNames[key]} was not provided`);
					}
					else{
						message.channel.send('Sorry! Something went wrong!');
					}
					error = true;
				}
			});
			if(error) {
				return;
			}
			webClient.axiosInstance.post('api/clan/signup', webRequest)
				.then(response => {
					const success = new Discord.MessageEmbed()
						.setColor('#1a6ba1')
						.setTitle(`Successfully signed up the clan ${webRequest.name}`)
						.setDescription(`Link to set in Clanmate Export Plugin: ${response.data.link}`)
						.addFields(
							{ name: 'Runelite Plugin link', value: 'https://runelite.net/plugin-hub/show/clanmate-export' },
						)
						.setTimestamp();
					message.channel.send(success);
				})
				.catch(errorFromCall => {
					message.channel.send(errorFromCall.response.data.message);
				});
		}
	},
};
