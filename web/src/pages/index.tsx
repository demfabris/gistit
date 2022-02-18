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
        <a href="#prerequisites">
          <button className="flex flex-row items-center px-8 text-lg font-medium text-white bg-blue-500 rounded-full h-14 shadow-sm">
            Get started
            <i>
              <VscRocket size="20" className="ml-3" />
            </i>
          </button>
        </a>
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
            <h2 className="mb-3 text-xl font-semibold">Peer to peer</h2>
            <span className="font-thin leading-7">
              Gistit enables peer to peer file sharing via <b>libp2p</b>. The
              network stack behind <b>IPFS</b>
            </span>
          </li>
        </ul>
        <video
          src="/recording.mp4"
          autoPlay={true}
          loop={true}
          muted={true}
          width="768px"
          className="my-20 bg-gray-200 border-gray-700 rounded-lg dark:bg-gray-700 border-[1px]"
        />
        <article className="flex flex-col w-full mb-24 text-left">
          <Heading
            text="Prerequisites"
            id="prerequisites"
            href="#prerequisites"
          />
          <SubHeading text="Clipboard" />
          <div className="pl-6">
            <DocText text="For clipboard features make sure you have a working clipboard backend. (e.g. 'xclip', 'wl-copy' on Linux. 'pbcopy' on MacOs)" />
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
            <DocText text="With cargo: " />
            <Code codeString="cargo install gistit" lang="sh" />
          </div>
          <SubHeading text="Cli binary" />
          <div className="pl-6">
            <DocText text="Get the latest build for your system here: " />
            <DocLink
              text="Github"
              href="https://github.com/fabricio7p/gistit/releases"
            />
          </div>
          <Heading text="Basic usage" id="basic-usage" href="#basic-usage" />
          <SubHeading text="Sending" />
          <div className="pl-6">
            <div className="mb-4">
              <DocText text="Send a gistit: " />
              <Code
                codeString="gistit send file.txt [-c] [--author AUTHOR] [--description DESCRIPTION]"
                lang="sh"
              />
            </div>
            <div className="mb-4">
              <DocText text="Send stdin: " />
              <Code codeString="ls | gistit" lang="sh" />
            </div>
            <div className="mb-4">
              <DocText text="Post to " />
              <DocLink text="Github Gist" href="https://gist.github.com/" />
              <Code codeString="gistit file.txt --github" lang="sh" />
            </div>
          </div>
          <SubHeading text="Fetching" />
          <div className="pl-6">
            <div className="mb-4">
              <DocText text="Fetching a gistit: " />
              <Code codeString="gistit f [HASH] [--save]" lang="sh" />
            </div>
          </div>
        </article>
      </>
    </Layout>
  )
}

export default Home
