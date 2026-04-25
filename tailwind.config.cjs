/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./src/**/*.{html,js,vue}'],
  theme: {
    extend: {
      colors: {
        hoa: {
          0: '#ffdbe4',
          100: '#ffd9e2',
          200: '#ffd4df',
          300: '#ffcdda',
          400: '#ffc3d3',
          500: '#ffb7cb',
          600: '#fea7c1',
          700: '#fe96b7',
          800: '#fc84ad',
          900: '#fb73a4',
          1000: '#f8639d',
          1100: '#f35697',
          1200: '#eb4a91',
          1300: '#dc4089',
          1400: '#c5367d',
          1500: '#a82c6c',
        },
      },
    },
  },
  darkMode: 'class',
}
