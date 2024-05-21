"use server";
import { db } from "@repo/db";
import { IdType, idSchema, throwTRPCError } from "../common";

export const getComputers = async () => {
  try {
    const c = await db.computer.findMany();
    const totc = await db.computer.count();
    return { computers: c, totalComputer: totc };
  } catch (err) {
    return throwTRPCError(err);
  }
};
export type GetComputerReturns = Awaited<ReturnType<typeof getComputers>>;

export const getComputerById = async (id: IdType) => {
  idSchema.parse(id);
  try {
    const c = await db.computer.findFirst({ where: { id } });
    return { computer: c };
  } catch (err) {
    return throwTRPCError(err);
  }
};

export const getUserComputers = async (userId: IdType) => {
  idSchema.parse(userId);
  try {
    const c = await db.computer.findMany({ where: { userId } });
    return { computers: c };
  } catch (err) {
    return throwTRPCError(err);
  }
};
