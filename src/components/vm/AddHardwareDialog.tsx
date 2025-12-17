import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { open as openFileDialog } from '@tauri-apps/plugin-dialog'
import { homeDir } from '@tauri-apps/api/path'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'
import { api } from '@/lib/tauri'
import {
  HardDrive,
  Disc,
  Network,
  Monitor,
  Video,
  Volume2,
  Keyboard,
  ShieldCheck,
  Cpu,
  Usb,
  Gamepad2,
  FileText,
  Dice1,
  AlertTriangle,
  Radio,
  CreditCard,
  Unplug,
  Check,
  Info,
  FolderOpen,
  Cable,
} from 'lucide-react'
import { toast } from 'sonner'
import type { VM } from '@/lib/types'

// Filter types for context-specific hardware dialogs
export type HardwareFilter = 'all' | 'storage' | 'cdrom' | 'network' | 'graphics' | 'additional'

interface AddHardwareDialogProps {
  vm: VM
  open: boolean
  onOpenChange: (open: boolean) => void
  filter?: HardwareFilter
}

interface HardwareCategory {
  name: string
  devices: HardwareDevice[]
}

interface HardwareDevice {
  id: string
  name: string
  description: string
  icon: React.ElementType
  available: boolean
  comingSoon?: boolean
}

const hardwareCategories: HardwareCategory[] = [
  {
    name: 'Storage',
    devices: [
      {
        id: 'disk',
        name: 'Storage',
        description: 'Add a new disk image or select existing storage',
        icon: HardDrive,
        available: true,
        comingSoon: false,
      },
      {
        id: 'cdrom',
        name: 'CD/DVD',
        description: 'Mount an ISO image as a virtual CD-ROM drive',
        icon: Disc,
        available: true,
        comingSoon: false,
      },
      {
        id: 'filesystem',
        name: 'Shared Folder',
        description: 'Share a host folder with the guest using virtio-9p or virtiofs',
        icon: FolderOpen,
        available: true,
        comingSoon: false,
      },
    ],
  },
  {
    name: 'Networking',
    devices: [
      {
        id: 'network',
        name: 'Network Interface',
        description: 'Add a virtual network adapter',
        icon: Network,
        available: true,
        comingSoon: false,
      },
    ],
  },
  {
    name: 'Display',
    devices: [
      {
        id: 'graphics',
        name: 'Graphics',
        description: 'Add VNC or SPICE graphics device',
        icon: Monitor,
        available: true,
        comingSoon: false,
      },
      {
        id: 'video',
        name: 'Video',
        description: 'Add a virtual video card',
        icon: Video,
        available: true,
        comingSoon: false,
      },
    ],
  },
  {
    name: 'Audio & Input',
    devices: [
      {
        id: 'sound',
        name: 'Sound',
        description: 'Add a virtual sound card (ich9, ac97, etc.)',
        icon: Volume2,
        available: true,
        comingSoon: false,
      },
      {
        id: 'input',
        name: 'Input',
        description: 'Add keyboard, mouse, or tablet device',
        icon: Keyboard,
        available: true,
        comingSoon: false,
      },
    ],
  },
  {
    name: 'Controllers',
    devices: [
      {
        id: 'usb-controller',
        name: 'USB Controller',
        description: 'Add USB 1.1, 2.0, or 3.0 controller',
        icon: Usb,
        available: true,
        comingSoon: false,
      },
      {
        id: 'scsi-controller',
        name: 'SCSI Controller',
        description: 'Add a SCSI storage controller',
        icon: Cpu,
        available: true,
        comingSoon: false,
      },
    ],
  },
  {
    name: 'Passthrough',
    devices: [
      {
        id: 'pci',
        name: 'PCI Host Device',
        description: 'Pass through a PCI device (GPU, NIC, etc.) to the VM',
        icon: Gamepad2,
        available: true,
        comingSoon: false,
      },
      {
        id: 'usb-host',
        name: 'USB Host Device',
        description: 'Pass through a USB device from the host',
        icon: Usb,
        available: true,
        comingSoon: false,
      },
      {
        id: 'mdev',
        name: 'MDEV Host Device',
        description: 'Pass through a mediated device (vGPU)',
        icon: Video,
        available: true,
        comingSoon: false,
      },
    ],
  },
  {
    name: 'Serial & Parallel',
    devices: [
      {
        id: 'serial',
        name: 'Serial Port',
        description: 'Add a virtual serial port',
        icon: Unplug,
        available: true,
        comingSoon: false,
      },
      {
        id: 'parallel',
        name: 'Parallel Port',
        description: 'Add a parallel port device (legacy)',
        icon: Cable,
        available: true,
        comingSoon: false,
      },
      {
        id: 'console',
        name: 'Console',
        description: 'Add a virtio console device',
        icon: FileText,
        available: true,
        comingSoon: false,
      },
      {
        id: 'channel',
        name: 'Channel',
        description: 'Add QEMU guest agent or SPICE channel',
        icon: Radio,
        available: true,
        comingSoon: false,
      },
    ],
  },
  {
    name: 'Security & Other',
    devices: [
      {
        id: 'tpm',
        name: 'TPM',
        description: 'Add Trusted Platform Module',
        icon: ShieldCheck,
        available: true,
        comingSoon: false,
      },
      {
        id: 'rng',
        name: 'RNG',
        description: 'Add random number generator device (virtio-rng)',
        icon: Dice1,
        available: true,
        comingSoon: false,
      },
      {
        id: 'watchdog',
        name: 'Watchdog',
        description: 'Add hardware watchdog timer for VM health monitoring',
        icon: AlertTriangle,
        available: true,
        comingSoon: false,
      },
      {
        id: 'panic',
        name: 'Panic Notifier',
        description: 'Notify the host when the guest kernel panics',
        icon: AlertTriangle,
        available: true,
        comingSoon: false,
      },
      {
        id: 'vsock',
        name: 'VirtIO VSOCK',
        description: 'Fast guest-host communication without network configuration',
        icon: Radio,
        available: true,
        comingSoon: false,
      },
      {
        id: 'smartcard',
        name: 'Smartcard',
        description: 'Add smartcard reader device',
        icon: CreditCard,
        available: true,
        comingSoon: false,
      },
    ],
  },
]

export function AddHardwareDialog({ vm, open, onOpenChange, filter = 'all' }: AddHardwareDialogProps) {
  const [selectedDevice, setSelectedDevice] = useState<string | null>(null)

  // Filter categories based on the filter prop
  const filteredCategories = hardwareCategories.filter(category => {
    if (filter === 'all') return true
    if (filter === 'storage') return category.name === 'Storage' && category.devices.some(d => d.id === 'disk' || d.id === 'filesystem')
    if (filter === 'cdrom') return category.name === 'Storage' && category.devices.some(d => d.id === 'cdrom')
    if (filter === 'network') return category.name === 'Networking'
    if (filter === 'graphics') return category.name === 'Display'
    if (filter === 'additional') return !['Storage', 'Networking'].includes(category.name)
    return true
  }).map(category => {
    // Further filter devices within categories
    if (filter === 'storage') {
      return { ...category, devices: category.devices.filter(d => d.id === 'disk' || d.id === 'filesystem') }
    }
    if (filter === 'cdrom') {
      return { ...category, devices: category.devices.filter(d => d.id === 'cdrom') }
    }
    return category
  })

  // Get dialog title based on filter
  const getDialogTitle = () => {
    switch (filter) {
      case 'storage': return 'Add Storage'
      case 'cdrom': return 'Add CD/DVD Drive'
      case 'network': return 'Add Network Interface'
      case 'graphics': return 'Add Display Device'
      case 'additional': return 'Add Hardware'
      default: return 'Add Hardware'
    }
  }

  // Auto-select if only one device type
  const allDevices = filteredCategories.flatMap(c => c.devices)
  const autoSelect = allDevices.length === 1 ? allDevices[0].id : null
  const effectiveSelectedDevice = selectedDevice || autoSelect

  const [networkConfig, setNetworkConfig] = useState({
    network: 'default',
    model: 'virtio',
    macAddress: '',
  })
  const [diskConfig, setDiskConfig] = useState({
    diskPath: '',
    deviceTarget: 'vdb',
    busType: 'virtio',
  })
  const [cdromConfig, setCdromConfig] = useState({
    isoPath: '',
  })
  const [soundConfig, setSoundConfig] = useState({
    model: 'ich9',
  })
  const [inputConfig, setInputConfig] = useState({
    deviceType: 'tablet',
    bus: 'usb',
  })
  const [pciConfig, setPciConfig] = useState({
    selectedDevice: '',
    managed: true,
  })
  const [rngConfig, setRngConfig] = useState({
    backend: '/dev/urandom',
  })
  const [watchdogConfig, setWatchdogConfig] = useState({
    model: 'i6300esb',
    action: 'reset',
  })
  const [usbConfig, setUsbConfig] = useState({
    vendorId: '',
    productId: '',
  })
  const [channelConfig, setChannelConfig] = useState({
    channelType: 'qemu-ga',
  })
  const [filesystemConfig, setFilesystemConfig] = useState({
    sourcePath: '',
    targetMount: '',
    fsType: 'virtio-9p',
    readonly: false,
  })
  const [graphicsConfig, setGraphicsConfig] = useState({
    type: 'spice',
    listenAddress: '127.0.0.1',
    port: -1, // auto
  })
  const [videoConfig, setVideoConfig] = useState({
    model: 'virtio',
    vram: 65536,
    heads: 1,
    acceleration3d: false,
  })
  const [mdevConfig, setMdevConfig] = useState({
    mdevUuid: '',
  })
  const [serialConfig, setSerialConfig] = useState({
    portType: 'pty',
    targetPort: 0,
  })
  const [consoleConfig, setConsoleConfig] = useState({
    targetPort: 0,
    targetType: 'virtio',
  })
  const [tpmConfig, setTpmConfig] = useState({
    model: 'tpm-crb',
    version: '2.0',
  })
  const [usbControllerConfig, setUsbControllerConfig] = useState({
    model: 'qemu-xhci',
  })
  const [scsiControllerConfig, setScsiControllerConfig] = useState({
    model: 'virtio-scsi',
  })
  const [panicConfig, setPanicConfig] = useState({
    model: 'isa',
  })
  const [vsockConfig, setVsockConfig] = useState({
    cid: 3, // CID 0, 1, 2 are reserved
  })
  const [parallelConfig, setParallelConfig] = useState({
    targetPort: 0,
  })
  const [smartcardConfig, setSmartcardConfig] = useState({
    mode: 'passthrough',
  })

  const queryClient = useQueryClient()

  // Fetch available networks
  const { data: networks = [] } = useQuery({
    queryKey: ['networks'],
    queryFn: () => api.getNetworks(),
  })

  // Fetch PCI devices when the PCI option is selected
  const { data: pciDevices = [], isLoading: pciLoading } = useQuery({
    queryKey: ['pciDevices'],
    queryFn: () => api.listPciDevices(),
    enabled: selectedDevice === 'pci',
  })

  // Fetch IOMMU status
  const { data: iommuStatus } = useQuery({
    queryKey: ['iommuStatus'],
    queryFn: () => api.checkIommuStatus(),
    enabled: selectedDevice === 'pci',
  })

  // Fetch USB devices when USB option is selected
  const { data: usbDevices = [], isLoading: usbLoading } = useQuery({
    queryKey: ['usbDevices'],
    queryFn: () => api.listUsbDevices(),
    enabled: selectedDevice === 'usb-host',
  })

  // Fetch MDEV status and devices when MDEV option is selected
  const { data: mdevStatus } = useQuery({
    queryKey: ['mdevStatus'],
    queryFn: () => api.checkMdevStatus(),
    enabled: selectedDevice === 'mdev',
  })

  const { data: mdevDevices = [], isLoading: mdevLoading } = useQuery({
    queryKey: ['mdevDevices'],
    queryFn: () => api.listMdevDevices(),
    enabled: selectedDevice === 'mdev',
  })

  const { data: mdevTypes = [] } = useQuery({
    queryKey: ['mdevTypes'],
    queryFn: () => api.listMdevTypes(),
    enabled: selectedDevice === 'mdev',
  })

  // Mutation for attaching network interface
  const attachInterfaceMutation = useMutation({
    mutationFn: () => api.attachInterface(
      vm.id,
      networkConfig.network,
      networkConfig.model,
      networkConfig.macAddress || undefined
    ),
    onSuccess: (mac) => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success(`Network interface added with MAC ${mac}`)
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to add network interface: ${error}`)
    },
  })

  // Mutation for attaching disk
  const attachDiskMutation = useMutation({
    mutationFn: () => api.attachDisk(
      vm.id,
      diskConfig.diskPath,
      diskConfig.deviceTarget,
      diskConfig.busType
    ),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success(`Disk attached as ${diskConfig.deviceTarget}`)
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to attach disk: ${error}`)
    },
  })

  // Mutation for mounting ISO to CD-ROM
  const mountIsoMutation = useMutation({
    mutationFn: () => api.mountIso(vm.id, cdromConfig.isoPath),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success('ISO mounted successfully')
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to mount ISO: ${error}`)
    },
  })

  // Mutation for attaching sound device
  const attachSoundMutation = useMutation({
    mutationFn: () => api.attachSound(vm.id, soundConfig.model),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success(`Sound device (${soundConfig.model}) added successfully`)
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to add sound device: ${error}`)
    },
  })

  // Mutation for attaching input device
  const attachInputMutation = useMutation({
    mutationFn: () => api.attachInput(vm.id, inputConfig.deviceType, inputConfig.bus),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success(`Input device (${inputConfig.deviceType}) added successfully`)
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to add input device: ${error}`)
    },
  })

  // Mutation for attaching PCI device
  const attachPciMutation = useMutation({
    mutationFn: () => api.attachPciDevice(vm.id, pciConfig.selectedDevice, pciConfig.managed),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      queryClient.invalidateQueries({ queryKey: ['pciDevices'] })
      toast.success(`PCI device ${pciConfig.selectedDevice} attached successfully`)
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to attach PCI device: ${error}`)
    },
  })

  // Mutation for binding PCI device to VFIO
  const bindToVfioMutation = useMutation({
    mutationFn: (pciAddress: string) => api.bindToVfio(pciAddress),
    onSuccess: (_, pciAddress) => {
      queryClient.invalidateQueries({ queryKey: ['pciDevices'] })
      queryClient.invalidateQueries({ queryKey: ['vfioStatus', pciAddress] })
      toast.success(`Device ${pciAddress} bound to vfio-pci driver`)
    },
    onError: (error) => {
      toast.error(`Failed to bind to VFIO: ${error}`)
    },
  })

  // Mutation for unbinding PCI device from VFIO
  const unbindFromVfioMutation = useMutation({
    mutationFn: (pciAddress: string) => api.unbindFromVfio(pciAddress),
    onSuccess: (_, pciAddress) => {
      queryClient.invalidateQueries({ queryKey: ['pciDevices'] })
      queryClient.invalidateQueries({ queryKey: ['vfioStatus', pciAddress] })
      toast.success(`Device ${pciAddress} unbound from vfio-pci driver`)
    },
    onError: (error) => {
      toast.error(`Failed to unbind from VFIO: ${error}`)
    },
  })

  // Mutation for attaching RNG device
  const attachRngMutation = useMutation({
    mutationFn: () => api.attachRng(vm.id, rngConfig.backend),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success('RNG device attached successfully')
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to attach RNG device: ${error}`)
    },
  })

  // Mutation for attaching Watchdog device
  const attachWatchdogMutation = useMutation({
    mutationFn: () => api.attachWatchdog(vm.id, watchdogConfig.model, watchdogConfig.action),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success(`Watchdog device (${watchdogConfig.model}) attached successfully`)
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to attach watchdog device: ${error}`)
    },
  })

  // Mutation for attaching USB device
  const attachUsbMutation = useMutation({
    mutationFn: () => api.attachUsbDevice(vm.id, usbConfig.vendorId, usbConfig.productId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      queryClient.invalidateQueries({ queryKey: ['usbDevices'] })
      toast.success(`USB device attached successfully`)
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to attach USB device: ${error}`)
    },
  })

  // Mutation for attaching Channel device
  const attachChannelMutation = useMutation({
    mutationFn: () => api.attachChannel(vm.id, channelConfig.channelType),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      const channelName = channelConfig.channelType === 'qemu-ga' ? 'QEMU Guest Agent' : 'Spice'
      toast.success(`${channelName} channel attached successfully`)
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to attach channel: ${error}`)
    },
  })

  // Mutation for attaching Filesystem share
  const attachFilesystemMutation = useMutation({
    mutationFn: () => api.attachFilesystem(
      vm.id,
      filesystemConfig.sourcePath,
      filesystemConfig.targetMount,
      filesystemConfig.fsType,
      filesystemConfig.readonly
    ),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success(`Shared folder mounted as ${filesystemConfig.targetMount}`)
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to attach filesystem: ${error}`)
    },
  })

  // Mutation for attaching Graphics device
  const attachGraphicsMutation = useMutation({
    mutationFn: () => api.attachGraphics(
      vm.id,
      graphicsConfig.type,
      graphicsConfig.listenAddress,
      graphicsConfig.port === -1 ? undefined : graphicsConfig.port
    ),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success(`${graphicsConfig.type.toUpperCase()} graphics device added`)
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to attach graphics: ${error}`)
    },
  })

  // Mutation for attaching Video device
  const attachVideoMutation = useMutation({
    mutationFn: () => api.attachVideo(
      vm.id,
      videoConfig.model,
      videoConfig.vram,
      videoConfig.heads,
      videoConfig.acceleration3d
    ),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success(`${videoConfig.model} video device added`)
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to attach video: ${error}`)
    },
  })

  // Mutation for attaching MDEV device
  const attachMdevMutation = useMutation({
    mutationFn: () => api.attachMdev(vm.id, mdevConfig.mdevUuid),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      queryClient.invalidateQueries({ queryKey: ['mdevDevices'] })
      toast.success('MDEV (vGPU) device attached')
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to attach MDEV: ${error}`)
    },
  })

  // Mutation for attaching Serial port
  const attachSerialMutation = useMutation({
    mutationFn: () => api.attachSerial(vm.id, serialConfig.portType, serialConfig.targetPort),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success(`Serial port added (${serialConfig.portType})`)
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to add serial port: ${error}`)
    },
  })

  // Mutation for attaching Console device
  const attachConsoleMutation = useMutation({
    mutationFn: () => api.attachConsole(vm.id, consoleConfig.targetPort, consoleConfig.targetType),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success(`${consoleConfig.targetType} console added`)
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to add console: ${error}`)
    },
  })

  // Mutation for attaching TPM device
  const attachTpmMutation = useMutation({
    mutationFn: () => api.attachTpm(vm.id, tpmConfig.model, tpmConfig.version),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success(`TPM ${tpmConfig.version} device added`)
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to add TPM: ${error}`)
    },
  })

  // Mutation for attaching USB Controller
  const attachUsbControllerMutation = useMutation({
    mutationFn: () => api.attachUsbController(vm.id, usbControllerConfig.model),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success(`USB controller (${usbControllerConfig.model}) added`)
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to add USB controller: ${error}`)
    },
  })

  // Mutation for attaching SCSI Controller
  const attachScsiControllerMutation = useMutation({
    mutationFn: () => api.attachScsiController(vm.id, scsiControllerConfig.model),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success(`SCSI controller (${scsiControllerConfig.model}) added`)
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to add SCSI controller: ${error}`)
    },
  })

  // Mutation for attaching Panic Notifier
  const attachPanicMutation = useMutation({
    mutationFn: () => api.attachPanicNotifier(vm.id, panicConfig.model),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success(`Panic notifier (${panicConfig.model}) added`)
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to add panic notifier: ${error}`)
    },
  })

  // Mutation for attaching VirtIO VSOCK
  const attachVsockMutation = useMutation({
    mutationFn: () => api.attachVsock(vm.id, vsockConfig.cid),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success(`VirtIO VSOCK (CID: ${vsockConfig.cid}) added`)
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to add VSOCK: ${error}`)
    },
  })

  // Mutation for attaching Parallel Port
  const attachParallelMutation = useMutation({
    mutationFn: () => api.attachParallel(vm.id, parallelConfig.targetPort),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success(`Parallel port ${parallelConfig.targetPort} added`)
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to add parallel port: ${error}`)
    },
  })

  // Mutation for attaching Smartcard
  const attachSmartcardMutation = useMutation({
    mutationFn: () => api.attachSmartcard(vm.id, smartcardConfig.mode),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vms'] })
      queryClient.invalidateQueries({ queryKey: ['vm', vm.id] })
      toast.success(`Smartcard reader (${smartcardConfig.mode}) added`)
      onOpenChange(false)
      resetState()
    },
    onError: (error) => {
      toast.error(`Failed to add smartcard: ${error}`)
    },
  })

  const resetState = () => {
    setSelectedDevice(null)
    setNetworkConfig({
      network: 'default',
      model: 'virtio',
      macAddress: '',
    })
    setDiskConfig({
      diskPath: '',
      deviceTarget: 'vdb',
      busType: 'virtio',
    })
    setCdromConfig({
      isoPath: '',
    })
    setSoundConfig({
      model: 'ich9',
    })
    setInputConfig({
      deviceType: 'tablet',
      bus: 'usb',
    })
    setPciConfig({
      selectedDevice: '',
      managed: true,
    })
    setRngConfig({
      backend: '/dev/urandom',
    })
    setWatchdogConfig({
      model: 'i6300esb',
      action: 'reset',
    })
    setUsbConfig({
      vendorId: '',
      productId: '',
    })
    setChannelConfig({
      channelType: 'qemu-ga',
    })
    setFilesystemConfig({
      sourcePath: '',
      targetMount: '',
      fsType: 'virtio-9p',
      readonly: false,
    })
    setGraphicsConfig({
      type: 'spice',
      listenAddress: '127.0.0.1',
      port: -1,
    })
    setVideoConfig({
      model: 'virtio',
      vram: 65536,
      heads: 1,
      acceleration3d: false,
    })
    setMdevConfig({
      mdevUuid: '',
    })
    setSerialConfig({
      portType: 'pty',
      targetPort: 0,
    })
    setConsoleConfig({
      targetPort: 0,
      targetType: 'virtio',
    })
    setTpmConfig({
      model: 'tpm-crb',
      version: '2.0',
    })
    setUsbControllerConfig({
      model: 'qemu-xhci',
    })
    setScsiControllerConfig({
      model: 'virtio-scsi',
    })
    setPanicConfig({
      model: 'isa',
    })
    setVsockConfig({
      cid: 3,
    })
    setParallelConfig({
      targetPort: 0,
    })
    setSmartcardConfig({
      mode: 'passthrough',
    })
  }

  const handleAdd = () => {
    if (!selectedDevice) return

    if (effectiveSelectedDevice === 'network') {
      attachInterfaceMutation.mutate()
    } else if (effectiveSelectedDevice === 'disk') {
      if (!diskConfig.diskPath.trim()) {
        toast.error('Please enter the disk path')
        return
      }
      attachDiskMutation.mutate()
    } else if (effectiveSelectedDevice === 'cdrom') {
      if (!cdromConfig.isoPath.trim()) {
        toast.error('Please enter the ISO path')
        return
      }
      mountIsoMutation.mutate()
    } else if (effectiveSelectedDevice === 'sound') {
      attachSoundMutation.mutate()
    } else if (effectiveSelectedDevice === 'input') {
      attachInputMutation.mutate()
    } else if (effectiveSelectedDevice === 'pci') {
      if (!pciConfig.selectedDevice) {
        toast.error('Please select a PCI device')
        return
      }
      attachPciMutation.mutate()
    } else if (effectiveSelectedDevice === 'rng') {
      attachRngMutation.mutate()
    } else if (effectiveSelectedDevice === 'watchdog') {
      attachWatchdogMutation.mutate()
    } else if (effectiveSelectedDevice === 'usb-host') {
      if (!usbConfig.vendorId || !usbConfig.productId) {
        toast.error('Please select a USB device')
        return
      }
      attachUsbMutation.mutate()
    } else if (effectiveSelectedDevice === 'channel') {
      attachChannelMutation.mutate()
    } else if (effectiveSelectedDevice === 'filesystem') {
      if (!filesystemConfig.sourcePath.trim() || !filesystemConfig.targetMount.trim()) {
        toast.error('Please enter both source path and mount target')
        return
      }
      attachFilesystemMutation.mutate()
    } else if (effectiveSelectedDevice === 'graphics') {
      attachGraphicsMutation.mutate()
    } else if (effectiveSelectedDevice === 'video') {
      attachVideoMutation.mutate()
    } else if (effectiveSelectedDevice === 'mdev') {
      if (!mdevConfig.mdevUuid) {
        toast.error('Please select an MDEV device')
        return
      }
      attachMdevMutation.mutate()
    } else if (effectiveSelectedDevice === 'serial') {
      attachSerialMutation.mutate()
    } else if (effectiveSelectedDevice === 'console') {
      attachConsoleMutation.mutate()
    } else if (effectiveSelectedDevice === 'tpm') {
      attachTpmMutation.mutate()
    } else if (effectiveSelectedDevice === 'usb-controller') {
      attachUsbControllerMutation.mutate()
    } else if (effectiveSelectedDevice === 'scsi-controller') {
      attachScsiControllerMutation.mutate()
    } else if (effectiveSelectedDevice === 'panic') {
      attachPanicMutation.mutate()
    } else if (effectiveSelectedDevice === 'vsock') {
      attachVsockMutation.mutate()
    } else if (effectiveSelectedDevice === 'parallel') {
      attachParallelMutation.mutate()
    } else if (effectiveSelectedDevice === 'smartcard') {
      attachSmartcardMutation.mutate()
    } else {
      toast.info(`Adding ${selectedDevice} device - Coming soon!`)
      onOpenChange(false)
      resetState()
    }
  }

  const handleCancel = () => {
    onOpenChange(false)
    resetState()
  }

  const isPending = attachInterfaceMutation.isPending || attachDiskMutation.isPending ||
    mountIsoMutation.isPending || attachSoundMutation.isPending || attachInputMutation.isPending ||
    attachPciMutation.isPending || attachRngMutation.isPending || attachWatchdogMutation.isPending ||
    attachUsbMutation.isPending || attachChannelMutation.isPending || attachFilesystemMutation.isPending ||
    attachGraphicsMutation.isPending || attachVideoMutation.isPending || attachMdevMutation.isPending ||
    attachSerialMutation.isPending || attachConsoleMutation.isPending || attachTpmMutation.isPending ||
    attachUsbControllerMutation.isPending || attachScsiControllerMutation.isPending ||
    attachPanicMutation.isPending || attachVsockMutation.isPending || attachParallelMutation.isPending ||
    attachSmartcardMutation.isPending

  const getSelectedDeviceInfo = () => {
    for (const category of hardwareCategories) {
      const device = category.devices.find(d => d.id === effectiveSelectedDevice)
      if (device) return device
    }
    return null
  }

  const selectedDeviceInfo = getSelectedDeviceInfo()

  // For single-device filters, skip the device list UI
  const showDeviceList = filteredCategories.length > 1 || allDevices.length > 1

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className={showDeviceList ? "max-w-3xl max-h-[80vh]" : "max-w-md"}>
        <DialogHeader>
          <DialogTitle>{getDialogTitle()} to {vm.name}</DialogTitle>
          {showDeviceList && (
            <DialogDescription>
              Select a hardware device type to add to this virtual machine
            </DialogDescription>
          )}
        </DialogHeader>

        <div className={showDeviceList ? "flex gap-4 h-[400px]" : ""}>
          {/* Left side: Device categories and list - only show if multiple options */}
          {showDeviceList && (
          <ScrollArea className="flex-1 border rounded-lg">
            <div className="p-2 space-y-4">
              {filteredCategories.map((category) => (
                <div key={category.name}>
                  <h3 className="text-xs font-semibold text-muted-foreground uppercase tracking-wide px-2 mb-1">
                    {category.name}
                  </h3>
                  <div className="space-y-0.5">
                    {category.devices.map((device) => {
                      const Icon = device.icon
                      return (
                        <button
                          key={device.id}
                          onClick={() => setSelectedDevice(device.id)}
                          className={cn(
                            'w-full flex items-center gap-3 px-2 py-2 rounded-md text-left transition-colors',
                            'hover:bg-muted',
                            effectiveSelectedDevice === device.id && 'bg-primary/10 border border-primary/20'
                          )}
                        >
                          <Icon className="w-4 h-4 text-muted-foreground flex-shrink-0" />
                          <span className="text-sm">{device.name}</span>
                          {device.comingSoon && (
                            <span className="ml-auto text-xs text-muted-foreground">Soon</span>
                          )}
                        </button>
                      )
                    })}
                  </div>
                </div>
              ))}
            </div>
          </ScrollArea>
          )}

          {/* Right side: Device details */}
          <div className={showDeviceList ? "w-64 border rounded-lg p-4" : "space-y-4"}>
            {selectedDeviceInfo ? (
              <div className="space-y-4">
                {showDeviceList && (
                <div className="flex items-center gap-3">
                  <div className="p-2 rounded-lg bg-muted">
                    <selectedDeviceInfo.icon className="w-6 h-6" />
                  </div>
                  <div>
                    <h3 className="font-semibold">{selectedDeviceInfo.name}</h3>
                    {selectedDeviceInfo.comingSoon && (
                      <span className="text-xs text-orange-500">Coming Soon</span>
                    )}
                  </div>
                </div>
                )}

                {effectiveSelectedDevice === 'network' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    <div className="space-y-1.5">
                      <Label className="text-xs">Network Source</Label>
                      <Select
                        value={networkConfig.network}
                        onValueChange={(v) => setNetworkConfig({ ...networkConfig, network: v })}
                      >
                        <SelectTrigger className="h-8 text-sm">
                          <SelectValue placeholder="Select network" />
                        </SelectTrigger>
                        <SelectContent>
                          {networks.map((net) => (
                            <SelectItem key={net.name} value={net.name}>
                              {net.name}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="space-y-1.5">
                      <Label className="text-xs">Device Model</Label>
                      <Select
                        value={networkConfig.model}
                        onValueChange={(v) => setNetworkConfig({ ...networkConfig, model: v })}
                      >
                        <SelectTrigger className="h-8 text-sm">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="virtio">VirtIO (Recommended)</SelectItem>
                          <SelectItem value="e1000">Intel e1000</SelectItem>
                          <SelectItem value="e1000e">Intel e1000e</SelectItem>
                          <SelectItem value="rtl8139">Realtek RTL8139</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="space-y-1.5">
                      <Label className="text-xs">MAC Address (optional)</Label>
                      <Input
                        className="h-8 text-sm font-mono"
                        placeholder="Auto-generate"
                        value={networkConfig.macAddress}
                        onChange={(e) => setNetworkConfig({ ...networkConfig, macAddress: e.target.value })}
                      />
                      <p className="text-xs text-muted-foreground">
                        Leave empty to auto-generate
                      </p>
                    </div>
                  </div>
                ) : effectiveSelectedDevice === 'disk' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    <div className="space-y-1.5">
                      <Label className="text-xs">Disk Path</Label>
                      <Input
                        className="h-8 text-sm font-mono"
                        placeholder="/var/lib/libvirt/images/disk.qcow2"
                        value={diskConfig.diskPath}
                        onChange={(e) => setDiskConfig({ ...diskConfig, diskPath: e.target.value })}
                      />
                      <p className="text-xs text-muted-foreground">
                        Full path to existing qcow2 or raw disk image
                      </p>
                    </div>

                    <div className="space-y-1.5">
                      <Label className="text-xs">Device Target</Label>
                      <Select
                        value={diskConfig.deviceTarget}
                        onValueChange={(v) => setDiskConfig({ ...diskConfig, deviceTarget: v })}
                      >
                        <SelectTrigger className="h-8 text-sm">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="vdb">vdb (VirtIO)</SelectItem>
                          <SelectItem value="vdc">vdc (VirtIO)</SelectItem>
                          <SelectItem value="vdd">vdd (VirtIO)</SelectItem>
                          <SelectItem value="sdb">sdb (SATA/SCSI)</SelectItem>
                          <SelectItem value="sdc">sdc (SATA/SCSI)</SelectItem>
                          <SelectItem value="hdb">hdb (IDE)</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="space-y-1.5">
                      <Label className="text-xs">Bus Type</Label>
                      <Select
                        value={diskConfig.busType}
                        onValueChange={(v) => setDiskConfig({ ...diskConfig, busType: v })}
                      >
                        <SelectTrigger className="h-8 text-sm">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="virtio">VirtIO (Recommended)</SelectItem>
                          <SelectItem value="nvme">NVMe (High Performance)</SelectItem>
                          <SelectItem value="sata">SATA</SelectItem>
                          <SelectItem value="scsi">SCSI</SelectItem>
                          <SelectItem value="ide">IDE</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </div>
                ) : effectiveSelectedDevice === 'cdrom' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    <div className="space-y-1.5">
                      <Label className="text-xs">ISO Path</Label>
                      <div className="flex gap-2">
                        <Input
                          className="h-8 text-sm font-mono flex-1"
                          placeholder="/var/lib/libvirt/images/ubuntu.iso"
                          value={cdromConfig.isoPath}
                          onChange={(e) => setCdromConfig({ ...cdromConfig, isoPath: e.target.value })}
                        />
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          className="h-8 px-3"
                          onClick={async () => {
                            try {
                              const home = await homeDir()
                              const selected = await openFileDialog({
                                title: 'Select ISO Image',
                                defaultPath: `${home}/Downloads`,
                                filters: [{ name: 'ISO Images', extensions: ['iso', 'img'] }],
                              })
                              if (selected && typeof selected === 'string') {
                                setCdromConfig({ ...cdromConfig, isoPath: selected })
                              }
                            } catch (err) {
                              console.error('Failed to open file dialog:', err)
                            }
                          }}
                        >
                          Browse
                        </Button>
                      </div>
                      <p className="text-xs text-muted-foreground">
                        Full path to ISO image file
                      </p>
                    </div>

                    <div className="p-3 bg-muted rounded-lg space-y-2">
                      <p className="text-xs text-muted-foreground">
                        The ISO will be mounted to the VM's CD-ROM drive. If the VM already has
                        a CD-ROM, the media will be swapped. Otherwise, a new SATA CD-ROM will be added.
                      </p>
                      {vm.state === 'running' && (
                        <p className="text-xs text-yellow-600">
                          ⚠️ Note: Adding a new CD-ROM to a running VM may fail.
                          Swapping media in an existing CD-ROM works while running.
                        </p>
                      )}
                    </div>
                  </div>
                ) : effectiveSelectedDevice === 'sound' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    <div className="space-y-1.5">
                      <Label className="text-xs">Sound Card Model</Label>
                      <Select
                        value={soundConfig.model}
                        onValueChange={(v) => setSoundConfig({ ...soundConfig, model: v })}
                      >
                        <SelectTrigger className="h-8 text-sm">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="ich9">Intel ICH9 (Recommended)</SelectItem>
                          <SelectItem value="ich6">Intel ICH6</SelectItem>
                          <SelectItem value="ac97">AC97</SelectItem>
                          <SelectItem value="es1370">Ensoniq ES1370</SelectItem>
                          <SelectItem value="sb16">Sound Blaster 16 (Legacy)</SelectItem>
                          <SelectItem value="usb">USB Audio</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="p-3 bg-muted rounded-lg space-y-2">
                      <p className="text-xs text-muted-foreground">
                        <strong>ICH9</strong> - Best for modern guests (Windows 10+, Linux)<br/>
                        <strong>AC97</strong> - Good compatibility with older systems<br/>
                        <strong>SB16</strong> - DOS and Windows 9x support
                      </p>
                      {vm.state === 'running' && (
                        <p className="text-xs text-yellow-600">
                          ⚠️ Sound devices cannot be added to running VMs. Please stop the VM first.
                        </p>
                      )}
                    </div>
                  </div>
                ) : effectiveSelectedDevice === 'input' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    <div className="space-y-1.5">
                      <Label className="text-xs">Device Type</Label>
                      <Select
                        value={inputConfig.deviceType}
                        onValueChange={(v) => setInputConfig({ ...inputConfig, deviceType: v })}
                      >
                        <SelectTrigger className="h-8 text-sm">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="tablet">Tablet (Recommended for Spice)</SelectItem>
                          <SelectItem value="mouse">Mouse</SelectItem>
                          <SelectItem value="keyboard">Keyboard</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="space-y-1.5">
                      <Label className="text-xs">Bus Type</Label>
                      <Select
                        value={inputConfig.bus}
                        onValueChange={(v) => setInputConfig({ ...inputConfig, bus: v })}
                      >
                        <SelectTrigger className="h-8 text-sm">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="usb">USB</SelectItem>
                          <SelectItem value="virtio">VirtIO</SelectItem>
                          <SelectItem value="ps2">PS/2</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="p-3 bg-muted rounded-lg space-y-2">
                      <p className="text-xs text-muted-foreground">
                        <strong>Tablet</strong> - Provides absolute pointer positioning, best for Spice/VNC<br/>
                        <strong>Mouse</strong> - Relative pointer, requires mouse grab<br/>
                        <strong>Keyboard</strong> - Additional USB keyboard
                      </p>
                      {vm.state === 'running' && (
                        <p className="text-xs text-yellow-600">
                          ⚠️ Input devices may not be hotpluggable. Stop the VM if adding fails.
                        </p>
                      )}
                    </div>
                  </div>
                ) : effectiveSelectedDevice === 'pci' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    {/* IOMMU Status Warning */}
                    {iommuStatus && !iommuStatus.enabled && (
                      <div className="p-3 bg-destructive/10 border border-destructive/20 rounded-lg">
                        <p className="text-xs text-destructive flex items-center gap-1">
                          <AlertTriangle className="w-3 h-3" />
                          {iommuStatus.warning || 'IOMMU is not enabled'}
                        </p>
                      </div>
                    )}

                    {iommuStatus?.enabled && (
                      <div className="p-2 bg-green-500/10 border border-green-500/20 rounded-lg">
                        <p className="text-xs text-green-600 flex items-center gap-1">
                          <Check className="w-3 h-3" />
                          {iommuStatus.iommuType || 'IOMMU'} enabled
                        </p>
                      </div>
                    )}

                    <div className="space-y-1.5">
                      <Label className="text-xs">Select PCI Device</Label>
                      {pciLoading ? (
                        <p className="text-xs text-muted-foreground">Loading devices...</p>
                      ) : (
                        <ScrollArea className="h-48 border rounded-md">
                          <div className="p-2 space-y-1">
                            {pciDevices.filter(d => d.passthroughSafe).length === 0 ? (
                              <p className="text-xs text-muted-foreground p-2">
                                No available PCI devices for passthrough.
                                Devices may be in use or not safe for passthrough.
                              </p>
                            ) : (
                              pciDevices
                                .filter(d => d.passthroughSafe)
                                .map((device) => {
                                  const isVfio = device.driver === 'vfio-pci'
                                  const isSelected = pciConfig.selectedDevice === device.address
                                  const isAttached = !!device.attachedToVm

                                  return (
                                    <div
                                      key={device.address}
                                      className={cn(
                                        'p-2 rounded-md text-xs border',
                                        'hover:bg-muted/50 transition-colors',
                                        isSelected && 'bg-primary/10 border-primary/20',
                                        !isSelected && 'border-transparent'
                                      )}
                                    >
                                      <button
                                        onClick={() => !isAttached && setPciConfig({ ...pciConfig, selectedDevice: device.address })}
                                        disabled={isAttached}
                                        className="w-full text-left"
                                      >
                                        <div className="flex items-center justify-between">
                                          <span className="font-mono">{device.address}</span>
                                          <div className="flex items-center gap-1">
                                            {isVfio ? (
                                              <Badge variant="default" className="text-[10px] bg-green-600">
                                                vfio-pci
                                              </Badge>
                                            ) : device.driver ? (
                                              <Badge variant="secondary" className="text-[10px]">
                                                {device.driver}
                                              </Badge>
                                            ) : (
                                              <Badge variant="outline" className="text-[10px]">
                                                unbound
                                              </Badge>
                                            )}
                                            <Badge variant="outline" className="text-[10px]">
                                              {device.deviceType}
                                            </Badge>
                                          </div>
                                        </div>
                                        <p className="text-muted-foreground truncate">
                                          {device.vendor} {device.deviceName}
                                        </p>
                                        <div className="flex items-center justify-between mt-1">
                                          {device.iommuGroup !== undefined && (
                                            <span className="text-muted-foreground">
                                              IOMMU Group: {device.iommuGroup}
                                            </span>
                                          )}
                                          {isAttached && (
                                            <span className="text-yellow-600">
                                              In use by: {device.attachedToVm}
                                            </span>
                                          )}
                                        </div>
                                      </button>

                                      {/* VFIO Binding Controls */}
                                      {isSelected && !isAttached && (
                                        <div className="mt-2 pt-2 border-t border-border/50 flex items-center justify-between">
                                          <span className="text-muted-foreground">
                                            Driver: {device.driver || 'none'}
                                          </span>
                                          {isVfio ? (
                                            <Button
                                              size="sm"
                                              variant="outline"
                                              className="h-6 text-xs"
                                              onClick={(e) => {
                                                e.stopPropagation()
                                                unbindFromVfioMutation.mutate(device.address)
                                              }}
                                              disabled={unbindFromVfioMutation.isPending}
                                            >
                                              {unbindFromVfioMutation.isPending ? 'Unbinding...' : 'Unbind from VFIO'}
                                            </Button>
                                          ) : (
                                            <Button
                                              size="sm"
                                              variant="default"
                                              className="h-6 text-xs"
                                              onClick={(e) => {
                                                e.stopPropagation()
                                                bindToVfioMutation.mutate(device.address)
                                              }}
                                              disabled={bindToVfioMutation.isPending}
                                            >
                                              {bindToVfioMutation.isPending ? 'Binding...' : 'Bind to VFIO'}
                                            </Button>
                                          )}
                                        </div>
                                      )}
                                    </div>
                                  )
                                })
                            )}
                          </div>
                        </ScrollArea>
                      )}
                    </div>

                    {pciConfig.selectedDevice && (
                      <div className="p-2 bg-blue-500/10 border border-blue-500/20 rounded-lg">
                        <p className="text-xs text-blue-600 flex items-center gap-1">
                          <Info className="w-3 h-3" />
                          Tip: Use "Bind to VFIO" to prepare the device for passthrough. This unbinds it from the host driver.
                        </p>
                      </div>
                    )}

                    {vm.state === 'running' && (
                      <div className="p-3 bg-yellow-500/10 border border-yellow-500/20 rounded-lg">
                        <p className="text-xs text-yellow-600">
                          ⚠️ PCI passthrough requires the VM to be stopped. Hot-adding PCI devices is not supported.
                        </p>
                      </div>
                    )}
                  </div>
                ) : effectiveSelectedDevice === 'rng' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    <p className="text-sm text-muted-foreground">
                      {selectedDeviceInfo.description}
                    </p>

                    <div className="space-y-1.5">
                      <Label className="text-xs">Entropy Source</Label>
                      <Select
                        value={rngConfig.backend}
                        onValueChange={(val) => setRngConfig({ backend: val })}
                      >
                        <SelectTrigger className="h-8 text-xs">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="/dev/urandom">/dev/urandom (Recommended)</SelectItem>
                          <SelectItem value="/dev/random">/dev/random (Blocking)</SelectItem>
                          <SelectItem value="/dev/hwrng">/dev/hwrng (Hardware RNG)</SelectItem>
                        </SelectContent>
                      </Select>
                      <p className="text-[10px] text-muted-foreground">
                        /dev/urandom is recommended for most use cases. /dev/random may block if entropy is low.
                      </p>
                    </div>

                    <div className="p-2 bg-blue-500/10 border border-blue-500/20 rounded-lg">
                      <p className="text-xs text-blue-600 flex items-center gap-1">
                        <Info className="w-3 h-3" />
                        VirtIO RNG provides high-quality entropy to the guest
                      </p>
                    </div>
                  </div>
                ) : effectiveSelectedDevice === 'watchdog' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    <p className="text-sm text-muted-foreground">
                      {selectedDeviceInfo.description}
                    </p>

                    <div className="space-y-1.5">
                      <Label className="text-xs">Watchdog Model</Label>
                      <Select
                        value={watchdogConfig.model}
                        onValueChange={(val) => setWatchdogConfig({ ...watchdogConfig, model: val })}
                      >
                        <SelectTrigger className="h-8 text-xs">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="i6300esb">i6300esb (Intel)</SelectItem>
                          <SelectItem value="ib700">ib700 (iBase)</SelectItem>
                          <SelectItem value="diag288">diag288 (s390x)</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="space-y-1.5">
                      <Label className="text-xs">Watchdog Action</Label>
                      <Select
                        value={watchdogConfig.action}
                        onValueChange={(val) => setWatchdogConfig({ ...watchdogConfig, action: val })}
                      >
                        <SelectTrigger className="h-8 text-xs">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="reset">Reset VM</SelectItem>
                          <SelectItem value="shutdown">Graceful Shutdown</SelectItem>
                          <SelectItem value="poweroff">Power Off</SelectItem>
                          <SelectItem value="pause">Pause VM</SelectItem>
                          <SelectItem value="none">None (Log Only)</SelectItem>
                          <SelectItem value="dump">Dump (Debug)</SelectItem>
                          <SelectItem value="inject-nmi">Inject NMI</SelectItem>
                        </SelectContent>
                      </Select>
                      <p className="text-[10px] text-muted-foreground">
                        Action to take if the guest fails to respond to the watchdog timer.
                      </p>
                    </div>

                    <div className="p-2 bg-yellow-500/10 border border-yellow-500/20 rounded-lg">
                      <p className="text-xs text-yellow-600 flex items-center gap-1">
                        <AlertTriangle className="w-3 h-3" />
                        Guest OS must have watchdog driver support
                      </p>
                    </div>
                  </div>
                ) : effectiveSelectedDevice === 'usb-host' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    <p className="text-sm text-muted-foreground">
                      {selectedDeviceInfo.description}
                    </p>

                    <div className="space-y-1.5">
                      <Label className="text-xs">Select USB Device</Label>
                      {usbLoading ? (
                        <p className="text-xs text-muted-foreground">Loading devices...</p>
                      ) : (
                        <ScrollArea className="h-48 border rounded-md">
                          <div className="p-2 space-y-1">
                            {usbDevices.filter(d => !d.inUse).length === 0 ? (
                              <p className="text-xs text-muted-foreground p-2">
                                No available USB devices found.
                                All devices may be in use or none are connected.
                              </p>
                            ) : (
                              usbDevices
                                .filter(d => !d.inUse)
                                .map((device) => (
                                  <button
                                    key={`${device.vendorId}:${device.productId}`}
                                    onClick={() => setUsbConfig({
                                      vendorId: device.vendorId,
                                      productId: device.productId
                                    })}
                                    className={cn(
                                      'w-full text-left p-2 rounded-md text-xs',
                                      'hover:bg-muted transition-colors',
                                      usbConfig.vendorId === device.vendorId &&
                                      usbConfig.productId === device.productId &&
                                      'bg-primary/10 border border-primary/20'
                                    )}
                                  >
                                    <div className="flex items-center justify-between">
                                      <span className="font-medium">{device.vendorName}</span>
                                      <Badge variant="outline" className="text-[10px]">
                                        {device.vendorId}:{device.productId}
                                      </Badge>
                                    </div>
                                    <p className="text-muted-foreground truncate">
                                      {device.productName || device.description}
                                    </p>
                                    {device.speed && (
                                      <p className="text-muted-foreground text-[10px]">
                                        {device.speed}
                                      </p>
                                    )}
                                  </button>
                                ))
                            )}
                          </div>
                        </ScrollArea>
                      )}
                    </div>

                    {vm.state !== 'running' && (
                      <div className="p-3 bg-yellow-500/10 border border-yellow-500/20 rounded-lg">
                        <p className="text-xs text-yellow-600">
                          ⚠️ USB passthrough requires the VM to be running.
                        </p>
                      </div>
                    )}

                    <div className="p-2 bg-blue-500/10 border border-blue-500/20 rounded-lg">
                      <p className="text-xs text-blue-600 flex items-center gap-1">
                        <Info className="w-3 h-3" />
                        USB devices are passed through by vendor:product ID
                      </p>
                    </div>
                  </div>
                ) : effectiveSelectedDevice === 'channel' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    <p className="text-sm text-muted-foreground">
                      {selectedDeviceInfo.description}
                    </p>

                    <div className="space-y-1.5">
                      <Label className="text-xs">Channel Type</Label>
                      <Select
                        value={channelConfig.channelType}
                        onValueChange={(val) => setChannelConfig({ channelType: val })}
                      >
                        <SelectTrigger className="h-8 text-xs">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="qemu-ga">QEMU Guest Agent</SelectItem>
                          <SelectItem value="spice">Spice Agent</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    {channelConfig.channelType === 'qemu-ga' && (
                      <div className="p-2 bg-blue-500/10 border border-blue-500/20 rounded-lg">
                        <p className="text-xs text-blue-600 flex items-center gap-1">
                          <Info className="w-3 h-3" />
                          Enables host-guest communication for commands, file transfer, and shutdown
                        </p>
                      </div>
                    )}

                    {channelConfig.channelType === 'spice' && (
                      <div className="p-2 bg-blue-500/10 border border-blue-500/20 rounded-lg">
                        <p className="text-xs text-blue-600 flex items-center gap-1">
                          <Info className="w-3 h-3" />
                          Required for Spice clipboard sharing and seamless mouse
                        </p>
                      </div>
                    )}

                    <div className="p-2 bg-yellow-500/10 border border-yellow-500/20 rounded-lg">
                      <p className="text-xs text-yellow-600 flex items-center gap-1">
                        <AlertTriangle className="w-3 h-3" />
                        Guest OS must have the agent software installed
                      </p>
                    </div>
                  </div>
                ) : effectiveSelectedDevice === 'filesystem' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    <p className="text-sm text-muted-foreground">
                      {selectedDeviceInfo.description}
                    </p>

                    <div className="space-y-1.5">
                      <Label className="text-xs">Source Path (Host)</Label>
                      <Input
                        className="h-8 text-xs"
                        placeholder="/path/to/shared/folder"
                        value={filesystemConfig.sourcePath}
                        onChange={(e) => setFilesystemConfig({ ...filesystemConfig, sourcePath: e.target.value })}
                      />
                    </div>

                    <div className="space-y-1.5">
                      <Label className="text-xs">Mount Tag (Guest)</Label>
                      <Input
                        className="h-8 text-xs"
                        placeholder="shared"
                        value={filesystemConfig.targetMount}
                        onChange={(e) => setFilesystemConfig({ ...filesystemConfig, targetMount: e.target.value })}
                      />
                      <p className="text-[10px] text-muted-foreground">
                        Use this tag to mount in guest: mount -t 9p shared /mnt/shared
                      </p>
                    </div>

                    <div className="space-y-1.5">
                      <Label className="text-xs">Filesystem Type</Label>
                      <Select
                        value={filesystemConfig.fsType}
                        onValueChange={(val) => setFilesystemConfig({ ...filesystemConfig, fsType: val })}
                      >
                        <SelectTrigger className="h-8 text-xs">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="virtio-9p">virtio-9p (9P filesystem)</SelectItem>
                          <SelectItem value="virtiofs">virtiofs (faster, requires vhost-user)</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="flex items-center gap-2">
                      <input
                        type="checkbox"
                        id="fs-readonly"
                        checked={filesystemConfig.readonly}
                        onChange={(e) => setFilesystemConfig({ ...filesystemConfig, readonly: e.target.checked })}
                        className="h-3 w-3"
                      />
                      <Label htmlFor="fs-readonly" className="text-xs">Read-only</Label>
                    </div>

                    <div className="p-2 bg-blue-500/10 border border-blue-500/20 rounded-lg">
                      <p className="text-xs text-blue-600 flex items-center gap-1">
                        <Info className="w-3 h-3" />
                        Guest must mount: mount -t 9p {filesystemConfig.targetMount || 'tag'} /mnt/point
                      </p>
                    </div>
                  </div>
                ) : effectiveSelectedDevice === 'graphics' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    <h4 className="text-sm font-medium">Add Graphics Device</h4>
                    <p className="text-xs text-muted-foreground">
                      Add a VNC or SPICE display server for graphical console access.
                    </p>

                    <div className="space-y-1">
                      <Label htmlFor="graphics-type" className="text-xs">Graphics Type</Label>
                      <Select
                        value={graphicsConfig.type}
                        onValueChange={(val) => setGraphicsConfig({ ...graphicsConfig, type: val })}
                      >
                        <SelectTrigger id="graphics-type" className="h-8 text-xs">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="spice">SPICE (recommended)</SelectItem>
                          <SelectItem value="vnc">VNC</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="space-y-1">
                      <Label htmlFor="graphics-listen" className="text-xs">Listen Address</Label>
                      <Select
                        value={graphicsConfig.listenAddress}
                        onValueChange={(val) => setGraphicsConfig({ ...graphicsConfig, listenAddress: val })}
                      >
                        <SelectTrigger id="graphics-listen" className="h-8 text-xs">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="127.0.0.1">Localhost only (127.0.0.1)</SelectItem>
                          <SelectItem value="0.0.0.0">All interfaces (0.0.0.0)</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="p-2 bg-blue-500/10 border border-blue-500/20 rounded-lg">
                      <p className="text-xs text-blue-600 flex items-center gap-1">
                        <Info className="w-3 h-3" />
                        {graphicsConfig.type === 'spice'
                          ? 'SPICE offers better performance, USB redirection, and clipboard sharing.'
                          : 'VNC provides basic remote display with wide compatibility.'}
                      </p>
                    </div>
                  </div>
                ) : effectiveSelectedDevice === 'video' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    <h4 className="text-sm font-medium">Add Video Device</h4>
                    <p className="text-xs text-muted-foreground">
                      Add a virtual video card for graphics rendering in the VM.
                    </p>

                    <div className="space-y-1">
                      <Label htmlFor="video-model" className="text-xs">Video Model</Label>
                      <Select
                        value={videoConfig.model}
                        onValueChange={(val) => setVideoConfig({ ...videoConfig, model: val })}
                      >
                        <SelectTrigger id="video-model" className="h-8 text-xs">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="virtio">VirtIO (best performance)</SelectItem>
                          <SelectItem value="qxl">QXL (for SPICE)</SelectItem>
                          <SelectItem value="vga">VGA (legacy)</SelectItem>
                          <SelectItem value="bochs">Bochs (UEFI compatible)</SelectItem>
                          <SelectItem value="ramfb">ramfb (minimal UEFI)</SelectItem>
                          <SelectItem value="cirrus">Cirrus (legacy)</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="space-y-1">
                      <Label htmlFor="video-vram" className="text-xs">VRAM (KB)</Label>
                      <Select
                        value={videoConfig.vram.toString()}
                        onValueChange={(val) => setVideoConfig({ ...videoConfig, vram: parseInt(val) })}
                      >
                        <SelectTrigger id="video-vram" className="h-8 text-xs">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="16384">16 MB</SelectItem>
                          <SelectItem value="32768">32 MB</SelectItem>
                          <SelectItem value="65536">64 MB</SelectItem>
                          <SelectItem value="131072">128 MB</SelectItem>
                          <SelectItem value="262144">256 MB</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    {videoConfig.model === 'virtio' && (
                      <div className="flex items-center gap-2">
                        <input
                          type="checkbox"
                          id="video-3d"
                          checked={videoConfig.acceleration3d}
                          onChange={(e) => setVideoConfig({ ...videoConfig, acceleration3d: e.target.checked })}
                          className="h-3 w-3"
                        />
                        <Label htmlFor="video-3d" className="text-xs">Enable 3D acceleration (requires virgl)</Label>
                      </div>
                    )}

                    <div className="p-2 bg-blue-500/10 border border-blue-500/20 rounded-lg">
                      <p className="text-xs text-blue-600 flex items-center gap-1">
                        <Info className="w-3 h-3" />
                        {videoConfig.model === 'virtio'
                          ? 'VirtIO GPU offers the best performance with virgl 3D support.'
                          : videoConfig.model === 'qxl'
                          ? 'QXL is optimized for SPICE connections with good 2D performance.'
                          : 'This model provides basic display compatibility.'}
                      </p>
                    </div>
                  </div>
                ) : effectiveSelectedDevice === 'mdev' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    <h4 className="text-sm font-medium">Add MDEV Host Device</h4>
                    <p className="text-xs text-muted-foreground">
                      Pass through a mediated device (vGPU) for GPU virtualization.
                    </p>

                    {!mdevStatus?.supported ? (
                      <div className="p-2 bg-yellow-500/10 border border-yellow-500/20 rounded-lg">
                        <p className="text-xs text-yellow-600 flex items-center gap-1">
                          <AlertTriangle className="w-3 h-3" />
                          {mdevStatus?.message || 'MDEV not supported on this system'}
                        </p>
                      </div>
                    ) : mdevLoading ? (
                      <p className="text-xs text-muted-foreground">Loading MDEV devices...</p>
                    ) : mdevDevices.length === 0 ? (
                      <div className="space-y-2">
                        <div className="p-2 bg-yellow-500/10 border border-yellow-500/20 rounded-lg">
                          <p className="text-xs text-yellow-600 flex items-center gap-1">
                            <AlertTriangle className="w-3 h-3" />
                            No MDEV instances found. Create one from available types.
                          </p>
                        </div>
                        {mdevTypes.length > 0 && (
                          <div className="space-y-1">
                            <Label className="text-xs">Available MDEV Types</Label>
                            <div className="max-h-32 overflow-y-auto border rounded-lg p-1 space-y-1">
                              {mdevTypes.map((type) => (
                                <div key={type.name} className="p-1.5 bg-muted/50 rounded text-xs">
                                  <div className="font-medium">{type.name}</div>
                                  <div className="text-muted-foreground">
                                    {type.description || type.parentName}
                                  </div>
                                  <div className="text-muted-foreground">
                                    Available: {type.availableInstances}/{type.maxInstances}
                                  </div>
                                </div>
                              ))}
                            </div>
                          </div>
                        )}
                      </div>
                    ) : (
                      <div className="space-y-1">
                        <Label htmlFor="mdev-device" className="text-xs">Select MDEV Device</Label>
                        <Select
                          value={mdevConfig.mdevUuid}
                          onValueChange={(val) => setMdevConfig({ mdevUuid: val })}
                        >
                          <SelectTrigger id="mdev-device" className="h-8 text-xs">
                            <SelectValue placeholder="Select MDEV device..." />
                          </SelectTrigger>
                          <SelectContent>
                            {mdevDevices.filter(d => !d.inUse).map((device) => (
                              <SelectItem key={device.uuid} value={device.uuid}>
                                {device.mdevType} ({device.uuid.slice(0, 8)}...)
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      </div>
                    )}

                    <div className="p-2 bg-blue-500/10 border border-blue-500/20 rounded-lg">
                      <p className="text-xs text-blue-600 flex items-center gap-1">
                        <Info className="w-3 h-3" />
                        MDEV allows GPU sharing (Intel GVT-g, NVIDIA vGPU, AMD SR-IOV).
                      </p>
                    </div>
                  </div>
                ) : effectiveSelectedDevice === 'serial' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    <div className="space-y-1.5">
                      <Label className="text-xs">Port Type</Label>
                      <Select
                        value={serialConfig.portType}
                        onValueChange={(v) => setSerialConfig({ ...serialConfig, portType: v })}
                      >
                        <SelectTrigger className="h-8 text-sm">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="pty">PTY (Pseudo-terminal)</SelectItem>
                          <SelectItem value="tcp">TCP</SelectItem>
                          <SelectItem value="unix">Unix Socket</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="space-y-1.5">
                      <Label className="text-xs">Target Port</Label>
                      <Input
                        type="number"
                        min={0}
                        max={3}
                        value={serialConfig.targetPort}
                        onChange={(e) => setSerialConfig({ ...serialConfig, targetPort: parseInt(e.target.value) || 0 })}
                        className="h-8 text-sm"
                      />
                      <p className="text-xs text-muted-foreground">Port 0 is typically /dev/ttyS0</p>
                    </div>

                    <div className="p-3 bg-muted rounded-lg">
                      <p className="text-xs text-muted-foreground">
                        <strong>PTY</strong> - Connects to a pseudo-terminal on the host<br/>
                        <strong>TCP</strong> - Network-accessible serial port<br/>
                        <strong>Unix</strong> - Unix socket for local access
                      </p>
                      {vm.state === 'running' && (
                        <p className="text-xs text-yellow-600 mt-2">
                          ⚠️ Serial ports cannot be added to running VMs. Please stop the VM first.
                        </p>
                      )}
                    </div>
                  </div>
                ) : effectiveSelectedDevice === 'console' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    <div className="space-y-1.5">
                      <Label className="text-xs">Console Type</Label>
                      <Select
                        value={consoleConfig.targetType}
                        onValueChange={(v) => setConsoleConfig({ ...consoleConfig, targetType: v })}
                      >
                        <SelectTrigger className="h-8 text-sm">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="virtio">VirtIO Console</SelectItem>
                          <SelectItem value="serial">Serial Console</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="space-y-1.5">
                      <Label className="text-xs">Target Port</Label>
                      <Input
                        type="number"
                        min={0}
                        max={3}
                        value={consoleConfig.targetPort}
                        onChange={(e) => setConsoleConfig({ ...consoleConfig, targetPort: parseInt(e.target.value) || 0 })}
                        className="h-8 text-sm"
                      />
                    </div>

                    <div className="p-3 bg-muted rounded-lg">
                      <p className="text-xs text-muted-foreground">
                        <strong>VirtIO Console</strong> - High-performance paravirtualized console<br/>
                        <strong>Serial Console</strong> - Traditional serial console emulation
                      </p>
                      {vm.state === 'running' && (
                        <p className="text-xs text-yellow-600 mt-2">
                          ⚠️ Console devices cannot be added to running VMs. Please stop the VM first.
                        </p>
                      )}
                    </div>
                  </div>
                ) : effectiveSelectedDevice === 'tpm' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    <div className="space-y-1.5">
                      <Label className="text-xs">TPM Model</Label>
                      <Select
                        value={tpmConfig.model}
                        onValueChange={(v) => setTpmConfig({ ...tpmConfig, model: v })}
                      >
                        <SelectTrigger className="h-8 text-sm">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="tpm-crb">CRB (Recommended)</SelectItem>
                          <SelectItem value="tpm-tis">TIS (Legacy)</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="space-y-1.5">
                      <Label className="text-xs">TPM Version</Label>
                      <Select
                        value={tpmConfig.version}
                        onValueChange={(v) => setTpmConfig({ ...tpmConfig, version: v })}
                      >
                        <SelectTrigger className="h-8 text-sm">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="2.0">TPM 2.0 (Recommended)</SelectItem>
                          <SelectItem value="1.2">TPM 1.2 (Legacy)</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="p-3 bg-muted rounded-lg">
                      <p className="text-xs text-muted-foreground">
                        <strong>TPM 2.0 + CRB</strong> - Required for Windows 11 and modern security features<br/>
                        <strong>TPM 1.2 + TIS</strong> - For older Windows versions
                      </p>
                      {vm.state === 'running' && (
                        <p className="text-xs text-yellow-600 mt-2">
                          ⚠️ TPM cannot be added to running VMs. Please stop the VM first.
                        </p>
                      )}
                      {vm.firmware === 'bios' && (
                        <p className="text-xs text-orange-600 mt-2">
                          ⚠️ TPM is recommended with UEFI firmware for best compatibility.
                        </p>
                      )}
                    </div>
                  </div>
                ) : effectiveSelectedDevice === 'usb-controller' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    <div className="space-y-1.5">
                      <Label className="text-xs">USB Controller Model</Label>
                      <Select
                        value={usbControllerConfig.model}
                        onValueChange={(v) => setUsbControllerConfig({ model: v })}
                      >
                        <SelectTrigger className="h-8 text-sm">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="qemu-xhci">QEMU xHCI (USB 3.0)</SelectItem>
                          <SelectItem value="nec-xhci">NEC xHCI (USB 3.0)</SelectItem>
                          <SelectItem value="ich9-ehci1">ICH9 EHCI (USB 2.0)</SelectItem>
                          <SelectItem value="piix3-uhci">PIIX3 UHCI (USB 1.1)</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="p-3 bg-muted rounded-lg">
                      <p className="text-xs text-muted-foreground">
                        <strong>USB 3.0 (xHCI)</strong> - Fastest, modern devices<br/>
                        <strong>USB 2.0 (EHCI)</strong> - Good compatibility<br/>
                        <strong>USB 1.1 (UHCI)</strong> - Legacy support
                      </p>
                      {vm.state === 'running' && (
                        <p className="text-xs text-yellow-600 mt-2">
                          ⚠️ USB controllers cannot be added to running VMs. Please stop the VM first.
                        </p>
                      )}
                    </div>
                  </div>
                ) : effectiveSelectedDevice === 'scsi-controller' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    <div className="space-y-1.5">
                      <Label className="text-xs">SCSI Controller Model</Label>
                      <Select
                        value={scsiControllerConfig.model}
                        onValueChange={(v) => setScsiControllerConfig({ model: v })}
                      >
                        <SelectTrigger className="h-8 text-sm">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="virtio-scsi">VirtIO SCSI (Best Performance)</SelectItem>
                          <SelectItem value="lsilogic">LSI Logic</SelectItem>
                          <SelectItem value="lsisas1068">LSI SAS 1068</SelectItem>
                          <SelectItem value="megasas">MegaSAS</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="p-3 bg-muted rounded-lg">
                      <p className="text-xs text-muted-foreground">
                        <strong>VirtIO SCSI</strong> - Best performance with guest drivers<br/>
                        <strong>LSI/MegaSAS</strong> - For guests without VirtIO support
                      </p>
                      {vm.state === 'running' && (
                        <p className="text-xs text-yellow-600 mt-2">
                          ⚠️ SCSI controllers cannot be added to running VMs. Please stop the VM first.
                        </p>
                      )}
                    </div>
                  </div>
                ) : effectiveSelectedDevice === 'panic' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    <div className="space-y-1.5">
                      <Label className="text-xs">Panic Notifier Model</Label>
                      <Select
                        value={panicConfig.model}
                        onValueChange={(v) => setPanicConfig({ model: v })}
                      >
                        <SelectTrigger className="h-8 text-sm">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="isa">ISA (x86/x64)</SelectItem>
                          <SelectItem value="hyperv">Hyper-V</SelectItem>
                          <SelectItem value="pseries">pSeries (POWER)</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="p-3 bg-muted rounded-lg">
                      <p className="text-xs text-muted-foreground">
                        The panic notifier allows the host to detect when the guest kernel panics.
                        <br/><br/>
                        <strong>ISA</strong> - Standard for x86/x64 guests<br/>
                        <strong>Hyper-V</strong> - For Windows guests with Hyper-V enlightenments<br/>
                        <strong>pSeries</strong> - For POWER architecture guests
                      </p>
                      {vm.state === 'running' && (
                        <p className="text-xs text-yellow-600 mt-2">
                          ⚠️ Panic notifiers cannot be added to running VMs. Please stop the VM first.
                        </p>
                      )}
                    </div>
                  </div>
                ) : effectiveSelectedDevice === 'vsock' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    <div className="space-y-1.5">
                      <Label className="text-xs">Context ID (CID)</Label>
                      <Input
                        type="number"
                        min={3}
                        className="h-8 text-sm"
                        value={vsockConfig.cid}
                        onChange={(e) => setVsockConfig({ cid: parseInt(e.target.value) || 3 })}
                      />
                      <p className="text-xs text-muted-foreground">
                        CID must be ≥ 3 (0, 1, 2 are reserved)
                      </p>
                    </div>

                    <div className="p-3 bg-muted rounded-lg">
                      <p className="text-xs text-muted-foreground">
                        VirtIO VSOCK provides fast communication between guest and host without network configuration.
                        <br/><br/>
                        Useful for:
                        <br/>• Guest agent communication
                        <br/>• High-performance IPC
                        <br/>• Secure host-guest channels
                      </p>
                      {vm.state === 'running' && (
                        <p className="text-xs text-yellow-600 mt-2">
                          ⚠️ VSOCK devices may not be hotpluggable. Please stop the VM if addition fails.
                        </p>
                      )}
                    </div>
                  </div>
                ) : effectiveSelectedDevice === 'parallel' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    <div className="space-y-1.5">
                      <Label className="text-xs">Target Port Number</Label>
                      <Select
                        value={parallelConfig.targetPort.toString()}
                        onValueChange={(v) => setParallelConfig({ targetPort: parseInt(v) })}
                      >
                        <SelectTrigger className="h-8 text-sm">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="0">LPT1 (Port 0)</SelectItem>
                          <SelectItem value="1">LPT2 (Port 1)</SelectItem>
                          <SelectItem value="2">LPT3 (Port 2)</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="p-3 bg-muted rounded-lg">
                      <p className="text-xs text-muted-foreground">
                        Parallel ports are legacy devices used for printers and other peripherals.
                        <br/><br/>
                        Only add if required for legacy software compatibility.
                      </p>
                      {vm.state === 'running' && (
                        <p className="text-xs text-yellow-600 mt-2">
                          ⚠️ Parallel ports cannot be added to running VMs. Please stop the VM first.
                        </p>
                      )}
                    </div>
                  </div>
                ) : effectiveSelectedDevice === 'smartcard' && !selectedDeviceInfo.comingSoon ? (
                  <div className="space-y-3">
                    <div className="space-y-1.5">
                      <Label className="text-xs">Smartcard Mode</Label>
                      <Select
                        value={smartcardConfig.mode}
                        onValueChange={(v) => setSmartcardConfig({ mode: v })}
                      >
                        <SelectTrigger className="h-8 text-sm">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="passthrough">Passthrough (Host Reader)</SelectItem>
                          <SelectItem value="emulated">Emulated (Certificate-based)</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="p-3 bg-muted rounded-lg">
                      <p className="text-xs text-muted-foreground">
                        <strong>Passthrough:</strong> Forwards the host's physical smartcard reader to the guest. Requires SPICE graphics.
                        <br/><br/>
                        <strong>Emulated:</strong> Uses NSS certificate database for software-based smartcard emulation. No physical reader needed.
                      </p>
                      {vm.state === 'running' && (
                        <p className="text-xs text-yellow-600 mt-2">
                          ⚠️ Smartcard devices cannot be added to running VMs. Please stop the VM first.
                        </p>
                      )}
                    </div>
                  </div>
                ) : (
                  <>
                    <p className="text-sm text-muted-foreground">
                      {selectedDeviceInfo.description}
                    </p>

                    {selectedDeviceInfo.comingSoon && (
                      <div className="p-3 bg-muted rounded-lg">
                        <p className="text-xs text-muted-foreground">
                          This hardware type is not yet available for dynamic addition.
                          You can configure it during VM creation or edit the VM XML directly.
                        </p>
                      </div>
                    )}
                  </>
                )}
              </div>
            ) : (
              <div className="h-full flex items-center justify-center text-center">
                <p className="text-sm text-muted-foreground">
                  Select a hardware type to see details
                </p>
              </div>
            )}
          </div>
        </div>

        <div className="flex justify-end gap-2 pt-2">
          <Button variant="outline" onClick={handleCancel}>
            Cancel
          </Button>
          <Button
            onClick={handleAdd}
            disabled={!selectedDevice || selectedDeviceInfo?.comingSoon || isPending}
          >
            {isPending ? 'Adding...' : 'Add Hardware'}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}
