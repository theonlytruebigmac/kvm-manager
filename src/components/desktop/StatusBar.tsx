import * as React from "react"
import { cn } from "@/lib/utils"

/**
 * StatusBar - Desktop-style status bar component
 *
 * A fixed bottom status bar with small text and subtle styling,
 * designed to look like a native desktop application status bar.
 * Items are automatically separated by vertical dividers.
 *
 * @example
 * ```tsx
 * <StatusBar>
 *   <StatusItem>{vmCount} VMs</StatusItem>
 *   <StatusItem>{runningCount} Running, {stoppedCount} Stopped</StatusItem>
 *   <StatusSpacer />
 *   <StatusItem>CPU: {cpuUsage}%</StatusItem>
 *   <StatusItem>Memory: {memUsed}/{memTotal}GB</StatusItem>
 * </StatusBar>
 * ```
 */

interface StatusBarProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode
}

export const StatusBar = React.forwardRef<HTMLDivElement, StatusBarProps>(
  ({ className, children, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(
          "fixed bottom-0 left-0 right-0 z-40",
          "flex items-center gap-0",
          "h-6 px-3",
          "text-desktop-xs leading-none",
          "bg-[var(--toolbar-bg)] border-t border-[var(--toolbar-border)]",
          className
        )}
        {...props}
      >
        {children}
      </div>
    )
  }
)
StatusBar.displayName = "StatusBar"

/**
 * StatusItem - Individual status segment
 *
 * A single status item that displays information in the status bar.
 * Automatically includes a vertical divider on the left (except for the first item).
 */

interface StatusItemProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode
  variant?: "default" | "muted" | "warning" | "success" | "error"
}

export const StatusItem = React.forwardRef<HTMLDivElement, StatusItemProps>(
  ({ className, children, variant = "default", ...props }, ref) => {
    const variantStyles = {
      default: "text-foreground",
      muted: "text-muted-foreground",
      warning: "text-yellow-600 dark:text-yellow-500",
      success: "text-green-600 dark:text-green-500",
      error: "text-red-600 dark:text-red-500",
    }

    return (
      <div
        ref={ref}
        className={cn(
          "flex items-center gap-2",
          "px-2 py-1",
          "first:pl-0",
          "relative",
          // Add divider before each item except the first
          "before:content-[''] before:absolute before:left-0 before:top-1/2 before:-translate-y-1/2",
          "before:h-3 before:w-px before:bg-border",
          "first:before:hidden",
          variantStyles[variant],
          className
        )}
        {...props}
      >
        {children}
      </div>
    )
  }
)
StatusItem.displayName = "StatusItem"

/**
 * StatusSpacer - Flex spacer
 *
 * Pushes subsequent status items to the right side.
 * Useful for creating left-aligned and right-aligned groups.
 */

interface StatusSpacerProps extends React.HTMLAttributes<HTMLDivElement> {}

export const StatusSpacer = React.forwardRef<HTMLDivElement, StatusSpacerProps>(
  ({ className, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn("flex-1", className)}
        {...props}
      />
    )
  }
)
StatusSpacer.displayName = "StatusSpacer"

/**
 * StatusButton - Interactive status item
 *
 * A clickable status item for interactive elements like notifications or settings.
 * Supports hover effects and active state.
 */

interface StatusButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  children: React.ReactNode
}

export const StatusButton = React.forwardRef<HTMLButtonElement, StatusButtonProps>(
  ({ className, children, ...props }, ref) => {
    return (
      <button
        ref={ref}
        type="button"
        className={cn(
          "flex items-center gap-2",
          "px-2 py-1",
          "text-foreground",
          "relative",
          // Add divider before each button except the first
          "before:content-[''] before:absolute before:left-0 before:top-1/2 before:-translate-y-1/2",
          "before:h-3 before:w-px before:bg-border",
          "first:before:hidden",
          // Hover state
          "hover:bg-accent/50 rounded-sm",
          "transition-colors duration-150",
          // Focus state
          "focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring focus-visible:ring-offset-1",
          // Disabled state
          "disabled:pointer-events-none disabled:opacity-50",
          className
        )}
        {...props}
      >
        {children}
      </button>
    )
  }
)
StatusButton.displayName = "StatusButton"
