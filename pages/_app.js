import MainNavigation from "@/components/layout/main-navigation";
import Notification from "../context/notification";

import "@/styles/globals.css";

export default function App({ Component, pageProps }) {
  return (
    <Notification>
      <MainNavigation>
        <Component {...pageProps} />
      </MainNavigation>
    </Notification>
  );
}
