import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import { toast } from 'sonner'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { X, Plus, Tag } from 'lucide-react'

interface TagManagerProps {
  vmId: string
  currentTags: string[]
}

export function TagManager({ vmId, currentTags }: TagManagerProps) {
  const queryClient = useQueryClient()
  const [newTag, setNewTag] = useState('')
  const [isAdding, setIsAdding] = useState(false)

  const addTagMutation = useMutation({
    mutationFn: (tag: string) => api.addVmTags(vmId, [tag]),
    onSuccess: (_data, tag) => {
      queryClient.invalidateQueries({ queryKey: ['vm', vmId] })
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`Tag "${tag}" added`)
      setNewTag('')
      setIsAdding(false)
    },
    onError: (error, tag) => {
      toast.error(`Failed to add tag "${tag}": ${error}`)
    },
  })

  const removeTagMutation = useMutation({
    mutationFn: (tag: string) => api.removeVmTags(vmId, [tag]),
    onSuccess: (_data, tag) => {
      queryClient.invalidateQueries({ queryKey: ['vm', vmId] })
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      toast.success(`Tag "${tag}" removed`)
    },
    onError: (error, tag) => {
      toast.error(`Failed to remove tag "${tag}": ${error}`)
    },
  })

  const handleAddTag = () => {
    const trimmed = newTag.trim().toLowerCase()
    if (!trimmed) {
      toast.error('Tag cannot be empty')
      return
    }
    if (currentTags.includes(trimmed)) {
      toast.error('Tag already exists')
      return
    }
    if (trimmed.length > 20) {
      toast.error('Tag must be 20 characters or less')
      return
    }
    if (!/^[a-z0-9-]+$/.test(trimmed)) {
      toast.error('Tag can only contain lowercase letters, numbers, and hyphens')
      return
    }
    addTagMutation.mutate(trimmed)
  }

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleAddTag()
    } else if (e.key === 'Escape') {
      setIsAdding(false)
      setNewTag('')
    }
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Tag className="h-5 w-5" />
            <CardTitle>Tags</CardTitle>
          </div>
          {!isAdding && (
            <Button size="sm" variant="outline" onClick={() => setIsAdding(true)}>
              <Plus className="h-4 w-4 mr-1" />
              Add Tag
            </Button>
          )}
        </div>
        <CardDescription>Organize and categorize this VM</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="flex flex-wrap gap-2">
          {currentTags.length === 0 && !isAdding && (
            <p className="text-sm text-muted-foreground">No tags yet. Add some to organize your VMs.</p>
          )}

          {currentTags.map((tag) => (
            <Badge key={tag} variant="secondary" className="flex items-center gap-1 px-3 py-1">
              {tag}
              <button
                onClick={() => removeTagMutation.mutate(tag)}
                disabled={removeTagMutation.isPending}
                className="ml-1 hover:text-destructive transition-colors"
              >
                <X className="h-3 w-3" />
              </button>
            </Badge>
          ))}

          {isAdding && (
            <div className="flex items-center gap-2">
              <Input
                placeholder="Enter tag name"
                value={newTag}
                onChange={(e) => setNewTag(e.target.value)}
                onKeyDown={handleKeyPress}
                className="h-8 w-40"
                autoFocus
                disabled={addTagMutation.isPending}
              />
              <Button
                size="sm"
                onClick={handleAddTag}
                disabled={addTagMutation.isPending || !newTag.trim()}
              >
                Add
              </Button>
              <Button
                size="sm"
                variant="ghost"
                onClick={() => {
                  setIsAdding(false)
                  setNewTag('')
                }}
                disabled={addTagMutation.isPending}
              >
                Cancel
              </Button>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  )
}
