export default function PageButton({ action, left = false }) {
  return (
    <div
      onClick={action}
      className={`w-[20px] px-2 py-1 font-semibold text-gray-900 bg-gradient-to-l from-black to-white shadow-sm grow-0 ${
        left ? "bg-gradient-to-r" : "bg-gradient-to-l"
      }`}
    />
  );
}
