import { Layout, Snippet } from 'components'
import { useRouter } from 'next/router'
import { useEffect, useState } from 'react'
import protobuf from 'protobufjs'
import payload from '../../../public/payload.json'

export type GistitPayload = {
  hash: string
  author: string
  description: string
  timestamp: string
  inner: {
    name: string
    lang: string
    data: string
    size: number
  }[]
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
        const proto = await protobuf.Root.fromJSON(payload)
        const Gistit = proto.lookupType('gistit.payload.Gistit')
        const body = Gistit.encode({ hash }).finish()

        const response = await fetch(url, {
          method: 'POST',
          body: JSON.stringify(Buffer.from(body))
        })

        if (
          response.status === 404 ||
          response.status === 500 ||
          response.status === 400
        ) {
          setError(true)
        }

        const buffer = await response.arrayBuffer()
        const gistit = Gistit.decode(
          new Uint8Array(buffer)
        ) as unknown as GistitPayload

        setGistit(gistit)
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
              name={gistit.inner[0].name}
              size={gistit.inner[0].size}
              code={gistit.inner[0].data}
              lang={gistit.inner[0].lang}
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
