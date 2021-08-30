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

			if(trimmedArgs[0] === 'add') {
				addCommand(args, message);
				return;
			}

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

			sendHelp(message);
		}
	},
};

function handleErrors(values, errorObj, message) {
	let error = false;
	Object.keys(values).forEach(key => {
		if (values[key] === undefined) {
			if (errorObj[key] !== undefined) {
				message.channel.send(`${errorObj[key]} was not provided`);
			}
			else {
				message.channel.send('Sorry! Something went wrong!');
			}
			error = true;
		}
	});

	if (error) {
		return;
	}
}

function listDonationType(args, message) {
	const server = message.guild.id;
	const user = message.author.id;

	const webRequest = {
		type: 'donationType',
		lookupId: args[2],
	};

	const errorMessagesCleanNames = {
		type: 'Lookup type is missing',
		lookupId: 'Donation type is missing',
	};

	handleErrors(webRequest, errorMessagesCleanNames, message);

	webClient.axiosInstance.post(`api/donations/list/${server}/${user}`, webRequest)
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
	const server = message.guild.id;
	const user = message.author.id;

	const webRequest = {
		type: 'user',
		lookupId: args[2],
	};

	const errorMessagesCleanNames = {
		type: 'Lookup type is missing',
		lookupId: 'Username is missing',
	};

	handleErrors(webRequest, errorMessagesCleanNames, message);

	webClient.axiosInstance.post(`api/donations/list/${server}/${user}`, webRequest)
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
	const server = message.guild.id;
	const user = message.author.id;
	const webRequest = {
		type: 'all',
	};
	webClient.axiosInstance.post(`api/donations/list/${server}/${user}`, webRequest)
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

	const server = message.guild.id;
	const user = message.author.id;

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


	let error = false;
	Object.keys(webRequest).forEach(key => {
		if (webRequest[key] === undefined) {
			if (errorMessagesCleanNames[key] !== undefined) {
				message.channel.send(`${errorMessagesCleanNames[key]} was not provided`);
			}
			else {
				message.channel.send('Sorry! Something went wrong!');
			}
			error = true;
		}
	});
	if (error) {
		return;
	}
	webClient.axiosInstance.post(`api/donations/add/gold/${server}/${user}`, webRequest)
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
