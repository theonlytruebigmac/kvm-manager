import { Link, useLocation } from 'react-router-dom'
import { ChevronRight, Home } from 'lucide-react'
import { cn } from '@/lib/utils'

interface BreadcrumbItem {
  label: string
  href?: string
}

// Route configuration for breadcrumbs
const routeConfig: Record<string, { label: string; parent?: string }> = {
  '/': { label: 'Virtual Machines' },
  '/dashboard': { label: 'Dashboard' },
  '/performance': { label: 'Performance Monitor' },
  '/storage': { label: 'Storage' },
  '/networks': { label: 'Networks' },
  '/insights': { label: 'Insights' },
  '/templates': { label: 'Templates' },
  '/schedules': { label: 'Schedules' },
  '/alerts': { label: 'Alerts' },
  '/backups': { label: 'Backups' },
  '/settings': { label: 'Settings' },
}

export function Breadcrumbs() {
  const location = useLocation()
  const pathname = location.pathname

  // Build breadcrumb trail
  const getBreadcrumbs = (): BreadcrumbItem[] => {
    const crumbs: BreadcrumbItem[] = []

    // Add home
    crumbs.push({ label: 'Home', href: '/' })

    // Get current route config
    const currentRoute = routeConfig[pathname]
    if (currentRoute && pathname !== '/') {
      crumbs.push({ label: currentRoute.label })
    }

    // Handle VM details routes (/vms/:vmId)
    const vmMatch = pathname.match(/^\/vms\/(.+)$/)
    if (vmMatch) {
      crumbs.push({ label: 'Virtual Machines', href: '/' })
      crumbs.push({ label: 'VM Details' })
    }

    // Handle console routes (/console/:vmId)
    const consoleMatch = pathname.match(/^\/console\/(.+)$/)
    if (consoleMatch) {
      crumbs.push({ label: 'Virtual Machines', href: '/' })
      crumbs.push({ label: 'Console' })
    }

    return crumbs
  }

  const breadcrumbs = getBreadcrumbs()

  // Don't show breadcrumbs on root or if only home
  if (pathname === '/' || breadcrumbs.length <= 1) {
    return null
  }

  return (
    <div className="px-4 py-1.5 border-b border-[var(--panel-border)] bg-[var(--window-bg)]">
      <nav className="flex items-center gap-1 text-sm text-muted-foreground" aria-label="Breadcrumb">
        {breadcrumbs.map((crumb, index) => (
          <div key={index} className="flex items-center gap-1">
            {index > 0 && <ChevronRight className="w-3.5 h-3.5" />}
            {crumb.href && index < breadcrumbs.length - 1 ? (
              <Link
                to={crumb.href}
                className={cn(
                  'hover:text-foreground transition-colors flex items-center gap-1',
                  index === 0 && 'text-muted-foreground/80'
                )}
              >
                {index === 0 && <Home className="w-3.5 h-3.5" />}
                <span>{crumb.label}</span>
              </Link>
            ) : (
              <span className={cn(
                'flex items-center gap-1',
                index === breadcrumbs.length - 1 && 'text-foreground font-medium'
              )}>
                {index === 0 && <Home className="w-3.5 h-3.5" />}
                {crumb.label}
              </span>
            )}
          </div>
        ))}
      </nav>
    </div>
  )
}
