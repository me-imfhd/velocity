import axios from "axios";
import { db } from "@repo/db";
import { CLOUDFRONT_URL, IdType, throwTRPCError } from "../common";

// call on client side
export const uploadImage = async (
  userId: IdType,
  file: File,
  fields: Record<string, string>,
  preSignedUrl: string
) => {
  try {
    const formData = new FormData();
    formData.set("bucket", fields["bucket"]!);
    formData.set("X-Amz-Algorithm", fields["X-Amz-Algorithm"]!);
    formData.set("X-Amz-Credential", fields["X-Amz-Credential"]!);
    formData.set("X-Amz-Algorithm", fields["X-Amz-Algorithm"]!);
    formData.set("X-Amz-Date", fields["X-Amz-Date"]!);
    formData.set("key", fields["key"]!);
    formData.set("Policy", fields["Policy"]!);
    formData.set("X-Amz-Signature", fields["X-Amz-Signature"]!);
    formData.set("X-Amz-Algorithm", fields["X-Amz-Algorithm"]!);
    formData.append("file", file);
    axios.post(preSignedUrl, formData);

    const image = `${CLOUDFRONT_URL}/${fields["key"]!}`;
    await db.user.update({ where: { id: userId }, data: { image } });
    return image;
  } catch (e) {
    return throwTRPCError(e);
  }
};
