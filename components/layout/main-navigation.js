import { Cog8ToothIcon, HomeIcon, StarIcon } from "@heroicons/react/24/outline";
import Link from "next/link";
const navigation = [
	{ name: "Books", href: "/books", icon: HomeIcon },
	{ name: "Settings", href: "/settings", icon: Cog8ToothIcon },
	{ name: "Favourites", href: "#", icon: StarIcon },
];

export default function MainNavigation({ children = null }) {
	return (
		<>
			<div>
				<div className="fixed inset-y-0 z-50 flex flex-col w-16 duration-500 ease-in-out transition-width hover:w-60">
					<div className="flex flex-col text-transparent transition-colors duration-200 ease-in-out bg-white rounded-r-lg hover:text-black hover:duration-700 grow gap-y-5">
						<nav className="flex flex-col flex-1 pt-8 navbar-icon">
						<ul className="" role="list">
								{navigation.map((item) => (
								<li key={item.name} className="flex items-center p-2 ">
								<div className="relative w-16"> {/* Wrapper element with relative positioning */}
									<item.icon
									className="absolute inset-0 w-5.5 h-6 m-auto ml-3  text-black transition-width duration-600"
									aria-hidden="true"
									/>
								</div>
								<Link
								href={item.href}
									className="w-full mx-auto font-semibold leading-6 text-center ease-out duration-250"
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
			</div>
		</>
	);
}
