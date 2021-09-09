const Discord = require('discord.js');
const webClient = require('../../services/WebClient.js');
const prefix = process.env.PREFIX;


const helpEmbed = new Discord.MessageEmbed()
	.setColor('#1a6ba1')
	.setTitle('Sign up your clan with Runelite!')
	.setDescription(prefix + 'donations command help')
	.addFields(
		{ name: 'add', value: prefix + 'donations add, <donation type>, <player>, <amount>' },
		{ name: 'list all', value:  prefix + 'donations list, all' },
		{ name: 'list user', value:  prefix + 'donations list, user, <player>' },
		{ name: 'list type', value:  prefix + 'donations list, type, <donation type>' },
		{ name: 'list top', value: prefix + 'donations list, top, <donation type>' },
		{ name: 'add type', value: prefix + 'donations add, type, <new donation type>' },
		{ name: 'remove type', value: prefix + 'donations remove, type, <donation type to remove>' },
	)
	.setTimestamp();

function sendHelp(message) {
	message.channel.send(helpEmbed);
}

// ??signup x, y
module.exports = {
	name: 'donations',
	description: 'Handles donation calls',
	args: true,
	usage:'listtypes',
	execute(message, args) {
		if(args[0] === '' || args[0] === 'help') {
			// Make help embed vvv
			sendHelp(message);
		}
		else {
			const trimmedArgs = args.map(arg => arg.trim());


			if(trimmedArgs[0] === 'list' && trimmedArgs[1] === 'all') {
				listAllCommand(message);
				return;
			}

			if(trimmedArgs[0] === 'list' && trimmedArgs[1] === 'user') {
				listUser(args, message);
				return;
			}

			if(trimmedArgs[0] === 'list' && trimmedArgs[1] === 'type') {
				listDonationType(args, message);
				return;
			}

			// TODO !!donastions add, type, donation type name
			if(trimmedArgs[0] === 'add' && trimmedArgs[1] === 'type') {
				addDonationType(args, message);
				return;
			}

			if(trimmedArgs[0] === 'remove' && trimmedArgs[1] === 'type') {
				removeDonationType(args, message);
				return;
			}

			if(trimmedArgs[0] === 'add') {
				addCommand(args, message);
				return;
			}

			if(trimmedArgs[0] === 'list' && trimmedArgs[1] === 'top') {
				listTopDonators(args, message);
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

function listDonationType(args, message) {

	const webRequest = {
		type: 'donationType',
		lookupId: args[2],
	};

	const errorMessagesCleanNames = {
		type: 'Lookup type is missing',
		lookupId: 'Donation type is missing',
	};

	const error = handleErrors(webRequest, errorMessagesCleanNames, message);
	if(error) {
		return;
	}
	webClient.axiosInstance.post('api/donations/list', webRequest, { headers: setAuthHeader(message) })
		.then(response => {
			const success = new Discord.MessageEmbed()
				.setColor('#1a6ba1')
				.setTitle(`Successfully added ${webRequest.amount} to ${ response.data.name}`)
				.addFields(
					{
						name: 'Grand Total Donated For ' + response.data.name,
						value: response.data.total,
					},
				);
			success.setTimestamp();
			message.channel.send(success);
		})
		.catch(errorFromCall => {
			message.channel.send(errorFromCall.response.data.message);
		});
}

function listUser(args, message) {

	const webRequest = {
		type: 'user',
		lookupId: args[2],
	};

	const errorMessagesCleanNames = {
		type: 'Lookup type is missing',
		lookupId: 'Username is missing',
	};

	handleErrors(webRequest, errorMessagesCleanNames, message);

	webClient.axiosInstance.post('api/donations/list', webRequest, { headers: setAuthHeader(message) })
		.then(response => {
			const success = new Discord.MessageEmbed()
				.setColor('#1a6ba1')
				.setTitle(`Successfully added ${webRequest.amount} to ${ response.data.name}`)
				.addFields(
					{
						name: 'Grand Total Donated For ' + response.data.name,
						value: response.data.total,
					},
				);
			success.setTimestamp();
			message.channel.send(success);
		})
		.catch(errorFromCall => {
			message.channel.send(errorFromCall.response.data.message);
		});
}


function listAllCommand(message) {
	const webRequest = {
		type: 'all',
	};
	webClient.axiosInstance.post('api/donations/list', webRequest, { headers: setAuthHeader(message) })
		.then(response => {
			const success = new Discord.MessageEmbed()
				.setColor('#1a6ba1')
				.setTitle('Total donated')
				.addFields(
					{
						name: 'Grand Total Donated',
						value: response.data.grandTotal,
					},
				);
			response.data.donationTypes.forEach(x => {
				success.addFields({
					name: x.name,
					value: x.formattedAmount,
				});
			});
			success.setTimestamp();
			message.channel.send(success);
		})
		.catch(errorFromCall => {
			message.channel.send(errorFromCall.response.data.message);
		});
}

function addCommand(args, message) {

	const webRequest = {
		donationType: args[1],
		username: args[2],
		amount: args[3],
	};
	const errorMessagesCleanNames = {
		donationType: 'Donation type is missing',
		username: 'Username is missing',
		amount: 'Amount is missing',
	};

	const error = handleErrors(webRequest, errorMessagesCleanNames, message);
	if (error) {
		return;
	}
	webClient.axiosInstance.post('api/donations/add/donation', webRequest, { headers: setAuthHeader(message) })
		.then(response => {
			const success = new Discord.MessageEmbed()
				.setColor('#1a6ba1')
				.setTitle(`Successfully added ${webRequest.amount} to ${webRequest.username}`)
				.addFields(
					{
						name: 'Total donated',
						value: response.data.total,
					},
				)
				.setTimestamp();
			message.channel.send(success);
		})
		.catch(errorFromCall => {
			message.channel.send(errorFromCall.response.data.message);
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
function addDonationType(args, message) {

	const webRequest = {
		name: args[2],
	};

	const errorMessagesCleanNames = {
		name: 'New donation type is missing',
	};

	const error = handleErrors(webRequest, errorMessagesCleanNames, message);
	if (error) {
		return;
	}

	webClient.axiosInstance.post('api/donations/add/type', webRequest, { headers: setAuthHeader(message) })
		.then(response => {
			const success = new Discord.MessageEmbed()
				.setColor('#1a6ba1')
				.setTitle(`${response.data.name} successfully added as a donation type!`)
				.setTimestamp();
			message.channel.send(success);
		})
		.catch(errorFromCall => {
			message.channel.send(errorFromCall.response.data.message);
		});

}

function removeDonationType(args, message) {
	const webRequest = {
		name: args[2],
	};

	const errorMessagesCleanNames = {
		name: 'Donation type target is missing!',
	};

	const error = handleErrors(webRequest, errorMessagesCleanNames, message);
	if (error) {
		return;
	}
	const headers = setAuthHeader(message);

	webClient.axiosInstance.delete('api/donations/remove/type', { data: webRequest, headers })
		.then(response => {
			const success = new Discord.MessageEmbed()
				.setColor('#1a6ba1')
				.setTitle(`${response.data.name} successfully removed from donation type list!`)
				.setTimestamp();
			message.channel.send(success);
		})
		.catch(errorFromCall => {
			message.channel.send(errorFromCall.response.data.message);
		});
}

function listTopDonators(args, message) {
	const webRequest = {
		name: args[2],
	};

	const errorMessagesCleanNames = {
		name: 'Donation type is missing!',
	};

	const error = handleErrors(webRequest, errorMessagesCleanNames, message);
	if (error) {
		return;
	}

	const headers = setAuthHeader(message);

	webClient.axiosInstance.post('api/donations/list/topDonators', webRequest, { headers })
		.then(response => {
			const parsedResponse = response.data.map((donator, index) => {
				return { name: `${index + 1}. ${donator.name}`, value: donator.total };
			});
			const success = new Discord.MessageEmbed()
				.setColor('#1a6ba1')
				.setTitle(`${args[2]} current donation rankings!`)
				.addFields(parsedResponse)
				.setTimestamp();
			message.channel.send(success);
		})
		.catch(errorFromCall => {
			message.channel.send(errorFromCall.response.data.message);
		});
}