import Image from "next/image";
import Link from "next/link";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/tauri";

export default function BookCover({ book }) {
  const router = useRouter();
  const [coverUrl, setCoverUrl] = useState();

  useEffect(() => {
    setCoverUrl(convertFileSrc(book.cover_location));
  }, []);

  return (
    <Link
      href={`${router.pathname}/${book.title}`}
      className="h-[500px] w-[300px]
	  overflow-clip rounded-lg border-2 border-black bg-white px-3 pb-8 pt-5
	  text-black transition duration-500 ease-in-out hover:border-white hover:bg-black hover:text-white"
    >
      <div className="flex h-full flex-col justify-between">
        {coverUrl ? (
          <div className="flex h-4/5 max-h-fit max-w-fit grow justify-center overflow-hidden">
            <Image
              className="rounded"
              alt={book.title}
              width={300}
              quality={100}
              object-fit="cover"
              height={500}
              src={coverUrl}
              onError={(e) => {
                e.target.src = "error.jpg";
              }}
            />
          </div>
        ) : (
          <></>
        )}

        <div className="h-1/5 max-w-xs self-start pt-2 text-base font-semibold">
          <span>{book.title}</span>
        </div>
      </div>
    </Link>
  );
}
