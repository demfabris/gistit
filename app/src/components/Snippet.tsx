import SyntaxHighlighter from 'react-syntax-highlighter'
import codeStyle from 'react-syntax-highlighter/dist/cjs/styles/hljs/an-old-hope'

interface Props {
  code: string
  lang: string
}
export const Snippet = ({ code, lang }: Props) => {
  return (
    <div className="relative flex min-w-full">
      <span className="absolute text-xs text-gray-400 top-2 right-2">
        {lang}
      </span>
      <SyntaxHighlighter
        className="min-w-full rounded-md"
        language={lang}
        style={codeStyle}
        showLineNumbers
        customStyle={{
          fontSize: '0.95em',
          lineHeight: '3em',
          background: '#374151'
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
