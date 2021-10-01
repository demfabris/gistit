import { Layout, Snippet } from 'components'
import { useRouter } from 'next/router'
import { useEffect } from 'react'

const SnippetPage = () => {
  const router = useRouter()
  const { hash } = router.query

  useEffect(() => {
    console.log(hash)
  })

  return (
    <Layout withHeaderBar>
      <main className="flex flex-col w-full h-full">
        <Snippet code="const aux = 'hello'" lang="javascript" />
      </main>
    </Layout>
  )
}

export default SnippetPage
