import Link from "next/link";
export default function NoDirectory(){
    return (  <div className="flex justify-center min-h-screen ml-20 ">
    <div className="flex items-center ">
      <div className="flex flex-col justify-around text-black bg-white rounded-md h-4/5 w-80 ">
        <div className="flex flex-col items-center justify-center ">
          <div className="px-4 py-4 m-2 text-center bg-gray-400 rounded-md">
            Please configure your book directory:
          </div>
          <div className="flex flex-col items-center justify-center ">
            <Link
              className="flex items-center justify-center w-20 h-12 underline bg-white rounded-md text-black-500"
              href="/settings"
            >
              Here
            </Link>
          </div>
        </div>
        <div className="flex items-center justify-center pb-5 m-2 border-b-2 border-white " />
        {/* <div className="flex items-center justify-center pb-5 ">
          <div className="flex items-center justify-center border-4 border-white border-dashed rounded-md text-black-500 h-52 w-52">
            Drop a .epub file here
          </div>
        </div> */}
      </div>
    </div>
  </div>);
}