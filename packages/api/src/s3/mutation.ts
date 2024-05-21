import "server-only";
// requires env, thus should do operation server side only
import { createPresignedPost } from "@aws-sdk/s3-presigned-post";
import {
  ACCESS_KEY_ID,
  ACCESS_SECRET,
  BUCKET_NAME,
  REGION,
  throwTRPCError,
} from "../common";
import { S3Client } from "@aws-sdk/client-s3";
import { preSignedUrlLimit } from "../rate-limit";
import { TRPCError } from "@trpc/server";
import { checkAuth } from "@repo/auth/server";

const s3Client = new S3Client({
  credentials: {
    accessKeyId: ACCESS_KEY_ID,
    secretAccessKey: ACCESS_SECRET,
  },
  region: REGION,
});
// run this on server only
"use server"; // for server actions
export const getPresignedUrl = async () => {
  const { id } = await checkAuth();
  const { success } = await preSignedUrlLimit.limit(id);
  if (!success) {
    throw new TRPCError({
      code: "TOO_MANY_REQUESTS",
      message: "Rate Limit Exceeded, try after some time",
    });
  }
  try {
    const { url, fields } = await createPresignedPost(s3Client, {
      Bucket: BUCKET_NAME,
      Key: `Velocity/${id}/image.jpg`,
      Conditions: [
        ["content-length-range", 0, 5 * 1024 * 1024], // 5 MB max
      ],
      Expires: 3600,
    });

    return {
      preSignedUrl: url,
      fields,
    };
  } catch (error) {
    return throwTRPCError(error);
  }
};
