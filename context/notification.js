import React, {
  createContext,
  useCallback,
  useEffect,
  useReducer,
} from "react";
import { AnimatePresence, motion } from "framer-motion";

import NotificationCard from "../components/layout/notification";

import {
  notificationState,
  notificationInitialState,
  notificationReducer,
} from "../lib/notifications/notification";

export const NotificationContext = createContext();

function Notification({ children }) {
  const [state, dispatch] = useReducer(
    notificationReducer,
    notificationInitialState,
  );

  const deleteNotifcation = (id) => {
    dispatch({
      type: notificationState.DELETE,
      payload: {
        id: id,
      },
    });
  };

  const closeNotification = (id) => {
    dispatch({
      type: notificationState.INACTIVE,
      payload: {
        id: id,
      },
    });
    setTimeout(() => {
      deleteNotifcation(id);
    }, 1000);
  };

  const notify = (type, message) => {
    const notificationId = state.notifications.length;
    dispatch({
      type: notificationState.ADD,
      payload: {
        notification: {
          id: notificationId,
          type: type,
          message: message,
          active: true,
        },
      },
    });
    setTimeout(() => {
      closeNotification(notificationId);
    }, 6000);
    return notificationId;
  };

  const showNotifications = useCallback(
    () => (
      <>
        {state.notifications.map((notification) => (
          <AnimatePresence key={notification?.id}>
            {notification?.active && (
              <motion.div
                initial={{
                  opacity: 0,
                  scale: 0.8,
                  y: "10%",
                }}
                animate={{
                  opacity: 1,
                  scale: 1,
                  y: "0%",
                }}
                exit={{
                  opacity: 0,
                  scale: 0.8,
                  y: "10%",
                }}
              >
                <NotificationCard
                  type={notification?.type}
                  message={notification?.message}
                />
              </motion.div>
            )}
          </AnimatePresence>
        ))}
      </>
    ),
    [state],
  );

  useEffect(() => {
    state;
  }, [state]);

  const value = {
    notifications: state?.notifications,
    notify,
    closeNotification,
  };
  return (
    <>
      <NotificationContext.Provider value={value}>
        <div className="fixed left-0 top-0 z-50 flex h-fit w-full flex-col items-center justify-center gap-3 pt-10">
          {showNotifications()}
        </div>
        {children}
      </NotificationContext.Provider>
    </>
  );
}

export default Notification;
