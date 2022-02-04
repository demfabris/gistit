import { useState } from 'react'
import Head from 'next/head'
import { Header, Sidebar, Footer } from 'components'

import type { FC } from 'react'

interface Props {
  withHeaderBar: boolean
  title: string
}

export const Layout: FC<Props> = ({
  children,
  withHeaderBar,
  title = 'Gistit'
}) => {
  const sidebarHandler = useState(false)

  return (
    <div>
      <Head>
        <title>{title}</title>
      </Head>
      <section className="flex flex-col items-center justify-center min-h-screen">
        <Header withHeaderBar={withHeaderBar} sidebarHandler={sidebarHandler} />
        <div className="flex flex-col items-center w-full px-6 md:px-0 md:w-layout">
          <Sidebar sidebarHandler={sidebarHandler} />
          {children}
        </div>
        <Footer />
      </section>
    </div>
  )
}
