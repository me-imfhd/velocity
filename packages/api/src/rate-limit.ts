import "server-only";
// requires env, thus should do operation server side only
import { Ratelimit } from "@upstash/ratelimit";
import { Redis } from "@upstash/redis";

export const preSignedUrlLimit = new Ratelimit({
  redis: Redis.fromEnv(),
  limiter: Ratelimit.slidingWindow(3, "60 s"),
  analytics: true,
  prefix: "@upstash/ratelimit",
});

export const createComputerLimit = new Ratelimit({
  redis: Redis.fromEnv(),
  limiter: Ratelimit.slidingWindow(5, "120 s"),
  analytics: true,
  prefix: "@upstash/ratelimit",
  
});
