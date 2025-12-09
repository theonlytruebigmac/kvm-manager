import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import type { OptimizationSuggestion, OptimizationCategory, OptimizationSeverity } from '@/lib/types'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Cpu,
  MemoryStick,
  HardDrive,
  Network,
  AlertCircle,
  AlertTriangle,
  Info,
  RefreshCw
} from 'lucide-react'

interface OptimizationSuggestionsProps {
  vmId?: string
  vmName?: string
}

export function OptimizationSuggestions({ vmId, vmName }: OptimizationSuggestionsProps) {
  const [timeRange, setTimeRange] = useState<number>(24)

  // Query for suggestions
  const { data: suggestions, isLoading, refetch } = useQuery<OptimizationSuggestion[]>({
    queryKey: vmId ? ['optimizationSuggestions', vmId, timeRange] : ['optimizationSuggestions', 'all', timeRange],
    queryFn: () => vmId && vmName
      ? api.analyzeVmPerformance(vmId, vmName, timeRange)
      : api.analyzeAllVms(timeRange),
    refetchInterval: 5 * 60 * 1000, // Refetch every 5 minutes
  })

  const getCategoryIcon = (category: OptimizationCategory) => {
    switch (category) {
      case 'cpu':
        return <Cpu className="h-4 w-4" />
      case 'memory':
        return <MemoryStick className="h-4 w-4" />
      case 'disk':
        return <HardDrive className="h-4 w-4" />
      case 'network':
        return <Network className="h-4 w-4" />
    }
  }

  const getSeverityIcon = (severity: OptimizationSeverity) => {
    switch (severity) {
      case 'critical':
        return <AlertCircle className="h-4 w-4 text-red-500" />
      case 'warning':
        return <AlertTriangle className="h-4 w-4 text-yellow-500" />
      case 'info':
        return <Info className="h-4 w-4 text-blue-500" />
    }
  }

  const getSeverityBadge = (severity: OptimizationSeverity) => {
    switch (severity) {
      case 'critical':
        return <Badge variant="destructive">Critical</Badge>
      case 'warning':
        return <Badge className="bg-yellow-500 hover:bg-yellow-600">Warning</Badge>
      case 'info':
        return <Badge variant="secondary">Info</Badge>
    }
  }

  const getCategoryLabel = (category: OptimizationCategory) => {
    return category.charAt(0).toUpperCase() + category.slice(1)
  }

  if (isLoading) {
    return (
      <Card className="border-border/40 shadow-sm">
        <CardContent className="flex items-center justify-center p-8">
          <div className="text-muted-foreground">Analyzing performance...</div>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card className="border-border/40 shadow-sm">
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="text-lg font-semibold">Performance Optimization Suggestions</CardTitle>
            <CardDescription>AI-driven recommendations based on historical metrics</CardDescription>
          </div>
          <div className="flex items-center gap-2">
            <select
              value={timeRange}
              onChange={(e) => setTimeRange(parseInt(e.target.value))}
              className="flex h-9 rounded-md border border-input bg-background px-3 py-1 text-sm ring-offset-background"
            >
              <option value="1">Last Hour</option>
              <option value="6">Last 6 Hours</option>
              <option value="24">Last 24 Hours</option>
              <option value="168">Last 7 Days</option>
              <option value="720">Last 30 Days</option>
            </select>
            <Button size="sm" variant="outline" onClick={() => refetch()}>
              <RefreshCw className="h-3 w-3" />
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        {!suggestions || suggestions.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-8 text-center">
            <Info className="h-12 w-12 text-green-500 mb-4" />
            <p className="font-medium">No optimization suggestions</p>
            <p className="text-sm text-muted-foreground mt-1">
              {vmId
                ? 'This VM is running optimally based on current metrics'
                : 'All VMs are running optimally based on current metrics'}
            </p>
          </div>
        ) : (
          <div className="space-y-3">
          {suggestions.map((suggestion, index) => (
            <Card key={index} className="border-l-4" style={{
              borderLeftColor:
                suggestion.severity === 'critical' ? 'rgb(239, 68, 68)' :
                suggestion.severity === 'warning' ? 'rgb(234, 179, 8)' :
                'rgb(59, 130, 246)'
            }}>
              <CardHeader className="pb-3">
                <div className="flex items-start justify-between">
                  <div className="flex items-center gap-2">
                    {getSeverityIcon(suggestion.severity)}
                    <CardTitle className="text-base">{suggestion.title}</CardTitle>
                  </div>
                  <div className="flex items-center gap-2">
                    {getSeverityBadge(suggestion.severity)}
                    <Badge variant="outline" className="flex items-center gap-1">
                      {getCategoryIcon(suggestion.category)}
                      {getCategoryLabel(suggestion.category)}
                    </Badge>
                  </div>
                </div>
                {!vmId && (
                  <CardDescription className="text-xs">
                    VM: {suggestion.vmName}
                  </CardDescription>
                )}
              </CardHeader>
              <CardContent className="space-y-3">
                <div>
                  <div className="text-sm text-muted-foreground mb-1">Analysis</div>
                  <p className="text-sm">{suggestion.description}</p>
                </div>

                <div>
                  <div className="text-sm text-muted-foreground mb-1">Recommendation</div>
                  <p className="text-sm font-medium">{suggestion.recommendation}</p>
                </div>

                <div className="flex items-center gap-4 text-xs text-muted-foreground pt-2 border-t">
                  <div>
                    <span className="font-medium">Current Value:</span>{' '}
                    {suggestion.currentValue.toFixed(2)}
                  </div>
                  <div>
                    <span className="font-medium">Threshold:</span>{' '}
                    {suggestion.threshold.toFixed(2)}
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
          </div>
        )}
      </CardContent>
    </Card>
  )
}
