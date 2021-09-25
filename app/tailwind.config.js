const mode = process.env.JIT_COMPILE

module.exports = {
  mode: mode ? 'jit' : undefined,
  purge: {
    content: [
      './src/pages/**/*.{js,ts,jsx,tsx}',
      './src/components/**/*.{js,ts,jsx,tsx}'
    ],
    options: {
      safelist: ['dark']
    }
  },
  darkMode: 'class', // or 'media' or 'class'
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
