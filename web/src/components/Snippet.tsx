import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter'
import { useColorStore } from 'hooks'
import { coldarkDark } from 'styles/coldark-dark'
import { coldarkCold } from 'styles/coldark-cold'

interface Props {
  code: string
  lang: string
  name: string
  size: number
}

function formatBytes(bytes: number, decimals: number) {
  if (bytes == 0) return '0 Bytes'
  var k = 1024,
    dm = decimals || 2,
    sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'],
    i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i]
}

export const Snippet = ({ code, lang, name, size }: Props) => {
  const isDarkMode = useColorStore((state) => state.dark)
  console.log(isDarkMode)
  return (
    <div className="relative flex min-w-full">
      <span className="absolute text-xs text-gray-400 top-4 right-4">
        {lang}
      </span>
      <div className="absolute top-4 left-4">
        <span className="text-sm font-bold text-blue-500 ">{name}</span>
        <span className="ml-4 text-sm text-gray-500">
          {formatBytes(size, 2)}
        </span>
      </div>
      <SyntaxHighlighter
        className="min-w-full border-gray-300 dark:border-gray-700 rounded-md border-[1px]"
        language={lang}
        style={isDarkMode ? coldarkDark : coldarkCold}
        showLineNumbers
        showInlineLineNumbers
        lineNumberContainerStyle
        wrapLongLines
        customStyle={{
          fontSize: '0.85em'
        }}
        codeTagProps={{
          style: {
            lineHeight: 'inherit',
            fontSize: 'inherit',
            padding: '0.5em 0.85em 0 0.85em'
          }
        }}
      >
        {code}
      </SyntaxHighlighter>
    </div>
  )
}
