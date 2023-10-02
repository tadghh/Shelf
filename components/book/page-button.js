export default function PageButton({ action, left = false }) {
  return (
    <div
      onClick={action}
      className={`w-[20px] grow-0  animate-fade  bg-gradient-to-l from-transparent  to-white px-2 py-1 font-semibold text-gray-900 shadow-sm transition-opacity duration-75 ease-in ${
        left ? "bg-gradient-to-r" : "bg-gradient-to-l"
      }`}
    />
  );
}
