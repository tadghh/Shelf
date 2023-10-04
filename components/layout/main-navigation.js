/* eslint-disable no-mixed-spaces-and-tabs */
import { Cog8ToothIcon, HomeIcon, StarIcon } from "@heroicons/react/24/outline";
import Link from "next/link";

const navigation = [
  { name: "Books", href: "/dashboard", icon: HomeIcon },
  { name: "Settings", href: "/settings", icon: Cog8ToothIcon },
];

export default function MainNavigation({ children = null }) {
  return (
    <>
      <div className="transition-width fixed inset-y-0 z-50 flex w-16 flex-col duration-500 ease-in-out hover:w-60">
        <div className="flex grow flex-col gap-y-5 rounded-r-lg bg-white text-transparent transition-colors duration-200 ease-in-out hover:text-black hover:duration-700">
          <nav className="navbar-icon flex flex-1 flex-col pt-8">
            <ul className="" role="list">
              {navigation.map((item) => (
                <li key={item.name} className="flex items-center p-2 ">
                  <div className="relative w-16">
                    <item.icon
                      className="w-5.5 transition-width duration-600 absolute inset-0 m-auto  ml-3 h-6 text-black"
                      aria-hidden="true"
                    />
                  </div>
                  <Link
                    href={item.href}
                    className="duration-250 mx-auto w-full text-center font-semibold leading-6 ease-out"
                  >
                    {item.name}
                  </Link>
                </li>
              ))}
            </ul>
          </nav>
        </div>
      </div>

      <main className="bg-black">{children}</main>
    </>
  );
}
