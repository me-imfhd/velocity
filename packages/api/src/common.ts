import { TRPCError } from "@trpc/server";
import { z } from "zod";

export const idSchema = z.string().uuid();
export type IdType = z.infer<typeof idSchema>;

export const BUCKET_NAME = process.env.BUCKET_NAME!;
export const ACCESS_KEY_ID = process.env.ACCESS_KEY_ID!;
export const ACCESS_SECRET = process.env.ACCESS_SECRET!;
export const REGION = process.env.REGION!;
export const CLOUDFRONT_URL = process.env.CLOUDFRONT_URL!;
export const throwTRPCError = (err: unknown) => {
  throw new TRPCError({
    code: "INTERNAL_SERVER_ERROR",
    message: (err as Error).message ?? "Error, please try again",
  });
};
