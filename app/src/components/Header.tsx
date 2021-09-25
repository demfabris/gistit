import { useColorMode } from "common/ColorMode";
import { Dispatch, Fragment, SetStateAction, useRef, useState } from "react";
import {
  VscMenu,
  VscGithub,
  VscSearch,
  VscColorMode,
  VscLibrary,
} from "react-icons/vsc";

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
        <Navigation />
        <Hamburguer onClick={() => setSidebar((state) => !state)} />
      </div>
    </header>
  ) : (
    <Fragment />
  );
};

const Logo = () => {
  return (
    <h1 className="font-bold text-xl text-fg h-full flex items-center justify-center mr-2">
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

const Navigation = () => {
  const toggle = useColorMode();
  return (
    <nav className="hidden justify-end items-center md:flex">
      <Search />
      <NavigationButton
        text="Github"
        icon={<VscGithub size={18} className="mr-2" />}
        href="www.github.com/fabricio7p/gistit.git"
      />
      <NavigationButton
        text="Docs"
        icon={<VscLibrary size={18} className="mr-2" />}
        href="/docs"
      />
      <NavigationButton
        text="Color Mode"
        icon={<VscColorMode size={18} className="mr-2" />}
        callback={toggle!!}
      />
    </nav>
  );
};

interface NavigationButtonProps {
  text: string;
  icon: React.ReactChild;
  href?: string;
  callback?: () => void | null;
}
const NavigationButton = ({
  text,
  icon,
  href,
  callback,
}: NavigationButtonProps) => {
  return (
    <div
      onClick={() => callback?.call(globalThis)}
      className="text-sm px-3 md:px-6 flex justify-center h-8 items-center font-medium
        border-2 border-transparent"
    >
      <a
        href={href}
        className="flex items-center h-full cursor-pointer border-b-2 border-transparent 
          hover:border-blue-500"
      >
        {icon}
        {text}
      </a>
    </div>
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
    <div className="px-3">
      <span
        onFocus={handleOpen}
        onBlur={handleClose}
        tabIndex={0}
        className="flex items-center h-9 border-2 rounded-full border-blue-500 px-6 
        cursor-pointer text-sm text-blue-500 font-bold"
      >
        <VscSearch size={18} className="mr-3" />
        Find
        <input
          type="text"
          className={`bg-transparent placeholder-gray-500 outline-none transition-all z-10
            transform-gpu text-gray-700 font-medium w-0 ${
              focus ? "pl-3 w-44" : "w-0"
            }`}
          ref={innerRef}
          placeholder="Hash, title or author..."
        />
      </span>
    </div>
  );
};
