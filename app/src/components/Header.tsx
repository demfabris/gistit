import { useColorMode } from 'hooks'
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
    <header className="flex justify-center w-full mb-24 md:mb-36">
      <div className="flex items-center justify-between w-full h-24">
        <Logo />
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
  return (
    <nav className="items-center justify-end hidden md:flex">
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
  )
}

interface NavigationButtonProps {
  text: string
  icon: React.ReactChild
  href?: string
  callback?: () => void | null
}
const NavigationButton = ({
  text,
  icon,
  href,
  callback
}: NavigationButtonProps) => {
  return (
    <div
      onClick={() => callback?.call(globalThis)}
      className="flex items-center justify-center h-8 px-2 text-sm font-medium border-2 border-transparent md:px-4"
    >
      <a
        href={href}
        className="flex items-center h-full border-b-2 border-transparent cursor-pointer hover:border-blue-500"
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
        className="flex items-center h-10 px-6 text-sm font-bold text-blue-500 border-2 border-blue-500 rounded-full cursor-pointer"
      >
        <VscSearch size={18} className="mr-3" />
        Find
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
