import * as React from "react"
import { type LucideIcon } from "lucide-react"
import { cn } from "@/lib/utils"

/**
 * Toolbar - Desktop-style toolbar container
 *
 * A horizontal toolbar with tight spacing and subtle background,
 * designed to look like a native desktop application toolbar.
 *
 * @example
 * ```tsx
 * <Toolbar>
 *   <ToolbarButton icon={Plus} onClick={handleNew}>New VM</ToolbarButton>
 *   <ToolbarButton icon={Play} onClick={handleStart}>Start</ToolbarButton>
 *   <ToolbarSeparator />
 *   <ToolbarButton icon={Monitor} onClick={handleConsole}>Console</ToolbarButton>
 *   <ToolbarSpacer />
 *   <ToolbarButton icon={Settings} onClick={handleSettings} />
 * </Toolbar>
 * ```
 */

interface ToolbarProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode
}

export const Toolbar = React.forwardRef<HTMLDivElement, ToolbarProps>(
  ({ className, children, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(
          "flex items-center gap-1 bg-[var(--toolbar-bg)] border-b border-[var(--toolbar-border)] px-2 py-1",
          className
        )}
        {...props}
      >
        {children}
      </div>
    )
  }
)
Toolbar.displayName = "Toolbar"

/**
 * ToolbarButton - Icon button with optional label
 *
 * A compact button designed for toolbar use with an icon and optional text label.
 * Supports disabled state and hover effects.
 */

interface ToolbarButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  icon: LucideIcon
  children?: React.ReactNode
  tooltip?: string
}

export const ToolbarButton = React.forwardRef<HTMLButtonElement, ToolbarButtonProps>(
  ({ className, icon: Icon, children, tooltip, disabled, ...props }, ref) => {
    const button = (
      <button
        ref={ref}
        className={cn(
          "inline-flex items-center gap-1.5 px-2.5 py-1 rounded",
          "text-desktop-sm font-medium transition-colors",
          "hover:bg-[var(--toolbar-hover)]",
          "focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring",
          "disabled:pointer-events-none disabled:opacity-50",
          className
        )}
        disabled={disabled}
        type="button"
        {...props}
      >
        <Icon className="w-4 h-4" />
        {children && <span>{children}</span>}
      </button>
    )

    // If tooltip is provided, wrap in a simple title attribute
    // For more advanced tooltips, integrate with a tooltip library
    if (tooltip && !disabled) {
      return (
        <div title={tooltip}>
          {button}
        </div>
      )
    }

    return button
  }
)
ToolbarButton.displayName = "ToolbarButton"

/**
 * ToolbarSeparator - Visual divider
 *
 * A vertical line to visually separate groups of toolbar buttons.
 */

interface ToolbarSeparatorProps extends React.HTMLAttributes<HTMLDivElement> {}

export const ToolbarSeparator = React.forwardRef<HTMLDivElement, ToolbarSeparatorProps>(
  ({ className, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(
          "w-px h-6 bg-border mx-1",
          className
        )}
        {...props}
      />
    )
  }
)
ToolbarSeparator.displayName = "ToolbarSeparator"

/**
 * ToolbarSpacer - Flex spacer
 *
 * Pushes subsequent toolbar items to the right side.
 * Useful for creating left-aligned and right-aligned groups.
 */

interface ToolbarSpacerProps extends React.HTMLAttributes<HTMLDivElement> {}

export const ToolbarSpacer = React.forwardRef<HTMLDivElement, ToolbarSpacerProps>(
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
ToolbarSpacer.displayName = "ToolbarSpacer"
