const { PHASE_DEVELOPMENT_SERVER } = require('next/constants')

/** @type {import('next').NextConfig} */
module.exports = (phase, { defaultConfig }) => {
  let config = { ...defaultConfig, reactStrictMode: true }

  if (phase === PHASE_DEVELOPMENT_SERVER) {
    config.env.GITHUB_OAUTH_URL =
      'http://localhost:4001/gistit-base/us-central1/auth'
    config.env.SERVER_GET_URL =
      'http://localhost:4001/gistit-base/us-central1/get'
  } else {
    config.env.GITHUB_OAUTH_URL =
      'https://us-central1-gistit-base.cloudfunctions.net/auth'
    config.env.SERVER_GET_URL =
      'https://us-central1-gistit-base.cloudfunctions.net/get'
  }

  return config
}
