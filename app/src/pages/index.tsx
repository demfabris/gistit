import type { NextPage } from "next";

const Home: NextPage = () => {
  return (
    <section>
      <header>
        <div className="pl-16 pr-16 h-16 w-full flex border-b-2 bg-gray-100 items-center">
          <h1 className="font-bold text-xl pl-10 pr-10 h-full bg-blue-500 flex items-center justify-center">
            Gistit
          </h1>
          <ul className="flex justify-center items-center">
            <li className="">link</li>
          </ul>
        </div>
      </header>
    </section>
  );
};

export default Home;
