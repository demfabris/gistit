import { useState } from "react";
import { Header } from "components/Header";
import { Sidebar } from "components/Sidebar";

interface Props {
  children: React.ReactChild;
  withHeaderBar: boolean;
}
export const Layout = ({ children, withHeaderBar }: Props) => {
  const sidebarHandler = useState(false);

  return (
    <section className="flex justify-center h-full">
      <div className="flex flex-col items-center mx-6 w-full justify-center md:w-4/5 xl:w-3/5">
        <Header withHeaderBar={withHeaderBar} sidebarHandler={sidebarHandler} />
        <Sidebar sidebarHandler={sidebarHandler} />
        {children}
      </div>
    </section>
  );
};
