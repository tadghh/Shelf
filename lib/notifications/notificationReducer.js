import { notificationState } from "./notificationStates";

// Initial notification state. It's empty for now
export const notificationInitialState = {
  notifications: [],
};

// Our reducer function
export default function NotificationReducer(
  state = notificationInitialState,
  { type, payload },
) {
  switch (type) {
    case notificationState.ADD:
      // Add notification to the list (state..notifications)
      return { notifications: [...state.notifications, payload.notification] };
    case notificationState.DELETE:
      // Remove/Delete notification
      const deleteNotifcation = state.notifications?.filter(
        (notification) => notification.id !== payload.id,
      );
      return { notifications: [...deleteNotifcation] };
    case notificationState.INACTIVE:
      // Make notifcation inactive
      const notifications = state.notifications?.map((notification) => {
        if (notification.id === payload.id) {
          return {
            ...notification,
            active: false,
          };
        }
        return notification;
      });
      return { notifications: [...notifications] };
    default:
      return state;
  }
}
