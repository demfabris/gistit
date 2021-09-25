import { Layout } from 'components'
import { useRouter } from 'next/router'
import Code from 'react-syntax-highlighter'

const Gist = () => {
  const router = useRouter()
  const { id } = router.query
  const gistContent = "const appearGist = ()=>{ 'here is the gist'}"

  return (
    <Layout withHeaderBar>
      <div className="border-solid border-2 p-8 rounded-md w-4/5">
        <p className="mb-8">Gist Hash: {id}</p>
        <Code language="javascript" className="rounded-lg">
          {gistContent}
        </Code>
      </div>
    </Layout>
  )
}

export default Gist
