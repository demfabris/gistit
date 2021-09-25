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
      <div className="flex flex-col items-center mx-6 w-full justify-center md:w-5/6 xl:w-4/6 2xl:w-3/5 xl:px-14">
        <Header withHeaderBar={withHeaderBar} sidebarHandler={sidebarHandler} />
        <Sidebar sidebarHandler={sidebarHandler} />
        {children}
        <Footer />
      </div>
    </section>
  )
}
