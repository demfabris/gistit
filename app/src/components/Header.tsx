import { Dispatch, Fragment, SetStateAction, useRef, useState } from "react";
import { VscMenu, VscGithub, VscSearch } from "react-icons/vsc";

interface HeaderProps {
  withHeaderBar: boolean;
  sidebarHandler: [boolean, Dispatch<SetStateAction<boolean>>];
}
export const Header = ({ withHeaderBar, sidebarHandler }: HeaderProps) => {
  const [, setSidebar] = sidebarHandler;

  return withHeaderBar ? (
    <header className="mb-24 flex justify-center w-full">
      <div className="w-full h-24 flex items-center justify-between">
        <Logo />
        <Links />
        <Hamburguer onClick={() => setSidebar((state) => !state)} />
      </div>
    </header>
  ) : (
    <Fragment />
  );
};

const Logo = () => {
  return (
    <h1
      className="font-bold text-xl text-fg h-full flex items-center justify-center mr-8
      text-gray-700"
    >
      Gistit<b className="text-blue-500">.</b>
    </h1>
  );
};

const Hamburguer = ({ ...rest }) => {
  return (
    <button className="md:hidden" {...rest}>
      <VscMenu size={24} />
    </button>
  );
};

const Links = () => {
  return (
    <ul className="hidden justify-end items-center md:flex">
      <Search />
      <li
        className="text-sm px-3 md:px-6 flex justify-center h-8 items-center font-medium
        border-2 border-transparent"
      >
        <a
          className="flex items-center h-full cursor-pointer border-b-2 border-transparent 
          hover:border-blue-500"
        >
          <VscGithub size={18} className="mr-2" />
          Github
        </a>
      </li>
      <li
        className="text-sm pl-3 md:pl-6 flex justify-center h-8 items-center font-medium 
        border-2 border-transparent"
      >
        <a
          className="flex items-center h-full cursor-pointer border-b-2 border-transparent 
          hover:border-blue-500"
        >
          Documentation
        </a>
      </li>
    </ul>
  );
};

const Search = () => {
  const [focus, setFocus] = useState(false);
  const innerRef = useRef<HTMLInputElement>(null!);

  function handleOpen() {
    setFocus(true);
    innerRef.current.focus();
  }

  function handleClose() {
    setFocus(false);
  }

  return (
    <li className="px-3">
      <span
        onFocus={handleOpen}
        onBlur={handleClose}
        tabIndex={0}
        className="flex items-center h-9 border-2 rounded-full border-blue-500 px-6 
        cursor-pointer text-sm text-blue-500 font-bold"
      >
        <VscSearch size={18} className="mr-3" />
        Search
        <input
          type="text"
          className={`bg-transparent placeholder-gray-500 outline-none transition-all z-10
            transform-gpu text-gray-700 font-medium w-0 ${
              focus ? "pl-3 w-56" : "w-0"
            }`}
          ref={innerRef}
          placeholder="Hash, title or author..."
        />
      </span>
    </li>
  );
};
