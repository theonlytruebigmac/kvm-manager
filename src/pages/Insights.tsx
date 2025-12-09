import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useNavigate } from 'react-router-dom'
import { api } from '@/lib/tauri'
import type { VM, OptimizationSuggestion, OptimizationCategory, OptimizationSeverity } from '@/lib/types'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Lightbulb,
  Cpu,
  MemoryStick,
  HardDrive,
  Network,
  AlertCircle,
  AlertTriangle,
  Info,
  RefreshCw,
  TrendingUp,
  CheckCircle,
  ArrowRight
} from 'lucide-react'

export default function Insights() {
  const navigate = useNavigate()
  const [timeRange, setTimeRange] = useState<number>(24)

  // Query for all VMs
  const { data: vms } = useQuery<VM[]>({
    queryKey: ['vms'],
    queryFn: () => api.getVms(),
  })

  // Query for system-wide suggestions
  const { data: suggestions, isLoading, refetch } = useQuery<OptimizationSuggestion[]>({
    queryKey: ['systemOptimizationSuggestions', timeRange],
    queryFn: () => api.analyzeAllVms(timeRange),
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

  // Calculate statistics
  const totalVms = vms?.length || 0
  const runningVms = vms?.filter(vm => vm.state === 'running').length || 0
  const criticalCount = suggestions?.filter(s => s.severity === 'critical').length || 0
  const warningCount = suggestions?.filter(s => s.severity === 'warning').length || 0
  const infoCount = suggestions?.filter(s => s.severity === 'info').length || 0

  // Group suggestions by VM
  const suggestionsByVm = suggestions?.reduce((acc, suggestion) => {
    if (!acc[suggestion.vmId]) {
      acc[suggestion.vmId] = []
    }
    acc[suggestion.vmId].push(suggestion)
    return acc
  }, {} as Record<string, OptimizationSuggestion[]>)

  // Group suggestions by category
  const suggestionsByCategory = suggestions?.reduce((acc, suggestion) => {
    if (!acc[suggestion.category]) {
      acc[suggestion.category] = []
    }
    acc[suggestion.category].push(suggestion)
    return acc
  }, {} as Record<OptimizationCategory, OptimizationSuggestion[]>)

  const handleNavigateToVm = (vmId: string) => {
    navigate(`/vms/${vmId}`)
  }

  return (
    <div className="container mx-auto p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Lightbulb className="h-8 w-8 text-primary" />
          <div>
            <h1 className="text-3xl font-bold">System Insights</h1>
            <p className="text-muted-foreground">Performance analysis and optimization recommendations</p>
          </div>
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

      {/* Statistics Cards */}
      <div className="grid grid-cols-4 gap-4">
        <Card>
          <CardHeader className="pb-3">
            <CardDescription>Total VMs</CardDescription>
            <CardTitle className="text-3xl">{totalVms}</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-xs text-muted-foreground">
              {runningVms} running
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-3">
            <CardDescription className="flex items-center gap-1">
              <AlertCircle className="h-3 w-3 text-red-500" />
              Critical Issues
            </CardDescription>
            <CardTitle className="text-3xl">{criticalCount}</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-xs text-muted-foreground">
              Requires immediate attention
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-3">
            <CardDescription className="flex items-center gap-1">
              <AlertTriangle className="h-3 w-3 text-yellow-500" />
              Warnings
            </CardDescription>
            <CardTitle className="text-3xl">{warningCount}</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-xs text-muted-foreground">
              Should be reviewed
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-3">
            <CardDescription className="flex items-center gap-1">
              <Info className="h-3 w-3 text-blue-500" />
              Suggestions
            </CardDescription>
            <CardTitle className="text-3xl">{infoCount}</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-xs text-muted-foreground">
              Optimization opportunities
            </div>
          </CardContent>
        </Card>
      </div>

      {isLoading ? (
        <Card>
          <CardContent className="flex items-center justify-center p-8">
            <div className="text-muted-foreground">Analyzing system performance...</div>
          </CardContent>
        </Card>
      ) : !suggestions || suggestions.length === 0 ? (
        <Card>
          <CardContent className="flex flex-col items-center justify-center p-12 text-center">
            <CheckCircle className="h-16 w-16 text-green-500 mb-4" />
            <h3 className="text-xl font-semibold mb-2">System Running Optimally</h3>
            <p className="text-muted-foreground">
              No performance issues detected. All VMs are operating within normal parameters.
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-6 lg:grid-cols-2">
          {/* Suggestions by VM */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <TrendingUp className="h-5 w-5" />
                Issues by Virtual Machine
              </CardTitle>
              <CardDescription>
                Recommendations grouped by VM
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              {suggestionsByVm && Object.entries(suggestionsByVm).map(([vmId, vmSuggestions]) => {
                const vmName = vmSuggestions[0]?.vmName || vmId
                const criticals = vmSuggestions.filter(s => s.severity === 'critical').length
                const warnings = vmSuggestions.filter(s => s.severity === 'warning').length
                const infos = vmSuggestions.filter(s => s.severity === 'info').length

                return (
                  <div
                    key={vmId}
                    className="flex items-center justify-between p-3 border rounded-lg hover:bg-muted/50 cursor-pointer transition-colors"
                    onClick={() => handleNavigateToVm(vmId)}
                  >
                    <div className="flex-1">
                      <div className="font-medium">{vmName}</div>
                      <div className="flex items-center gap-2 mt-1">
                        {criticals > 0 && (
                          <Badge variant="destructive" className="text-xs">
                            {criticals} critical
                          </Badge>
                        )}
                        {warnings > 0 && (
                          <Badge className="bg-yellow-500 hover:bg-yellow-600 text-xs">
                            {warnings} warning
                          </Badge>
                        )}
                        {infos > 0 && (
                          <Badge variant="secondary" className="text-xs">
                            {infos} info
                          </Badge>
                        )}
                      </div>
                    </div>
                    <ArrowRight className="h-4 w-4 text-muted-foreground" />
                  </div>
                )
              })}
            </CardContent>
          </Card>

          {/* Suggestions by Category */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Cpu className="h-5 w-5" />
                Issues by Category
              </CardTitle>
              <CardDescription>
                Recommendations grouped by resource type
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              {suggestionsByCategory && Object.entries(suggestionsByCategory).map(([category, catSuggestions]) => {
                const criticals = catSuggestions.filter(s => s.severity === 'critical').length
                const warnings = catSuggestions.filter(s => s.severity === 'warning').length
                const infos = catSuggestions.filter(s => s.severity === 'info').length

                return (
                  <div
                    key={category}
                    className="flex items-center justify-between p-3 border rounded-lg"
                  >
                    <div className="flex items-center gap-2 flex-1">
                      {getCategoryIcon(category as OptimizationCategory)}
                      <div>
                        <div className="font-medium">
                          {getCategoryLabel(category as OptimizationCategory)}
                        </div>
                        <div className="text-xs text-muted-foreground">
                          {catSuggestions.length} issue{catSuggestions.length !== 1 ? 's' : ''}
                        </div>
                      </div>
                    </div>
                    <div className="flex items-center gap-1">
                      {criticals > 0 && (
                        <Badge variant="destructive" className="text-xs">
                          {criticals}
                        </Badge>
                      )}
                      {warnings > 0 && (
                        <Badge className="bg-yellow-500 hover:bg-yellow-600 text-xs">
                          {warnings}
                        </Badge>
                      )}
                      {infos > 0 && (
                        <Badge variant="secondary" className="text-xs">
                          {infos}
                        </Badge>
                      )}
                    </div>
                  </div>
                )
              })}
            </CardContent>
          </Card>
        </div>
      )}

      {/* All Suggestions */}
      {suggestions && suggestions.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>All Recommendations</CardTitle>
            <CardDescription>
              Complete list of performance optimization suggestions
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {suggestions.map((suggestion, index) => (
              <div
                key={index}
                className="border-l-4 p-4 rounded-lg border"
                style={{
                  borderLeftColor:
                    suggestion.severity === 'critical' ? 'rgb(239, 68, 68)' :
                    suggestion.severity === 'warning' ? 'rgb(234, 179, 8)' :
                    'rgb(59, 130, 246)'
                }}
              >
                <div className="flex items-start justify-between mb-2">
                  <div className="flex items-center gap-2">
                    {getSeverityIcon(suggestion.severity)}
                    <div className="font-semibold">{suggestion.title}</div>
                  </div>
                  <div className="flex items-center gap-2">
                    {getSeverityBadge(suggestion.severity)}
                    <Badge variant="outline" className="flex items-center gap-1">
                      {getCategoryIcon(suggestion.category)}
                      {getCategoryLabel(suggestion.category)}
                    </Badge>
                  </div>
                </div>
                <div className="text-sm text-muted-foreground mb-1">
                  VM: <span
                    className="text-primary cursor-pointer hover:underline"
                    onClick={() => handleNavigateToVm(suggestion.vmId)}
                  >
                    {suggestion.vmName}
                  </span>
                </div>
                <p className="text-sm mb-2">{suggestion.description}</p>
                <p className="text-sm font-medium text-primary">{suggestion.recommendation}</p>
              </div>
            ))}
          </CardContent>
        </Card>
      )}
    </div>
  )
}
