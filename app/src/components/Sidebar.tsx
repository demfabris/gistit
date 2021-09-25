import { useColorMode } from "hooks";
import { Dispatch, SetStateAction, useState } from "react";
import {
  VscChromeClose,
  VscColorMode,
  VscDesktopDownload,
  VscGithub,
  VscLibrary,
  VscLightbulb,
  VscSearch,
  VscTerminal,
} from "react-icons/vsc";

interface Props {
  sidebarHandler: [boolean, Dispatch<SetStateAction<boolean>>];
}
export const Sidebar = ({ sidebarHandler }: Props) => {
  const [state, setState] = sidebarHandler;
  const toggle = useColorMode();

  return (
    <>
      <div
        className={`fixed flex right-0 top-0 w-full bg-black h-full z-10 transition-all ${
          state ? "opacity-30" : "pointer-events-none opacity-0"
        }`}
      />
      <div
        className={`fixed right-0 top-0 w-4/6 h-full shadow-2xl bg-white dark:bg-gray-900 z-20 transition transform ${
          state ? "" : "translate-x-full"
        }`}
      >
        <button
          className="absolute right-6 top-9"
          onClick={() => setState((state) => !state)}
        >
          <VscChromeClose size={24} />
        </button>
        <nav className="pt-24 w-full">
          <Navigation
            text="Search"
            href="/search"
            icon={<VscSearch size={20} />}
          />
          <NavigationDropDown
            text="Documentation"
            icon={<VscLibrary size={20} />}
          />
          <Navigation
            text="Github"
            icon={<VscGithub size={20} />}
            href="https://github.com/fabricio7p"
          />
          <Navigation
            text="Mode"
            icon={<VscColorMode size={20} />}
            callback={toggle!!}
          />
        </nav>
      </div>
    </>
  );
};

interface NavigationProps {
  text: String;
  icon: React.ReactElement;
  href?: string;
  callback?: () => void;
}
const Navigation = ({ text, icon, href, callback }: NavigationProps) => {
  return (
    <div onClick={() => callback?.call(globalThis)}>
      <a
        href={href}
        className="h-16 cursor-pointer pl-8 text-sm font-semibold flex items-center border-l-4 border-transparent
        hover:border-blue-500"
      >
        <i className="pr-4">{icon}</i>
        {text}
      </a>
    </div>
  );
};

interface NavigationDropDownProps {
  text: string;
  icon: React.ReactElement;
}
const NavigationDropDown = ({ text, icon }: NavigationDropDownProps) => {
  const [drop, setDrop] = useState(false);

  return (
    <div className="flex flex-col">
      <div
        onClick={() => setDrop((state) => !state)}
        className={`cursor-pointer pl-8 text-sm h-16 flex items-center font-semibold border-l-4 border-transparent
          hover:border-blue-500 ${drop ? "border-blue-500" : ""}`}
      >
        <i className="pr-4">{icon}</i>
        {text}
      </div>
      <ul className={`${drop ? "flex flex-col h-full" : "hidden"}`}>
        <SubNavigation
          text="Installation"
          href="#"
          icon={<VscDesktopDownload size={18} />}
        />
        <SubNavigation
          text="Features"
          href="#"
          icon={<VscLightbulb size={18} />}
        />
        <SubNavigation text="CLI" href="#" icon={<VscTerminal size={18} />} />
      </ul>
    </div>
  );
};

interface SubNavigationProps {
  text: string;
  href: string;
  icon?: React.ReactElement;
}
const SubNavigation = ({ text, href, icon }: SubNavigationProps) => {
  return (
    <li className="cursor-pointer flex font-semibold items-center pl-16 w-full h-12 text-xs border-blue-200 border-l-4 hover:border-blue-500">
      <a href={href} className="flex items-center">
        <i className="pr-4">{icon}</i>
        {text}
      </a>
    </li>
  );
};
