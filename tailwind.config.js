/* eslint-disable no-unused-vars */
/* eslint-disable no-mixed-spaces-and-tabs */
/** @type {import('tailwindcss').Config} */
module.exports = {
	content: [
		"./pages/**/*.{js,ts,jsx,tsx,mdx}",
		"./components/**/*.{js,ts,jsx,tsx,mdx}",
		"./app/**/*.{js,ts,jsx,tsx,mdx}",
	],
	theme: {
		extend: {
			animation: {
				fade: 'fadeOut .5s ease-in-out',
			  },

			  // that is actual animation
			  keyframes: theme => ({
				fadeOut: {
				  '0%': { opacity: 0 },
				  '100%': { opacity: 100 },
				},
			  }),
			backgroundImage: {
				"gradient-radial": "radial-gradient(var(--tw-gradient-stops))",
				"gradient-conic":
					"conic-gradient(from 180deg at 50% 50%, var(--tw-gradient-stops))",
			},
		},
	},

};
