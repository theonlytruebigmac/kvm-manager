import { useState, useCallback } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { toast } from 'sonner'
import { cn } from '@/lib/utils'
import { Disc, Upload } from 'lucide-react'

interface DropZoneProps {
  vmId: string
  vmName: string
  vmState: string
  children: React.ReactNode
  className?: string
}

export function IsoDropZone({ vmId, vmName, vmState, children, className }: DropZoneProps) {
  const [isDragging, setIsDragging] = useState(false)
  const queryClient = useQueryClient()

  const mountMutation = useMutation({
    mutationFn: (isoPath: string) => api.mountIso(vmId, isoPath),
    onSuccess: () => {
      toast.success(`ISO mounted to ${vmName}`)
      queryClient.invalidateQueries({ queryKey: ['vm', vmId] })
    },
    onError: (error) => {
      toast.error(`Failed to mount ISO: ${error}`)
    },
  })

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()

    // Check if it's a file drag
    if (e.dataTransfer.types.includes('Files')) {
      setIsDragging(true)
    }
  }, [])

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDragging(false)
  }, [])

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDragging(false)

    // Get dropped files
    const files = Array.from(e.dataTransfer.files)

    // Find ISO files
    const isoFiles = files.filter(file =>
      file.name.toLowerCase().endsWith('.iso') ||
      file.name.toLowerCase().endsWith('.img')
    )

    if (isoFiles.length === 0) {
      toast.error('Please drop an ISO or IMG file')
      return
    }

    if (isoFiles.length > 1) {
      toast.error('Only one ISO file can be mounted at a time')
      return
    }

    const isoFile = isoFiles[0]

    // Get the file path - in Tauri, dropped files have a path property
    // @ts-ignore - Tauri adds path to dropped files
    const filePath = isoFile.path || isoFile.name

    if (vmState !== 'running') {
      toast.warning('ISO can only be mounted to running VMs')
      return
    }

    mountMutation.mutate(filePath)
  }, [vmId, vmState, mountMutation])

  // Render drop overlay when dragging
  return (
    <div
      className={cn('relative', className)}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      {children}

      {/* Drop overlay */}
      {isDragging && (
        <div className="absolute inset-0 z-50 flex items-center justify-center bg-primary/10 border-2 border-dashed border-primary rounded-lg backdrop-blur-sm">
          <div className="flex flex-col items-center gap-2 text-primary">
            <Disc className="w-8 h-8 animate-bounce" />
            <span className="text-sm font-medium">
              Drop ISO to mount
            </span>
          </div>
        </div>
      )}

      {/* Loading overlay */}
      {mountMutation.isPending && (
        <div className="absolute inset-0 z-50 flex items-center justify-center bg-background/50 backdrop-blur-sm">
          <div className="flex items-center gap-2 text-muted-foreground">
            <Upload className="w-5 h-5 animate-pulse" />
            <span className="text-sm">Mounting ISO...</span>
          </div>
        </div>
      )}
    </div>
  )
}
