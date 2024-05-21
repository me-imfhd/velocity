import * as React from "react";
import { type VariantProps } from "class-variance-authority";
import { cn } from "../cn";
import { buttonVariants } from "./button-variants";
import { Loader2 } from "lucide-react";

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  variant?:
    | "default"
    | "destructive"
    | "ghost"
    | "link"
    | "outline"
    | "secondary";
  animationType?: "none" | "up" | "down" | "right" | "left";
  disabled?: boolean;
  tap?: "default" | "in" | "out";
  size?: "default" | "icon" | "lg" | "sm" | "xs";
  asChild?: boolean;
  withArrow?: boolean;
  buttonIcon?: React.ReactNode;
  iconDirection?: "left" | "right";
  isLoading?: boolean;
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      children,
      className,
      tap,
      variant,
      size,
      asChild = false,
      buttonIcon = <></>,
      iconDirection = "left",
      withArrow = false,
      disabled = false,
      isLoading = false,
      animationType,
      ...props
    },
    ref
  ) => {
    const buttonClassNames = cn(
      buttonVariants({ variant, size, className, animationType, tap }),
      !disabled && "hover:brightness-110 active:brightness-90",
      disabled && "cursor-not-allowed opacity-40",
      isLoading && "cursor-default opacity-60 transition-all"
    );
    return (
      <button
        className={buttonClassNames}
        ref={ref}
        disabled={disabled}
        {...props}
      >
        <span className="button-content-wrapper flex items-center gap-2">
          {isLoading && (
            <span key={"loader-wrapper"} className="hover:translate-x-3">
              <Loader2 className="h-4 w-4 animate-spin" />
            </span>
          )}
          {iconDirection === "left" && buttonIcon}
          {children}
          {iconDirection === "right" && buttonIcon}
        </span>
      </button>
    );
  }
);
Button.displayName = "Button";

export { Button };
