import { useColorMode, useColorStore } from 'hooks'
import { Dispatch, Fragment, SetStateAction, useRef, useState } from 'react'
import {
  VscMenu,
  VscGithub,
  VscSearch,
  VscColorMode,
  VscLibrary
} from 'react-icons/vsc'

interface HeaderProps {
  withHeaderBar: boolean
  sidebarHandler: [boolean, Dispatch<SetStateAction<boolean>>]
}
export const Header = ({ withHeaderBar, sidebarHandler }: HeaderProps) => {
  const [, setSidebar] = sidebarHandler

  return withHeaderBar ? (
    <header className="flex justify-center w-full px-6 mb-24 border-b-2 border-gray-200 dark:border-gray-700">
      <div className="flex items-center justify-between w-full h-20">
        <a href="/">
          <Logo />
        </a>
        <Navigation />
        <Hamburguer onClick={() => setSidebar((state) => !state)} />
      </div>
    </header>
  ) : (
    <Fragment />
  )
}

const Logo = () => {
  return (
    <h1 className="flex items-center justify-center h-full mr-2 text-xl font-bold text-fg">
      Gistit<b className="text-blue-500">.</b>
    </h1>
  )
}

const Hamburguer = ({ ...rest }) => {
  return (
    <button className="md:hidden" {...rest}>
      <VscMenu size={24} className="text-gray-800 dark:text-white" />
    </button>
  )
}

const Navigation = () => {
  const toggle = useColorMode()
  const store = useColorStore((state) => state.toggleColorMode)

  return (
    <nav className="items-center justify-end hidden md:flex">
      <Search />
      <NavigationButton
        text="Github"
        icon={<VscGithub size={18} className="mr-2" />}
        href="https://github.com/fabricio7p/gistit.git"
        target="_blank"
      />
      <NavigationButton
        text="Docs"
        icon={<VscLibrary size={18} className="mr-2" />}
        href="/docs"
      />
      <NavigationButton
        text="Color Mode"
        icon={<VscColorMode size={18} className="mr-2" />}
        callback={() => {
          store()
          return toggle?.call(globalThis)
        }}
      />
    </nav>
  )
}

interface NavigationButtonProps {
  text: string
  icon: React.ReactChild
  href?: string
  callback?: () => any
  target?: string
}
const NavigationButton = ({
  text,
  icon,
  href,
  callback,
  target
}: NavigationButtonProps) => {
  return (
    <div
      onClick={() => callback?.call(globalThis)}
      className="flex items-center justify-center h-10 px-2 text-sm font-medium border-2 border-transparent md:px-4"
    >
      <a
        href={href}
        className="flex items-center h-full border-b-2 border-transparent cursor-pointer hover:border-blue-500"
        target={target}
      >
        {icon}
        {text}
      </a>
    </div>
  )
}

const Search = () => {
  const [focus, setFocus] = useState(false)
  const innerRef = useRef<HTMLInputElement>(null!)

  function handleOpen() {
    setFocus(true)
    innerRef.current.focus()
  }

  function handleClose() {
    setFocus(false)
  }

  return (
    <div className="px-3">
      <span
        onFocus={handleOpen}
        onBlur={handleClose}
        tabIndex={0}
        className={`flex items-center h-10 px-6 text-sm border-2 border-gray-200 dark:border-gray-700 rounded-full cursor-pointer ${
          focus &&
          'border-blue-500 dark:border-blue-500 text-blue-500 font-bold'
        }`}
      >
        <VscSearch size={18} className="mr-4" />
        Find snippet
        <input
          type="text"
          className={`bg-transparent outline-none transition-all z-10 text-gray-800 dark:text-white transform-gpu font-medium w-0 ${
            focus ? 'pl-3 w-44' : 'w-0'
          }`}
          ref={innerRef}
          placeholder="Hash, title or author..."
        />
      </span>
    </div>
  )
}
