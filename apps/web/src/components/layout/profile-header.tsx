import React from "react";
import Link from "next/link";
import { buttonVariants } from "@repo/ui/components";
import { UserProfileDropdown } from "./user-profile-dropdown";
import { authOptions, getServerSession } from "@repo/auth/server";

export const ProfileHeader = async () => {
  const session = await getServerSession(authOptions);
  const user = session?.user;
  const initials = `${user?.name?.charAt(0) ?? ""}`;
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
        <Link href={"/sign-in"}>
          <div
            className={buttonVariants({
              size: "sm",
            })}
          >
            Sign In
            <span className="sr-only">Sign In</span>
          </div>
        </Link>
      )}
    </>
  );
};
