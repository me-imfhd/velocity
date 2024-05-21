import { cva } from "class-variance-authority";

export const buttonVariants = cva(
  "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50",
  {
    variants: {
      variant: {
        default: "bg-primary text-primary-foreground hover:bg-primary/90",
        destructive:
          "bg-destructive text-destructive-foreground hover:bg-destructive/90",
        outline:
          "border border-input bg-background hover:bg-accent hover:text-accent-foreground",
        secondary:
          "bg-secondary text-secondary-foreground hover:bg-secondary/80",
        ghost: "hover:bg-accent hover:text-accent-foreground",
        link: "text-primary underline-offset-4 hover:underline",
      },
      size: {
        default: "h-10 px-4 py-2",
        xs: "h-5 px-2 rounded-full text-xs", // pill-shaped
        sm: "h-[2.3rem] rounded-md px-3",
        lg: "h-11 rounded-md px-8",
        icon: "h-8 w-8",
      },
      animationType: {
        none: "transition-all",
        up: "animate-fade-up",
        down: "animate-fade-down",
        right: "animate-fade-right",
        left: "animate-fade-left",
      },
      tap: {
        default: "",
        in: "active:scale-[0.97]",
        out: "active:scale-[1.03]",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
      animationType: "none",
      tap: "default",
    },
  }
);
// animationType === "none" && "transition-all",
// isLoading && "cursor-default opacity-60 transition-all",
// disabled && "cursor-not-allowed opacity-40";
