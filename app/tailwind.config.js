module.exports = {
  mode: 'jit',
  purge: {
    content: [
      './src/pages/**/*.{js,ts,jsx,tsx}',
      './src/components/**/*.{js,ts,jsx,tsx}'
    ],
    options: {
      safelist: ['dark']
    }
  },
  darkMode: 'class',
  theme: {
    extend: {
      width: {
        layout: '900px'
      }
    }
  },
  variants: {
    extend: {
      borderWidth: ['last'],
      backgroundColor: ['dark'],
      textColor: ['dark']
    }
  },
  plugins: []
}
