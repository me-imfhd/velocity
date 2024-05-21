import { type ClassValue, clsx } from "clsx";

import { twMerge } from "tailwind-merge";
export { cva, type VariantProps } from "class-variance-authority";
export { Balancer } from "react-wrap-balancer";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export { type ClassValue, clsx } from "clsx";
export { twMerge } from "tailwind-merge";
