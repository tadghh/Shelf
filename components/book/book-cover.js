import Image from "next/image";
import Link from "next/link";

export default function BookCover({ title, coverPath }) {
  const bookLink = `./dashboard/${title}`;
  return (
    <Link
      href={bookLink}
      className=" duration-500 h-[500px]
		 w-[300px] border-2 hover:border-white border-black rounded-lg bg-white px-3 pb-8 pt-5
		  text-black  transition ease-in-out hover:bg-black hover:text-white  overflow-clip"
    >
      <div className="flex flex-col justify-between h-full ">
        <div className="flex justify-center overflow-hidden grow max-w-fit h-4/5 max-h-fit ">
          <Image
            className="rounded"
            alt={title}
            width={300}
            quality={100}
            object-fit="cover"
            height={500}
            src={coverPath}
          />
        </div>

        <div className="self-start max-w-xs pt-2 text-base font-semibold h-1/5">
          <span>{title}</span>
        </div>
      </div>
    </Link>
  );
}
