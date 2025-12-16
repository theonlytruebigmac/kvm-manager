import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { GripVertical, Save } from 'lucide-react'

interface BootOrderEditorProps {
  vmId: string
  vmName: string
  currentBootOrder: string[]
}

export function BootOrderEditor({ vmId, vmName, currentBootOrder }: BootOrderEditorProps) {
  const queryClient = useQueryClient()
  const [bootOrder, setBootOrder] = useState<string[]>(currentBootOrder.length > 0 ? currentBootOrder : ['hd'])
  const [hasChanges, setHasChanges] = useState(false)

  const updateMutation = useMutation({
    mutationFn: (newOrder: string[]) => api.updateVmBootOrder(vmId, newOrder),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vm', vmId] })
      toast.success('Boot order updated successfully')
      setHasChanges(false)
    },
    onError: (error: Error) => {
      toast.error('Failed to update boot order', {
        description: error.message
      })
    },
  })

  const handleReorder = (index: number, direction: 'up' | 'down') => {
    const newOrder = [...bootOrder]
    const targetIndex = direction === 'up' ? index - 1 : index + 1
    const temp = newOrder[index]
    newOrder[index] = newOrder[targetIndex]
    newOrder[targetIndex] = temp
    setBootOrder(newOrder)
    setHasChanges(true)
  }

  const handleRemove = (index: number) => {
    const newOrder = bootOrder.filter((_, i) => i !== index)
    if (newOrder.length === 0) {
      toast.error('At least one boot device is required')
      return
    }
    setBootOrder(newOrder)
    setHasChanges(true)
  }

  const handleAdd = (device: string) => {
    if (device && !bootOrder.includes(device)) {
      setBootOrder([...bootOrder, device])
      setHasChanges(true)
    }
  }

  const handleSave = () => {
    updateMutation.mutate(bootOrder)
  }

  const getDeviceLabel = (device: string) => {
    switch (device) {
      case 'hd': return 'Hard Disk'
      case 'cdrom': return 'CD-ROM'
      case 'network': return 'Network (PXE)'
      case 'fd': return 'Floppy'
      default: return device.toUpperCase()
    }
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Boot Order</CardTitle>
        <CardDescription>
          Configure the boot device priority for {vmName}. VM must be stopped to apply changes.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2 p-3 border rounded-lg bg-muted/50">
          {bootOrder.map((device, index) => (
            <div key={index} className="flex items-center gap-2 bg-background p-2 rounded">
              <GripVertical className="h-4 w-4 text-muted-foreground" />
              <span className="flex-1 font-mono text-sm">
                {index + 1}. {getDeviceLabel(device)}
              </span>
              <div className="flex gap-1">
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  disabled={index === 0}
                  onClick={() => handleReorder(index, 'up')}
                  title="Move up"
                >
                  ↑
                </Button>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  disabled={index === bootOrder.length - 1}
                  onClick={() => handleReorder(index, 'down')}
                  title="Move down"
                >
                  ↓
                </Button>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={() => handleRemove(index)}
                  title="Remove device"
                >
                  ✕
                </Button>
              </div>
            </div>
          ))}

          <Select value="" onValueChange={handleAdd}>
            <SelectTrigger>
              <SelectValue placeholder="Add boot device..." />
            </SelectTrigger>
            <SelectContent>
              {!bootOrder.includes('hd') && <SelectItem value="hd">Hard Disk</SelectItem>}
              {!bootOrder.includes('cdrom') && <SelectItem value="cdrom">CD-ROM</SelectItem>}
              {!bootOrder.includes('network') && <SelectItem value="network">Network (PXE)</SelectItem>}
              {!bootOrder.includes('fd') && <SelectItem value="fd">Floppy</SelectItem>}
            </SelectContent>
          </Select>
        </div>

        <div className="flex justify-between items-center">
          <p className="text-xs text-muted-foreground">
            Devices are tried in order. Use ↑↓ to reorder, ✕ to remove.
          </p>
          <Button
            onClick={handleSave}
            disabled={!hasChanges || updateMutation.isPending}
            size="sm"
            className="gap-2"
          >
            <Save className="h-4 w-4" />
            {updateMutation.isPending ? 'Saving...' : 'Save Changes'}
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}
