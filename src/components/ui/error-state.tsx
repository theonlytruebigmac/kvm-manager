import { AlertCircle, RefreshCw } from 'lucide-react'
import { Button } from './button'
import { Card, CardContent } from './card'

interface ErrorStateProps {
  title?: string
  message: string
  onRetry?: () => void
  suggestion?: string
}

export function ErrorState({ title = 'Error', message, onRetry, suggestion }: ErrorStateProps) {
  return (
    <div className="h-full flex items-center justify-center p-6">
      <Card className="max-w-md">
        <CardContent className="p-6">
          <div className="flex flex-col items-center gap-4 text-center">
            <div className="rounded-full bg-destructive/10 p-3">
              <AlertCircle className="w-8 h-8 text-destructive" />
            </div>
            <div className="space-y-2">
              <h3 className="font-semibold text-lg">{title}</h3>
              <p className="text-sm text-muted-foreground">{message}</p>
              {suggestion && (
                <p className="text-xs text-muted-foreground border-l-2 border-border pl-3 py-1 mt-3 text-left">
                  💡 {suggestion}
                </p>
              )}
            </div>
            {onRetry && (
              <Button onClick={onRetry} variant="outline" size="sm" className="mt-2">
                <RefreshCw className="w-3.5 h-3.5 mr-2" />
                Try Again
              </Button>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

interface EmptyStateProps {
  icon: React.ReactNode
  title: string
  description: string
  action?: {
    label: string
    onClick: () => void
  }
}

export function EmptyState({ icon, title, description, action }: EmptyStateProps) {
  return (
    <div className="h-full flex items-center justify-center p-6">
      <Card className="max-w-md">
        <CardContent className="p-12">
          <div className="flex flex-col items-center gap-4 text-center">
            <div className="text-muted-foreground">{icon}</div>
            <div className="space-y-2">
              <h3 className="font-semibold">{title}</h3>
              <p className="text-sm text-muted-foreground">{description}</p>
            </div>
            {action && (
              <Button onClick={action.onClick} className="mt-2">
                {action.label}
              </Button>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
