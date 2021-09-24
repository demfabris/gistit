import { useState } from "react";
import { Header } from "components/Header";
import { Sidebar } from "components/Sidebar";
import { Footer } from "./Footer";

interface Props {
  children: React.ReactChild;
  withHeaderBar: boolean;
}
export const Layout = ({ children, withHeaderBar }: Props) => {
  const sidebarHandler = useState(false);

  return (
    <section className="flex justify-center h-full">
      <div className="flex flex-col items-center mx-6 w-full justify-center md:w-4/6 xl:w-3/6">
        <Header withHeaderBar={withHeaderBar} sidebarHandler={sidebarHandler} />
        <Sidebar sidebarHandler={sidebarHandler} />
        {children}
        <Footer />
      </div>
    </section>
  );
};
