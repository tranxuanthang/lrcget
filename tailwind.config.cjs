/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/**/*.{html,js,vue}',
  ],
  theme: {
    extend: {
      colors: {
        hoa: {
          "0": "#ffdbe4",
          "100": "#ffd9e2",
          "200": "#ffd4df",
          "300": "#ffcdda",
          "400": "#ffc3d3",
          "500": "#ffb7cb",
          "600": "#fea7c1",
          "700": "#fe96b7",
          "800": "#fc84ad",
          "900": "#fb73a4",
          "1000": "#f8639d",
          "1100": "#f35697",
          "1200": "#eb4a91",
          "1300": "#dc4089",
          "1400": "#c5367d",
          "1500": "#a82c6c"
        },
        brave: {
          100: "#ffffff",
          99: "#fffbff",
          98: "#fff8f8",
          95: "#ffecf0",
          90: "#ffd9e2",
          80: "#ffb1c8",
          70: "#ff84af",
          60: "#ff4896",
          50: "#df247d",
          40: "#b90063",
          35: "#a30057",
          30: "#8e004b",
          25: "#79003f",
          20: "#650033",
          15: "#510028",
          10: "#3e001d",
          5: "#2b0012",
          1: "#0f0004",
          0: "#000000",
          primary: "#b90063",
          'background-dark': "#0f0004",
          'background-modal-dark': "#140000"
        }
      }
    },
  },
  darkMode: 'class',
}
