import { ReactNode, useState } from 'react'
import { NavLink } from 'react-router-dom'
import { Server, LayoutDashboard, Box, Database, Network, Lightbulb, FileText, Clock, Bell, HardDrive, Settings, ChevronLeft, ChevronRight } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'

interface LayoutProps {
  children: ReactNode
}

export function Layout({ children }: LayoutProps) {
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false)

  const navItems = [
    { to: '/dashboard', icon: LayoutDashboard, label: 'Dashboard' },
    { to: '/vms', icon: Box, label: 'Virtual Machines' },
    { to: '/storage', icon: Database, label: 'Storage' },
    { to: '/networks', icon: Network, label: 'Networks' },
    { to: '/insights', icon: Lightbulb, label: 'Insights' },
    { to: '/templates', icon: FileText, label: 'Templates' },
    { to: '/schedules', icon: Clock, label: 'Schedules' },
    { to: '/alerts', icon: Bell, label: 'Alerts' },
    { to: '/backups', icon: HardDrive, label: 'Backups' },
    { to: '/settings', icon: Settings, label: 'Settings' },
  ]

  return (
    <div className="h-screen w-screen flex bg-background overflow-hidden">
      {/* Sidebar */}
      <aside
        className={cn(
          "flex-shrink-0 border-r bg-card transition-all duration-300 ease-in-out flex flex-col",
          sidebarCollapsed ? "w-16" : "w-56"
        )}
      >
        {/* Sidebar Header */}
        <div className="h-14 border-b flex items-center justify-between px-4">
          {!sidebarCollapsed && (
            <div className="flex items-center gap-2">
              <Server className="w-5 h-5 text-primary" />
              <h1 className="text-sm font-bold">KVM Manager</h1>
            </div>
          )}
          {sidebarCollapsed && (
            <Server className="w-5 h-5 text-primary mx-auto" />
          )}
        </div>

        {/* Navigation */}
        <nav className="flex-1 overflow-y-auto py-4 scrollbar-thin">
          <div className="space-y-1 px-2">
            {navItems.map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                className={({ isActive }) =>
                  cn(
                    'flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all duration-200',
                    'hover:bg-accent',
                    isActive
                      ? 'bg-primary text-primary-foreground shadow-sm'
                      : 'text-muted-foreground hover:text-foreground',
                    sidebarCollapsed && 'justify-center'
                  )
                }
                title={sidebarCollapsed ? item.label : undefined}
              >
                <item.icon className="w-5 h-5 flex-shrink-0" />
                {!sidebarCollapsed && <span>{item.label}</span>}
              </NavLink>
            ))}
          </div>
        </nav>

        {/* Sidebar Footer - Collapse Toggle */}
        <div className="border-t p-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setSidebarCollapsed(!sidebarCollapsed)}
            className={cn("w-full", sidebarCollapsed && "px-0")}
            title={sidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"}
          >
            {sidebarCollapsed ? (
              <ChevronRight className="w-4 h-4" />
            ) : (
              <>
                <ChevronLeft className="w-4 h-4 mr-2" />
                <span className="text-xs">Collapse</span>
              </>
            )}
          </Button>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 min-h-0 overflow-hidden">
        {children}
      </main>
    </div>
  )
}
