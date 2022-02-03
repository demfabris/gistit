import { Layout, Snippet } from 'components'
import { useRouter } from 'next/router'
import { useEffect, useState } from 'react'

type GistitPayload = {
  hash: string
  author: string
  description: string
  timestamp: string
  inner: {
    name: string
    lang: string
    data: string
    size: number
  }
}

type ResponsePayload = {
  success: GistitPayload
  error: string
}

const SnippetPage = () => {
  const [gistit, setGistit] = useState<GistitPayload | null>(null)
  const [error, setError] = useState(false)
  const router = useRouter()
  const { hash } = router.query
  const url = process.env.SERVER_GET_URL as string

  useEffect(() => {
    if (hash)
      (async () => {
        const response = await fetch(url, {
          method: 'POST',
          body: JSON.stringify({
            hash
          })
        })

        if (
          response.status === 404 ||
          response.status === 500 ||
          response.status === 400
        ) {
          setError(true)
        }

        const gistit = (await response.json()) as ResponsePayload

        setGistit(gistit.success)
      })()
  }, [hash, url])

  return (
    <Layout withHeaderBar>
      <main className="flex flex-col w-full h-full mb-24">
        {error ? (
          <span className="flex items-center justify-center w-full mb-12 font-light">
            Gistit not found
          </span>
        ) : gistit ? (
          <>
            <h1 className="text-xl font-bold text-blue-500">{gistit.author}</h1>
            {gistit.description && (
              <span className="font-light text-gray-500">
                {gistit.description}
              </span>
            )}
            <Snippet
              name={gistit.inner.name}
              size={gistit.inner.size}
              code={Buffer.from(gistit.inner.data, 'base64').toString()}
              lang={gistit.inner.lang}
            />
          </>
        ) : (
          <>
            <div className="h-8 mb-4 bg-gray-300 dark:bg-gray-700 w-36 animate-pulse" />
            <div className="w-full h-8 mb-4 bg-gray-300 dark:bg-gray-700 animate-pulse" />
            <div className="w-full bg-gray-300 dark:bg-gray-700 h-96 animate-pulse"></div>
          </>
        )}
      </main>
    </Layout>
  )
}

export default SnippetPage
