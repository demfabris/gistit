import type { NextPage } from "next";
import { Layout } from "components";
import Code from "react-syntax-highlighter";

const Home: NextPage = () => {
  const codeString = 'const aux = "hello world"';
  return (
    <Layout withHeaderBar>
      <>
        {/* <Image src="/logo.svg" alt="logo" width="300px" height="150px" /> */}
        <p className="text-7xl font-black">
          Gistit<b className="text-blue-500">.</b>
        </p>
        <span className="text-xl font-thin text-center mt-4 mb-12">
          Quick and easy <b className="font-bold">anonymous</b> code snippet
          sharing.
        </span>
        <button className="rounded-full bg-blue-500 text-white text-lg font-bold h-14 px-8 shadow-sm">
          Get started
        </button>
        <div className="w-full flex border-b-2 border-gray-200 dark:border-gray-700 mt-20"></div>
        <ul className="w-full flex flex-col px-6 md:flex-row md:px-0 justify-between mt-12 gap-x-16 gap-y-10">
          <li className="flex flex-col">
            <h2 className="font-semibold text-xl">Practical</h2>
            <span className="text-md font-thin">
              Easy to use cli tool to quickly upload any code snippet to a
              temporary web page.
            </span>
          </li>
          <li className="flex flex-col">
            <h2 className="font-semibold text-xl">Practical</h2>
            <span className="text-md font-thin">
              Easy to use cli tool to quickly upload any code snippet to a
              temporary web page.
            </span>
          </li>
          <li className="flex flex-col">
            <h2 className="font-semibold text-xl">Practical</h2>
            <span className="text-md font-thin">
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
  );
};

export default Home;
