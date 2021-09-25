import { useState } from 'react'
import { Header, Sidebar, Footer } from 'components'

interface Props {
  children: React.ReactChild
  withHeaderBar: boolean
}
export const Layout = ({ children, withHeaderBar }: Props) => {
  const sidebarHandler = useState(false)

  return (
    <section className="flex justify-center h-full">
      <div className="flex flex-col items-center mx-6 justify-center w-layout">
        <Header withHeaderBar={withHeaderBar} sidebarHandler={sidebarHandler} />
        <Sidebar sidebarHandler={sidebarHandler} />
        {children}
        <Footer />
      </div>
    </section>
  )
}
