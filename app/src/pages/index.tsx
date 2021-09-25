import type { NextPage } from 'next'
import { Layout } from 'components'
import Code from 'react-syntax-highlighter'

const Home: NextPage = () => {
  const codeString = 'const aux = "hello world"'
  return (
    <Layout withHeaderBar>
      <>
        <p className="font-black text-7xl">
          Gistit<b className="text-blue-500">.</b>
        </p>
        <span className="mt-4 mb-12 text-xl font-thin text-center">
          Quick and easy <b className="font-bold">anonymous</b> code snippet
          sharing.
        </span>
        <button className="px-8 text-lg font-bold text-white bg-blue-500 rounded-full h-14 shadow-sm">
          Get started
        </button>
        <div className="flex w-full mt-20 border-b-2 border-gray-200 dark:border-gray-700"></div>
        <ul className="flex flex-col justify-between w-full px-6 mt-12 md:flex-row md:px-0 gap-x-16 gap-y-10">
          <li className="flex flex-col">
            <h2 className="text-xl font-semibold">Practical</h2>
            <span className="font-thin text-md">
              Easy to use cli tool to quickly upload any code snippet to a
              temporary web page.
            </span>
          </li>
          <li className="flex flex-col">
            <h2 className="text-xl font-semibold">Practical</h2>
            <span className="font-thin text-md">
              Easy to use cli tool to quickly upload any code snippet to a
              temporary web page.
            </span>
          </li>
          <li className="flex flex-col">
            <h2 className="text-xl font-semibold">Practical</h2>
            <span className="font-thin text-md">
              Easy to use cli tool to quickly upload any code snippet to a
              temporary web page.
            </span>
          </li>
        </ul>
        <video
          src="#"
          width="768px"
          height="481px"
          className="my-20 bg-gray-200 dark:bg-gray-700"
        />
        <Code language="javascript" className="rounded-lg">
          {codeString}
        </Code>
      </>
    </Layout>
  )
}

export default Home
