import React, { useEffect, useState } from "react";
import { notificationState } from "../../lib/notifications/notificationStates";

function NotificationCard({ type, message }) {
  const [currentStateColor, setColor] = useState("white");
  const [backgroundColor, setBackgroundColor] = useState(`bg-white`);
  const [borderColor, setBorderColor] = useState(`border-${currentStateColor}`);
  useEffect(() => {
    switch (type) {
      case notificationState.ALERT:
        setColor("border-yellow-400");
        break;
      case notificationState.ERROR:
        setColor("border-red-700");
        break;
      case notificationState.SUCCESS:
        setColor("border-green-700");
        break;
      case notificationState.WARNING:
        setColor("border-orange-600");
        break;
      default:
        setColor("border-gray-300");
        break;
    }
  }, [type, message]);

  return (
    <div className={`w-72 ${currentStateColor} rounded-md border-2 bg-white `}>
      <div className="p-2 text-black">
        <h2 className="font-bold">Notification</h2>
        <p>{message}</p>
      </div>
    </div>
  );
}

export default NotificationCard;
