/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './crates/app/src/**/*.{rs,html,leptos}',
    './assets/**/*.{html,css}',
  ],
  theme: {
    extend: {
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
      },
    },
  },
  plugins: [],
};
