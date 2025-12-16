import { useState } from 'react'
import {
  Monitor,
  Cpu,
  MemoryStick,
  Settings,
  HardDrive,
  Disc,
  Network,
  Video,
  Volume2,
  Keyboard,
  ShieldCheck,
  Plus,
  ChevronRight,
  ChevronDown,
  Edit,
  Trash2,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from '@/components/ui/context-menu'
import { cn } from '@/lib/utils'
import type { VM } from '@/lib/types'

interface HardwareTreeProps {
  vm: VM
  selectedItem: string
  onSelectItem: (itemId: string) => void
  onAddHardware?: () => void
}

interface TreeItemProps {
  id: string
  icon: React.ElementType
  label: string
  selected: boolean
  onClick: () => void
  onEdit?: () => void
  onRemove?: () => void
  indent?: number
}

function TreeItem({ icon: Icon, label, selected, onClick, onEdit, onRemove, indent = 0 }: TreeItemProps) {
  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>
    <button
      onClick={onClick}
      onDoubleClick={onEdit}
      className={cn(
        'w-full flex items-center gap-2 px-2 py-1 text-desktop-sm rounded transition-colors',
        'hover:bg-[var(--sidebar-hover)]',
        selected && 'bg-[var(--selected-bg)] font-medium',
        'text-left'
      )}
      style={{ paddingLeft: `${8 + indent * 12}px` }}
    >
      <Icon className="w-3.5 h-3.5 flex-shrink-0" />
      <span className="truncate">{label}</span>
    </button>
      </ContextMenuTrigger>

      <ContextMenuContent>
        {onEdit && (
          <ContextMenuItem icon={<Edit className="w-3.5 h-3.5" />} onClick={onEdit}>
            Edit Device
          </ContextMenuItem>
        )}
        {onRemove && (
          <ContextMenuItem icon={<Trash2 className="w-3.5 h-3.5" />} onClick={onRemove}>
            Remove Device
          </ContextMenuItem>
        )}
        {(onEdit || onRemove) && <ContextMenuSeparator />}
        <ContextMenuItem icon={<Plus className="w-3.5 h-3.5" />}>
          Add Hardware
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  )
}

interface TreeSectionProps {
  title: string
  children: React.ReactNode
  defaultExpanded?: boolean
}

function TreeSection({ title, children, defaultExpanded = true }: TreeSectionProps) {
  const [expanded, setExpanded] = useState(defaultExpanded)

  return (
    <div className="mb-1">
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center gap-1 px-2 py-0.5 text-desktop-xs font-semibold text-muted-foreground uppercase tracking-wide hover:text-foreground transition-colors"
      >
        {expanded ? (
          <ChevronDown className="w-3 h-3" />
        ) : (
          <ChevronRight className="w-3 h-3" />
        )}
        {title}
      </button>
      {expanded && <div className="space-y-0.5 mt-0.5">{children}</div>}
    </div>
  )
}

export function HardwareTree({ vm, selectedItem, onSelectItem, onAddHardware }: HardwareTreeProps) {
  return (
    <div className="w-64 h-full border-r border-[var(--sidebar-border)] bg-[var(--sidebar-bg)] flex flex-col">
      {/* Scrollable tree area */}
      <div className="flex-1 overflow-y-auto p-2 space-y-2">
        {/* Hardware Section */}
        <TreeSection title="Hardware">
          <TreeItem
            id="overview"
            icon={Monitor}
            label="Overview"
            selected={selectedItem === 'overview'}
            onClick={() => onSelectItem('overview')}
          />
          <TreeItem
            id="cpu"
            icon={Cpu}
            label="CPUs"
            selected={selectedItem === 'cpu'}
            onClick={() => onSelectItem('cpu')}
          />
          <TreeItem
            id="memory"
            icon={MemoryStick}
            label="Memory"
            selected={selectedItem === 'memory'}
            onClick={() => onSelectItem('memory')}
          />
          <TreeItem
            id="numa"
            icon={Cpu}
            label="NUMA"
            selected={selectedItem === 'numa'}
            onClick={() => onSelectItem('numa')}
          />
          <TreeItem
            id="boot"
            icon={Settings}
            label="Boot Options"
            selected={selectedItem === 'boot'}
            onClick={() => onSelectItem('boot')}
          />
        </TreeSection>

        {/* Storage Section */}
        <TreeSection title="Storage">
          {vm.disks?.map((disk, index) => (
            <TreeItem
              key={`disk-${index}`}
              id={`disk-${index}`}
              icon={HardDrive}
              label={`${disk.bus?.toUpperCase() || 'VirtIO'} Disk ${index + 1}`}
              selected={selectedItem === `disk-${index}`}
              onClick={() => onSelectItem(`disk-${index}`)}
            />
          )) || (
            <TreeItem
              id="disk-0"
              icon={HardDrive}
              label="Disk 1"
              selected={selectedItem === 'disk-0'}
              onClick={() => onSelectItem('disk-0')}
            />
          )}
          {vm.cdrom && (
            <TreeItem
              id="cdrom"
              icon={Disc}
              label="CDROM"
              selected={selectedItem === 'cdrom'}
              onClick={() => onSelectItem('cdrom')}
            />
          )}
        </TreeSection>

        {/* Network Section */}
        <TreeSection title="Network">
          {vm.networkInterfaces?.map((nic, index) => (
            <TreeItem
              key={`nic-${index}`}
              id={`nic-${index}`}
              icon={Network}
              label={`NIC ${index + 1} (${nic.type || 'virtio'})`}
              selected={selectedItem === `nic-${index}`}
              onClick={() => onSelectItem(`nic-${index}`)}
            />
          )) || (
            <TreeItem
              id="nic-0"
              icon={Network}
              label="NIC 1"
              selected={selectedItem === 'nic-0'}
              onClick={() => onSelectItem('nic-0')}
            />
          )}
        </TreeSection>

        {/* Display Section */}
        <TreeSection title="Display">
          <TreeItem
            id="graphics"
            icon={Monitor}
            label={`Graphics (${vm.graphics?.type?.toUpperCase() || 'VNC'})`}
            selected={selectedItem === 'graphics'}
            onClick={() => onSelectItem('graphics')}
          />
          <TreeItem
            id="video"
            icon={Video}
            label={`Video (${vm.video?.model || 'virtio'})`}
            selected={selectedItem === 'video'}
            onClick={() => onSelectItem('video')}
          />
        </TreeSection>

        {/* Other Devices Section */}
        <TreeSection title="Other Devices">
          {vm.sound && (
            <TreeItem
              id="sound"
              icon={Volume2}
              label="Sound"
              selected={selectedItem === 'sound'}
              onClick={() => onSelectItem('sound')}
            />
          )}
          <TreeItem
            id="input"
            icon={Keyboard}
            label="Input Devices"
            selected={selectedItem === 'input'}
            onClick={() => onSelectItem('input')}
          />
          {vm.tpm && (
            <TreeItem
              id="tpm"
              icon={ShieldCheck}
              label="TPM"
              selected={selectedItem === 'tpm'}
              onClick={() => onSelectItem('tpm')}
            />
          )}
        </TreeSection>
      </div>

      {/* Add Hardware Button */}
      <div className="border-t border-[var(--sidebar-border)] p-2">
        <Button
          variant="outline"
          size="sm"
          className="w-full justify-start gap-2 h-7 text-desktop-xs"
          onClick={onAddHardware}
        >
          <Plus className="w-3.5 h-3.5" />
          Add Hardware
        </Button>
      </div>
    </div>
  )
}
