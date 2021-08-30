const Discord = require('discord.js');
const webClient = require('../../services/WebClient.js');

// ??signup x, y
module.exports = {
	name: 'womsync',
	description: 'Sync Wise old man group to our service',
	args: false,
	usage:'',
	execute(message, args) {
		const server = message.guild.id;

		webClient.axiosInstance.get('api/clan/wom/sync/' + server)
			.then(response => {
				const success = new Discord.MessageEmbed()
					.setColor('#1a6ba1')
					.setTitle('Successfully Synced Wise Old Man!')
					.setTimestamp();
				message.channel.send(success);
			})
			.catch(errorFromCall => {
				message.channel.send(errorFromCall.response.data.message);
			});

	},
};
