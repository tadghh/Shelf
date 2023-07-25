import Image from "next/image";
import Link from "next/link";



export default function BookCover({ title, coverPath }) {
	const bookLink = `./books/${title}`;
	return (
		<div className=" duration-350 h-[500px] w-[300px] rounded-lg bg-white px-3 pb-8 pt-5 text-black shadow-xl ring-1 ring-gray-900/5 transition ease-in-out hover:bg-black hover:text-white ">
			<div className="flex flex-col justify-between h-full transition ease-in-out border-black duration-320 hover:border-1">
				<div className="flex justify-center overflow-hidden grow max-w-fit h-4/5 max-h-fit ">
					<Image
						className=""
						alt={title}
						width={300}
						quality={100}
						object-fit="cover"
						height={500}
						src={coverPath}
					/>
				</div>

				<div className="self-start max-w-xs pt-2 text-base font-semibold h-1/5 ">
					<Link className="text-lg" href={bookLink}>{title}</Link>
				</div>
			</div>
		</div>
	);
}
