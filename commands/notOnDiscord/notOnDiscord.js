const axios = require('axios').default;

module.exports = {
	name: 'notondiscord',
	description: 'See who is not on discord vs in game clan list',
	args: true,
	usage: '<Wiseoldman Group Id>',
	execute(message, args) {

		let justUsernamesFromDiscord;
		message.guild.members.fetch()
			.then(members => {
				justUsernamesFromDiscord = members.filter(x => !x.user.bot).map(x => {
					if(x.nickname == null) {
						return x.user.username.toLowerCase();
					}
					return x.nickname.toLowerCase();
				});
				axios.get(`https://api.wiseoldman.net/groups/${args[0]}/members`)
					.then(response => {

						const membersFromWom = response.data;
						const justUserNamesFromWom = membersFromWom.map(x => x.username);
						const notInDiscord = justUserNamesFromWom.filter(x => !justUsernamesFromDiscord.includes(x.toLowerCase())).sort((a, b) => a.localeCompare(b))
						;
						const csvString = notInDiscord.join('\n');
						for(let i = 0; i < csvString.length; i += 2000) {
							const toSend = csvString.substring(i, Math.min(csvString.length, i + 2000));
							message.channel.send(toSend, { spilt: true });
						}

					})
					.catch(errorFromCall => {
						message.channel.send('There was an error with calling Wise old man. Maybe check group id.');
						message.channel.send(errorFromCall.response.data.message);
					});
			})
			.catch(console.error);
	},
};

