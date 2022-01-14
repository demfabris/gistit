import type { NextPage } from 'next'
import { Heading, Layout, SubHeading, Code } from 'components'
import { DocLink, DocText } from 'components/Docs'
import { VscRocket } from 'react-icons/vsc'

const Home: NextPage = () => {
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
        <button className="flex flex-row items-center px-8 text-lg font-medium text-white bg-blue-500 rounded-full h-14 shadow-sm">
          Get started
          <i>
            <VscRocket size="20" className="ml-3" />
          </i>
        </button>
        <div className="flex w-full mt-20 border-b-2 border-gray-200 dark:border-gray-700"></div>
        <ul className="flex flex-col justify-between w-full px-6 mt-16 md:flex-row md:px-0 gap-x-14 gap-y-10">
          <li className="flex flex-col w-full">
            <h2 className="mb-3 text-xl font-semibold">Practical</h2>
            <span className="font-thin leading-7">
              Easy to use <b>cli</b> tool to quickly share a code snippet or a
              couple of files, with syntax highlighting!
            </span>
          </li>
          <li className="flex flex-col w-full">
            <h2 className="mb-3 text-xl font-semibold">Terminal Support</h2>
            <span className="font-thin leading-7">
              Shared snippets can be accessed via web page or by entering a
              unique <b>hash</b> directly in your terminal.
            </span>
          </li>
          <li className="flex flex-col w-full">
            <h2 className="mb-3 text-xl font-semibold">Rust-Powered</h2>
            <span className="font-thin leading-7">
              Fast, safe, and lightweight <b>cli</b> application to just run,
              share and move on with your life.
            </span>
          </li>
        </ul>
        <video
          src="#"
          width="768px"
          height="481px"
          className="my-20 bg-gray-200 dark:bg-gray-700"
        />
        <article className="flex flex-col w-full text-left">
          <Heading
            text="Prerequisites"
            id="prerequisites"
            href="#prerequisites"
          />
          <SubHeading text="Clipboard" />
          <div className="pl-6">
            <DocText text="For clipboard features make sure you have a working clipboard backend." />
          </div>
          <SubHeading text="Terminal colors" />
          <div className="pl-6">
            <DocText text="To get syntax highlighting when fetching snippets from the terminal make sure you have 256-bit colored terminal. " />
            <DocLink text="Term colors" href="#" />
          </div>
          <Heading text="Installation" id="installation" href="#installation" />
          Gistit is avaiable in most distribution channels.
          <SubHeading text="Quick Install" />
          <div className="pl-6">
            <DocText text="With shell : " />
            <Code
              codeString="sh -c '$(curl -fsSL https://gistit.io/install.sh)'"
              lang="sh"
            />
          </div>
          <SubHeading text="Cli binary" />
          <div className="pl-6">
            <DocText text="Get the latest build for your system here : " />
            <DocLink text="Github" href="#" />
          </div>
        </article>
      </>
    </Layout>
  )
}

export default Home
