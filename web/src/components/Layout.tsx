import { useState } from 'react'
import { Header, Sidebar, Footer } from 'components'

interface Props {
  children: React.ReactChild
  withHeaderBar: boolean
}
export const Layout = ({ children, withHeaderBar }: Props) => {
  const sidebarHandler = useState(false)

  return (
    <section className="flex flex-col items-center justify-center min-h-screen">
      <Header withHeaderBar={withHeaderBar} sidebarHandler={sidebarHandler} />
      <div className="flex flex-col items-center w-full px-6 md:px-0 md:w-layout">
        <Sidebar sidebarHandler={sidebarHandler} />
        {children}
      </div>
      <Footer />
    </section>
  )
}
