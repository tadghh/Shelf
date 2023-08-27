export default function BookItem({ children, bookRender }) {
  return (
    <>
      <div
        onClick={() => bookRender.prev()}
        className="w-[20px] h-auto px-2 text-xs font-semibold bg-gradient-to-r from-black to-white text-gray-900 shadow-sm grow-0 "
      />

      {children}
      <div
        onClick={() => {
bookRender.next(); console.log("uu");
}}
        className="px-2 py-1 w-[20px] z-50 text-xs font-semibold text-gray-900 bg-gradient-to-l from-black to-white shadow-sm grow-0 "
      />
    </>
  );
}
