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

  return (
    <>
      <div
        className={`fixed flex right-0 top-0 w-full h-full z-10 bg-black transition-all ${
          state ? "opacity-30" : "pointer-events-none opacity-0"
        }`}
      />
      <aside
        className={`fixed right-0 top-0 w-4/6 bg-white h-full shadow-2xl z-20 transition transform ${
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
          <Link text="Search" href="/search" icon={<VscSearch size={20} />} />
          <LinkDropDown text="Documentation" icon={<VscLibrary size={20} />} />
          <Link
            text="Github"
            href="https://github.com/fabricio7p"
            icon={<VscGithub size={20} />}
          />
          <Link
            text="Mode"
            href="https://github.com/fabricio7p"
            icon={<VscColorMode size={20} />}
          />
        </nav>
      </aside>
    </>
  );
};

interface LinkProps {
  text: String;
  icon: React.ReactElement;
  href: string;
}
const Link = ({ text, icon, href }: LinkProps) => {
  return (
    <div>
      <a
        href={href}
        className="h-16 cursor-pointer pl-8 text-sm font-semibold text-gray-700 flex items-center border-l-4 border-transparent
        hover:border-blue-500"
      >
        <i className="pr-4">{icon}</i>
        {text}
      </a>
    </div>
  );
};

interface LinkDropDownProps {
  text: string;
  icon: React.ReactElement;
}
const LinkDropDown = ({ text, icon }: LinkDropDownProps) => {
  const [drop, setDrop] = useState(false);

  return (
    <div className="flex flex-col">
      <div
        onClick={() => setDrop((state) => !state)}
        className={`cursor-pointer pl-8 text-sm h-16 flex items-center font-semibold text-gray-700 border-l-4 border-transparent
          hover:border-blue-500 ${drop ? "border-blue-500" : ""}`}
      >
        <i className="pr-4">{icon}</i>
        {text}
      </div>
      <ul className={`${drop ? "flex flex-col h-full" : "hidden"}`}>
        <SubLink
          text="Installation"
          href="#"
          icon={<VscDesktopDownload size={18} />}
        />
        <SubLink text="Features" href="#" icon={<VscLightbulb size={18} />} />
        <SubLink text="CLI" href="#" icon={<VscTerminal size={18} />} />
      </ul>
    </div>
  );
};

interface SubLinkProps {
  text: string;
  href: string;
  icon?: React.ReactElement;
}
const SubLink = ({ text, href, icon }: SubLinkProps) => {
  return (
    <li className="cursor-pointer flex font-semibold items-center pl-16 text-gray-700 w-full h-12 text-xs border-blue-200 border-l-4 hover:border-blue-500">
      <a href={href} className="flex items-center">
        <i className="pr-4">{icon}</i>
        {text}
      </a>
    </li>
  );
};
