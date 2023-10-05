import React, { useEffect, useState } from "react";
import { notificationState } from "../../lib/notifications/notificationStates";

function NotificationCard({ type, message }) {
  const [currentStateColor, setColor] = useState("-white");
  useEffect(() => {
    switch (type) {
      case notificationState.ALERT:
        setColor("-yellow-400");
        break;
      case notificationState.ERROR:
        setColor("-red-700");
        break;
      case notificationState.SUCCESS:
        setColor("-green-700");
        break;
      case notificationState.WARNING:
        setColor("-orange-600");
        break;
      default:
        setColor("-gray-300");
        break;
    }
  }, [type, message]);

  return (
    <div
      className={`w-72 border${currentStateColor} bg${currentStateColor} rounded-md`}
    >
      <div className="p-2 text-black">
        <h2 className="font-bold">Notification</h2>
        <p>{message}</p>
      </div>
    </div>
  );
}

export default NotificationCard;
