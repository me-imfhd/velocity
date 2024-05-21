import { PrismaClient } from "@prisma/client";

declare global {
  // allow global `var` declarations
  // eslint-disable-next-line no-var
  var db: PrismaClient | undefined;
}

export const db =
  global.db ||
  new PrismaClient({
    errorFormat: "pretty",
    log: ["error"],
  });

if (process.env.NODE_ENV !== "production") global.db = db;
export * from "./prisma/zod";
export {z} from "zod"
