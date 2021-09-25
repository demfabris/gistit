import { useColorMode } from 'hooks'
import { Dispatch, SetStateAction, useState } from 'react'
import {
  VscChromeClose,
  VscColorMode,
  VscDesktopDownload,
  VscGithub,
  VscLibrary,
  VscLightbulb,
  VscSearch,
  VscTerminal
} from 'react-icons/vsc'

interface Props {
  sidebarHandler: [boolean, Dispatch<SetStateAction<boolean>>]
}
export const Sidebar = ({ sidebarHandler }: Props) => {
  const [state, setState] = sidebarHandler
  const toggle = useColorMode()

  return (
    <>
      <div
        className={`fixed flex right-0 top-0 w-full bg-gray-900 h-full z-10 transition-all ${
          state ? 'opacity-50' : 'pointer-events-none opacity-0'
        }`}
      />
      <div
        className={`fixed right-0 top-0 w-72 h-full border-l-2 border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 z-20 transition transform ${
          state ? '' : 'translate-x-full'
        }`}
      >
        <button
          className="absolute right-6 top-9"
          onClick={() => setState((state) => !state)}
        >
          <VscChromeClose size={24} />
        </button>
        <nav className="w-full pt-24">
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
  )
}

interface NavigationProps {
  text: String
  icon: React.ReactElement
  href?: string
  callback?: () => void
}
const Navigation = ({ text, icon, href, callback }: NavigationProps) => {
  return (
    <div
      className="border-b-2 border-gray-200 dark:border-gray-700 last:border-b-0 hover:text-blue-500"
      onClick={() => callback?.call(globalThis)}
    >
      <a
        href={href}
        className="flex items-center h-16 pl-8 text-sm font-semibold cursor-pointer"
      >
        <i className="pr-4">{icon}</i>
        {text}
      </a>
    </div>
  )
}

interface NavigationDropDownProps {
  text: string
  icon: React.ReactElement
}
const NavigationDropDown = ({ text, icon }: NavigationDropDownProps) => {
  const [drop, setDrop] = useState(false)

  return (
    <div className="flex flex-col border-b-2 border-gray-200 dark:border-gray-700">
      <div
        onClick={() => setDrop((state) => !state)}
        className={`cursor-pointer pl-8 text-sm h-16 flex items-center font-semibold hover:text-blue-500 ${
          drop ? 'text-blue-500' : ''
        }`}
      >
        <i className="pr-4">{icon}</i>
        {text}
      </div>
      <ul className={`${drop ? 'flex flex-col h-full' : 'hidden'}`}>
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
  )
}

interface SubNavigationProps {
  text: string
  href: string
  icon?: React.ReactElement
}
const SubNavigation = ({ text, href, icon }: SubNavigationProps) => {
  return (
    <li className="flex items-center w-full h-12 pl-16 text-xs font-semibold cursor-pointer hover:text-blue-500">
      <a href={href} className="flex items-center">
        <i className="pr-4">{icon}</i>
        {text}
      </a>
    </li>
  )
}
