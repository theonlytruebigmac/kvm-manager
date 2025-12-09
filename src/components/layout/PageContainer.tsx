import { ReactNode } from 'react'

interface PageContainerProps {
  children: ReactNode
}

/**
 * PageContainer - Fixed-viewport page layout component
 *
 * Provides a full-height container that never scrolls at the page level.
 * Child components should implement their own scrolling areas as needed.
 */
export function PageContainer({ children }: PageContainerProps) {
  return (
    <div className="h-full w-full flex flex-col overflow-hidden">
      {children}
    </div>
  )
}

interface PageHeaderProps {
  title: string | ReactNode
  description?: string
  actions?: ReactNode
  stats?: ReactNode
}

/**
 * PageHeader - Fixed header section for pages
 * Will not scroll with page content
 */
export function PageHeader({ title, description, actions, stats }: PageHeaderProps) {
  return (
    <div className="flex-shrink-0 px-8 py-6 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="flex items-center justify-between gap-8">
        <div className="flex-1 min-w-0">
          {typeof title === 'string' ? (
            <h1 className="text-3xl font-bold tracking-tight">{title}</h1>
          ) : (
            <div className="text-3xl font-bold tracking-tight">{title}</div>
          )}
          {description && (
            <p className="text-sm text-muted-foreground mt-2">{description}</p>
          )}
        </div>
        {(actions || stats) && (
          <div className="flex items-center gap-4 flex-shrink-0">
            {stats}
            {actions}
          </div>
        )}
      </div>
    </div>
  )
}

interface PageToolbarProps {
  children: ReactNode
}

/**
 * PageToolbar - Fixed toolbar section below header
 * For filters, search, and other controls
 */
export function PageToolbar({ children }: PageToolbarProps) {
  return (
    <div className="flex-shrink-0 px-8 py-4 border-b bg-muted/20">
      {children}
    </div>
  )
}

interface PageContentProps {
  children: ReactNode
  noPadding?: boolean
}

/**
 * PageContent - Scrollable content area
 * Takes remaining vertical space and allows internal scrolling
 */
export function PageContent({ children, noPadding = false }: PageContentProps) {
  return (
    <div className={`flex-1 overflow-y-auto scrollbar-thin ${noPadding ? '' : 'px-8 py-8'}`}>
      {children}
    </div>
  )
}
