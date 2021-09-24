import { Dispatch, SetStateAction } from "react";
import { VscChromeClose } from "react-icons/vsc";

interface Props {
  sidebarHandler: [boolean, Dispatch<SetStateAction<boolean>>];
}
export const Sidebar = ({ sidebarHandler }: Props) => {
  const [state, setState] = sidebarHandler;

  function handleOpen(state: boolean): String {
    return state ? "translate-x-full" : "";
  }

  return (
    <div
      className={`fixed flex justify-end right-0 top-0 w-full h-full z-10 transition 
        transform-gpu ${handleOpen(state)}`}
    >
      <div className="p-6 w-4/6 bg-white h-full shadow-2xl">
        <button
          className="absolute right-6 top-9"
          onClick={() => setState((state) => !state)}
        >
          <VscChromeClose size={24} />
        </button>
      </div>
    </div>
  );
};
