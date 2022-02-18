import { useState } from 'react'
import { VscLinkExternal, VscSymbolNumeric } from 'react-icons/vsc'
import { Light as SyntaxHighlighter } from 'react-syntax-highlighter'
import sh from 'react-syntax-highlighter/dist/cjs/languages/hljs/bash'
import codeStyle from 'react-syntax-highlighter/dist/cjs/styles/hljs/an-old-hope'

interface HeadingProps {
  text: string
  icon?: React.ReactChild
  href?: string
  id?: string
  iconAlwaysOn?: boolean
}
export const Heading = ({
  text,
  icon,
  href,
  id,
  iconAlwaysOn = false
}: HeadingProps) => {
  const [hover, setHover] = useState(false)
  return (
    <h3
      id={id}
      onMouseOver={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      className="relative flex items-center my-6 text-xl font-bold"
    >
      <a
        className={`absolute cursor-pointer text-blue-500 opacity-0 -left-5 ${
          hover || iconAlwaysOn ? 'opacity-100' : ''
        }`}
        href={href}
      >
        <i>{icon ?? <VscSymbolNumeric size={16} />}</i>
      </a>
      {text}
    </h3>
  )
}

export const SubHeading = ({
  text,
  icon,
  href,
  id,
  iconAlwaysOn = false
}: HeadingProps) => {
  const [hover, setHover] = useState(false)
  return (
    <h3
      id={id}
      onMouseOver={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      className="relative flex items-center my-4 text-base font-semibold"
    >
      <a
        className={`absolute cursor-pointer text-blue-500 opacity-0 -left-5 ${
          hover || iconAlwaysOn ? 'opacity-100' : ''
        }`}
        href={href}
      >
        <i>{icon ?? <VscSymbolNumeric size={12} />}</i>
      </a>
      {text}
    </h3>
  )
}

interface DocTextProps {
  text: string
}
export const DocText = ({ text }: DocTextProps) => {
  return <span className="text-sm">{text}</span>
}

interface CodeProps {
  lang?: string
  codeString: string
}
export const Code = ({ lang, codeString = '' }: CodeProps) => {
  SyntaxHighlighter.registerLanguage('bash', sh)
  return (
    <div className="relative w-72 sm:w-full my-1">
      <span className="absolute text-xs text-gray-400 top-2 right-2">
        {lang}
      </span>
      <SyntaxHighlighter
        className="rounded-md"
        language="bash"
        style={codeStyle}
        customStyle={{
          fontSize: '0.95em',
          lineHeight: '2.5em',
          background: '#374151'
        }}
        codeTagProps={{
          style: {
            lineHeight: 'inherit',
            fontSize: 'inherit',
            padding: '0 0.85em'
          }
        }}
      >
        {codeString}
      </SyntaxHighlighter>
    </div>
  )
}

interface DocLinkProps {
  text: string
  href: string
  icon?: React.ReactChild
}
export const DocLink = ({ text, href, icon }: DocLinkProps) => {
  return (
    <a
      className="flex inline-flex flex-row items-center text-sm text-blue-500"
      href={href}
    >
      {text}
      {icon ?? <VscLinkExternal className="ml-1" size={14} />}
    </a>
  )
}
