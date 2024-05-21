"use server";
import { z } from "zod";
import { authAction } from "../safe-action";
import { createComputerLimit } from "@repo/api/src/rate-limit";
import {
  createComputer,
  deleteUsersAllComputers,
  insertComputerParams,
} from "@repo/api/src/computers";
import { throwTRPCError } from "@repo/api/src/common";
export const createComputerAction = authAction(
  insertComputerParams.omit({ userId: true }),
  async (input, { userId }) => {
    const RL = await createComputerLimit.limit(userId);
    if (!RL.success) {
      return throwTRPCError("Rate Limit Exceeded, Try after few minutes.");
    }
    console.log("Remaining Limit: ", RL.remaining);
    return await createComputer({ ...input, userId });
  }
);
export const deleteUsersComputersAction = authAction(
  z.undefined(),
  async (_, { userId }) => {
    return await deleteUsersAllComputers(userId);
  }
);
