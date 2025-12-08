import { ReactNode } from 'react'
import { NavLink } from 'react-router-dom'
import { Server, LayoutDashboard, Box, Database, Network } from 'lucide-react'
import { cn } from '@/lib/utils'

interface LayoutProps {
  children: ReactNode
}

export function Layout({ children }: LayoutProps) {
  return (
    <div className="h-full flex flex-col bg-background">
      <header className="flex-shrink-0 border-b bg-card">
        <div className="container mx-auto px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Server className="w-6 h-6 text-primary" />
              <h1 className="text-xl font-bold">KVM Manager</h1>
            </div>
            <nav className="flex items-center gap-6">
              <NavLink
                to="/dashboard"
                className={({ isActive }) =>
                  cn(
                    'flex items-center gap-2 text-sm font-medium transition-colors hover:text-primary',
                    isActive
                      ? 'text-primary'
                      : 'text-muted-foreground'
                  )
                }
              >
                <LayoutDashboard className="w-4 h-4" />
                Dashboard
              </NavLink>
              <NavLink
                to="/vms"
                className={({ isActive }) =>
                  cn(
                    'flex items-center gap-2 text-sm font-medium transition-colors hover:text-primary',
                    isActive
                      ? 'text-primary'
                      : 'text-muted-foreground'
                  )
                }
              >
                <Box className="w-4 h-4" />
                Virtual Machines
              </NavLink>
              <NavLink
                to="/storage"
                className={({ isActive }) =>
                  cn(
                    'flex items-center gap-2 text-sm font-medium transition-colors hover:text-primary',
                    isActive
                      ? 'text-primary'
                      : 'text-muted-foreground'
                  )
                }
              >
                <Database className="w-4 h-4" />
                Storage
              </NavLink>
              <NavLink
                to="/networks"
                className={({ isActive }) =>
                  cn(
                    'flex items-center gap-2 text-sm font-medium transition-colors hover:text-primary',
                    isActive
                      ? 'text-primary'
                      : 'text-muted-foreground'
                  )
                }
              >
                <Network className="w-4 h-4" />
                Networks
              </NavLink>
            </nav>
          </div>
        </div>
      </header>
      <main className="flex-1 overflow-hidden">
        <div className="h-full overflow-y-auto">
          <div className="container mx-auto px-6 py-8 h-full">
            {children}
          </div>
        </div>
      </main>
    </div>
  )
}
