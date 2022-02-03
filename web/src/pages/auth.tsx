import type { NextPage } from 'next'
import { useRouter } from 'next/router'
import { Layout, Spinner } from 'components'
import { useEffect, useState } from 'react'

interface RequiredParams {
  code: string
  state: string
  [rest: string]: string
}

const Auth: NextPage = () => {
  const [statusComponent, setStatusComponent] = useState(<Pending />)
  const router = useRouter()
  const { code, state } = router.query as RequiredParams
  const url = process.env.GITHUB_OAUTH_URL as string

  useEffect(() => {
    if (code && state)
      (async () => {
        const response = await fetch(url, {
          method: 'POST',
          body: JSON.stringify({
            code,
            state
          })
        })

        if (
          response.status === 404 ||
          response.status === 500 ||
          response.status === 400
        ) {
          setStatusComponent(<Expired />)
        } else {
          setStatusComponent(<Success />)
        }
      }).call(this)
  }, [code, url, state])

  return <Layout withHeaderBar>{statusComponent}</Layout>
}

const Pending = () => (
  <>
    <Spinner />
    <span className="font-thin">Authenticating</span>
  </>
)

const Success = () => (
  <>
    <div className="h-[40px] w-[40px] my-[50px]" />
    <span className="font-thin">
      <b>Success!</b> You can close this window now
    </span>
  </>
)

const Expired = () => (
  <>
    <div className="h-[40px] w-[40px] my-[50px]" />
    <span className="font-thin">This token is invalid or expired</span>
  </>
)

export default Auth
