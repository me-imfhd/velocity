import { auth } from "@repo/auth/server";
import { createSafeActionClient } from "next-safe-action";

export const authAction = createSafeActionClient({
  async middleware(parsedInput) {
    const user = await auth();
    if (!user?.id) {
      throw new Error("You are unauthorized, please log in.");
    }
    console.log(
      "HELLO FROM ACTION MIDDLEWARE, USER ID:",
      user.id,
      "PARSED INPUT:",
      parsedInput
    );

    return { userId: user.id, ...user };
  },
});
