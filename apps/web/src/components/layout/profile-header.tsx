import React from "react";
import { buttonVariants } from "@repo/ui/components";
import { UserProfileDropdown } from "./user-profile-dropdown";

export const ProfileHeader = async () => {
  const user = null;
  const initials = "";
  return (
    <>
      {user ? (
        <>
          <span>Go To Dashboard</span>
          <UserProfileDropdown
            user={user}
            initials={initials}
          ></UserProfileDropdown>
        </>
      ) : (
          <div
            className={buttonVariants({
              size: "sm",
            })}
          >
            Sign In
            <span className="sr-only">Sign In</span>
          </div>
      )}
    </>
  );
};
