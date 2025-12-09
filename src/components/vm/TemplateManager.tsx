import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/tauri'
import type { VmTemplate, CreateTemplateRequest, VmConfig } from '@/lib/types'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { toast } from 'sonner'
import { FileText, Plus, Trash2, Edit, Play, Calendar } from 'lucide-react'

interface TemplateManagerProps {
  onCreateFromTemplate?: (config: VmConfig) => void
}

export function TemplateManager({ onCreateFromTemplate }: TemplateManagerProps) {
  const queryClient = useQueryClient()
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [isEditDialogOpen, setIsEditDialogOpen] = useState(false)
  const [selectedTemplate, setSelectedTemplate] = useState<VmTemplate | null>(null)
  const [templateForm, setTemplateForm] = useState<CreateTemplateRequest>({
    name: '',
    description: '',
    config: {
      name: '',
      cpuCount: 2,
      memoryMb: 2048,
      diskSizeGb: 20,
      osType: 'linux',
      network: 'default',
      diskFormat: 'qcow2',
      bootMenu: false,
    },
  })

  // Query for templates
  const { data: templates, isLoading } = useQuery<VmTemplate[]>({
    queryKey: ['templates'],
    queryFn: () => api.listTemplates(),
  })

  // Create template mutation
  const createMutation = useMutation({
    mutationFn: (request: CreateTemplateRequest) => api.createTemplate(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['templates'] })
      toast.success('Template created successfully')
      setIsCreateDialogOpen(false)
      resetForm()
    },
    onError: (error: Error) => {
      toast.error(`Failed to create template: ${error.message}`)
    },
  })

  // Update template mutation
  const updateMutation = useMutation({
    mutationFn: ({ id, request }: { id: string; request: CreateTemplateRequest }) =>
      api.updateTemplate(id, request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['templates'] })
      toast.success('Template updated successfully')
      setIsEditDialogOpen(false)
      resetForm()
    },
    onError: (error: Error) => {
      toast.error(`Failed to update template: ${error.message}`)
    },
  })

  // Delete template mutation
  const deleteMutation = useMutation({
    mutationFn: (id: string) => api.deleteTemplate(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['templates'] })
      toast.success('Template deleted successfully')
    },
    onError: (error: Error) => {
      toast.error(`Failed to delete template: ${error.message}`)
    },
  })

  const resetForm = () => {
    setTemplateForm({
      name: '',
      description: '',
      config: {
        name: '',
        cpuCount: 2,
        memoryMb: 2048,
        diskSizeGb: 20,
        osType: 'linux',
        network: 'default',
        diskFormat: 'qcow2',
        bootMenu: false,
      },
    })
    setSelectedTemplate(null)
  }

  const handleCreate = () => {
    createMutation.mutate(templateForm)
  }

  const handleUpdate = () => {
    if (selectedTemplate) {
      updateMutation.mutate({ id: selectedTemplate.id, request: templateForm })
    }
  }

  const handleEdit = (template: VmTemplate) => {
    setSelectedTemplate(template)
    setTemplateForm({
      name: template.name,
      description: template.description,
      config: template.config,
    })
    setIsEditDialogOpen(true)
  }

  const handleDelete = (id: string, name: string) => {
    if (confirm(`Are you sure you want to delete template "${name}"?`)) {
      deleteMutation.mutate(id)
    }
  }

  const handleUseTemplate = (template: VmTemplate) => {
    if (onCreateFromTemplate) {
      onCreateFromTemplate(template.config)
      toast.success(`Using template: ${template.name}`)
    }
  }

  const formatDate = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString()
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="text-muted-foreground">Loading templates...</div>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold">VM Templates</h2>
          <p className="text-muted-foreground">Save and reuse VM configurations</p>
        </div>
        <Button onClick={() => setIsCreateDialogOpen(true)}>
          <Plus className="mr-2 h-4 w-4" />
          Create Template
        </Button>
      </div>

      {!templates || templates.length === 0 ? (
        <Card>
          <CardContent className="flex flex-col items-center justify-center p-8">
            <FileText className="h-12 w-12 text-muted-foreground mb-4" />
            <p className="text-muted-foreground text-center">
              No templates yet. Create a template to save VM configurations for reuse.
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {templates.map((template) => (
            <Card key={template.id}>
              <CardHeader>
                <div className="flex justify-between items-start">
                  <div>
                    <CardTitle className="text-lg">{template.name}</CardTitle>
                    <CardDescription className="text-sm mt-1">
                      {template.description}
                    </CardDescription>
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div className="flex flex-wrap gap-2">
                    <Badge variant="secondary">
                      {template.config.cpuCount} CPU
                    </Badge>
                    <Badge variant="secondary">
                      {(template.config.memoryMb / 1024).toFixed(1)} GB RAM
                    </Badge>
                    <Badge variant="secondary">
                      {template.config.diskSizeGb} GB Disk
                    </Badge>
                    <Badge variant="outline">{template.config.osType}</Badge>
                  </div>

                  <div className="flex items-center text-xs text-muted-foreground">
                    <Calendar className="mr-1 h-3 w-3" />
                    {formatDate(template.createdAt)}
                  </div>

                  <div className="flex gap-2 pt-2">
                    <Button
                      size="sm"
                      onClick={() => handleUseTemplate(template)}
                      className="flex-1"
                    >
                      <Play className="mr-1 h-3 w-3" />
                      Use
                    </Button>
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => handleEdit(template)}
                    >
                      <Edit className="h-3 w-3" />
                    </Button>
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => handleDelete(template.id, template.name)}
                    >
                      <Trash2 className="h-3 w-3" />
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {/* Create Template Dialog */}
      <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
        <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Create Template</DialogTitle>
            <DialogDescription>
              Save a VM configuration as a template for reuse
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <Label htmlFor="template-name">Template Name</Label>
              <Input
                id="template-name"
                value={templateForm.name}
                onChange={(e) =>
                  setTemplateForm({ ...templateForm, name: e.target.value })
                }
                placeholder="e.g., Ubuntu 22.04 Server"
              />
            </div>
            <div>
              <Label htmlFor="template-description">Description</Label>
              <Input
                id="template-description"
                value={templateForm.description}
                onChange={(e) =>
                  setTemplateForm({ ...templateForm, description: e.target.value })
                }
                placeholder="e.g., Standard Ubuntu server configuration"
              />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label htmlFor="cpu-count">CPU Cores</Label>
                <Input
                  id="cpu-count"
                  type="number"
                  min="1"
                  max="64"
                  value={templateForm.config.cpuCount}
                  onChange={(e) =>
                    setTemplateForm({
                      ...templateForm,
                      config: {
                        ...templateForm.config,
                        cpuCount: parseInt(e.target.value),
                      },
                    })
                  }
                />
              </div>
              <div>
                <Label htmlFor="memory">Memory (GB)</Label>
                <Input
                  id="memory"
                  type="number"
                  min="0.5"
                  step="0.5"
                  value={templateForm.config.memoryMb / 1024}
                  onChange={(e) =>
                    setTemplateForm({
                      ...templateForm,
                      config: {
                        ...templateForm.config,
                        memoryMb: Math.round(parseFloat(e.target.value) * 1024),
                      },
                    })
                  }
                />
              </div>
              <div>
                <Label htmlFor="disk-size">Disk Size (GB)</Label>
                <Input
                  id="disk-size"
                  type="number"
                  min="1"
                  value={templateForm.config.diskSizeGb}
                  onChange={(e) =>
                    setTemplateForm({
                      ...templateForm,
                      config: {
                        ...templateForm.config,
                        diskSizeGb: parseInt(e.target.value),
                      },
                    })
                  }
                />
              </div>
              <div>
                <Label htmlFor="os-type">OS Type</Label>
                <select
                  id="os-type"
                  value={templateForm.config.osType}
                  onChange={(e) =>
                    setTemplateForm({
                      ...templateForm,
                      config: {
                        ...templateForm.config,
                        osType: e.target.value as 'linux' | 'windows' | 'other',
                      },
                    })
                  }
                  className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background"
                >
                  <option value="linux">Linux</option>
                  <option value="windows">Windows</option>
                  <option value="other">Other</option>
                </select>
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setIsCreateDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleCreate} disabled={!templateForm.name}>
              Create Template
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Edit Template Dialog */}
      <Dialog open={isEditDialogOpen} onOpenChange={setIsEditDialogOpen}>
        <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Edit Template</DialogTitle>
            <DialogDescription>
              Update the template configuration
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <Label htmlFor="edit-template-name">Template Name</Label>
              <Input
                id="edit-template-name"
                value={templateForm.name}
                onChange={(e) =>
                  setTemplateForm({ ...templateForm, name: e.target.value })
                }
              />
            </div>
            <div>
              <Label htmlFor="edit-template-description">Description</Label>
              <Input
                id="edit-template-description"
                value={templateForm.description}
                onChange={(e) =>
                  setTemplateForm({ ...templateForm, description: e.target.value })
                }
              />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label htmlFor="edit-cpu-count">CPU Cores</Label>
                <Input
                  id="edit-cpu-count"
                  type="number"
                  min="1"
                  max="64"
                  value={templateForm.config.cpuCount}
                  onChange={(e) =>
                    setTemplateForm({
                      ...templateForm,
                      config: {
                        ...templateForm.config,
                        cpuCount: parseInt(e.target.value),
                      },
                    })
                  }
                />
              </div>
              <div>
                <Label htmlFor="edit-memory">Memory (GB)</Label>
                <Input
                  id="edit-memory"
                  type="number"
                  min="0.5"
                  step="0.5"
                  value={templateForm.config.memoryMb / 1024}
                  onChange={(e) =>
                    setTemplateForm({
                      ...templateForm,
                      config: {
                        ...templateForm.config,
                        memoryMb: Math.round(parseFloat(e.target.value) * 1024),
                      },
                    })
                  }
                />
              </div>
              <div>
                <Label htmlFor="edit-disk-size">Disk Size (GB)</Label>
                <Input
                  id="edit-disk-size"
                  type="number"
                  min="1"
                  value={templateForm.config.diskSizeGb}
                  onChange={(e) =>
                    setTemplateForm({
                      ...templateForm,
                      config: {
                        ...templateForm.config,
                        diskSizeGb: parseInt(e.target.value),
                      },
                    })
                  }
                />
              </div>
              <div>
                <Label htmlFor="edit-os-type">OS Type</Label>
                <select
                  id="edit-os-type"
                  value={templateForm.config.osType}
                  onChange={(e) =>
                    setTemplateForm({
                      ...templateForm,
                      config: {
                        ...templateForm.config,
                        osType: e.target.value as 'linux' | 'windows' | 'other',
                      },
                    })
                  }
                  className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background"
                >
                  <option value="linux">Linux</option>
                  <option value="windows">Windows</option>
                  <option value="other">Other</option>
                </select>
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setIsEditDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleUpdate} disabled={!templateForm.name}>
              Update Template
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
