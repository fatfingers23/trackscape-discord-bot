const axios = require('axios').default;

const configForAxios = {
	baseURL: process.env.API_URL,
	timeout: 100000,
	headers: {
		'Authorization': `Bearer ${process.env.API_TOKEN}`,
	},
};

const axiosInstance = axios.create(configForAxios);

module.exports = {
	axiosInstance,
};
