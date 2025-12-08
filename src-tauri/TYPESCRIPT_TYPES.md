# TypeScript Type Definitions for KVM Manager Backend

This file provides TypeScript types matching the Rust backend models. Frontend Agent should use these types when calling Tauri commands.

## Installation

Copy these types to your frontend TypeScript project:

```bash
# Suggested location
src/types/backend.ts
```

## Type Definitions

```typescript
/**
 * VM State enum - matches Rust VmState
 */
export type VmState = 'running' | 'stopped' | 'paused' | 'suspended';

/**
 * Network Interface configuration
 */
export interface NetworkInterface {
  name: string;
  macAddress: string;
  network: string;
}

/**
 * Virtual Machine model
 * Matches Rust VM struct from models/vm.rs
 */
export interface VM {
  id: string;              // UUID
  name: string;
  state: VmState;
  cpuCount: number;
  memoryMb: number;
  diskSizeGb: number;
  networkInterfaces: NetworkInterface[];
}

/**
 * Host Information
 * Matches Rust HostInfo from models/host.rs
 */
export interface HostInfo {
  hostname: string;
  cpuModel: string;
  cpuCount: number;
  memoryTotalMb: number;
  libvirtVersion: string;
}

/**
 * Connection Status
 * Matches Rust ConnectionStatus from models/host.rs
 */
export interface ConnectionStatus {
  connected: boolean;
  uri: string;
  error: string | null;
}

/**
 * VM Configuration for creating new VMs
 * Matches Rust VmConfig from models/vm.rs
 */
export interface VmConfig {
  name: string;
  cpuCount: number;
  memoryMb: number;
  diskSizeGb: number;
  osType: string;
  isoPath?: string;
  network: string;
}
```

## Command Functions

Create a backend service to wrap Tauri commands:

```typescript
import { invoke } from '@tauri-apps/api/core';
import type { VM, HostInfo, ConnectionStatus, VmConfig } from './types/backend';

/**
 * Backend service for communicating with Rust backend
 */
export class BackendService {
  /**
   * Get all VMs (active and inactive)
   */
  static async getVms(): Promise<VM[]> {
    return invoke<VM[]>('getVms');
  }

  /**
   * Get a single VM by ID
   */
  static async getVm(vmId: string): Promise<VM> {
    return invoke<VM>('getVm', { vmId });
  }

  /**
   * Start a stopped VM
   */
  static async startVm(vmId: string): Promise<void> {
    return invoke('startVm', { vmId });
  }

  /**
   * Stop a running VM
   */
  static async stopVm(vmId: string): Promise<void> {
    return invoke('stopVm', { vmId });
  }

  /**
   * Pause a running VM
   */
  static async pauseVm(vmId: string): Promise<void> {
    return invoke('pauseVm', { vmId });
  }

  /**
   * Resume a paused VM
   */
  static async resumeVm(vmId: string): Promise<void> {
    return invoke('resumeVm', { vmId });
  }

  /**
   * Reboot a VM
   */
  static async rebootVm(vmId: string): Promise<void> {
    return invoke('rebootVm', { vmId });
  }

  /**
   * Get host information
   */
  static async getHostInfo(): Promise<HostInfo> {
    return invoke<HostInfo>('getHostInfo');
  }

  /**
   * Get libvirt connection status
   */
  static async getConnectionStatus(): Promise<ConnectionStatus> {
    return invoke<ConnectionStatus>('getConnectionStatus');
  }

  /**
   * Create a new VM (coming in Week 2+)
   */
  static async createVm(config: VmConfig): Promise<string> {
    return invoke<string>('createVm', { config });
  }
}
```

## Usage Examples

### React Component Example

```tsx
import { useEffect, useState } from 'react';
import { BackendService } from './services/backend';
import type { VM } from './types/backend';

function VmList() {
  const [vms, setVms] = useState<VM[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadVms();
  }, []);

  async function loadVms() {
    try {
      setLoading(true);
      const data = await BackendService.getVms();
      setVms(data);
      setError(null);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }

  async function handleStartVm(vmId: string) {
    try {
      await BackendService.startVm(vmId);
      // Refresh the list
      await loadVms();
    } catch (err) {
      alert(`Failed to start VM: ${err}`);
    }
  }

  if (loading) return <div>Loading VMs...</div>;
  if (error) return <div>Error: {error}</div>;

  return (
    <div>
      <h1>Virtual Machines</h1>
      {vms.map((vm) => (
        <div key={vm.id}>
          <h3>{vm.name}</h3>
          <p>State: {vm.state}</p>
          <p>CPU: {vm.cpuCount} cores</p>
          <p>Memory: {vm.memoryMb} MB</p>
          {vm.state === 'stopped' && (
            <button onClick={() => handleStartVm(vm.id)}>Start</button>
          )}
        </div>
      ))}
    </div>
  );
}
```

### React Query Example

```tsx
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { BackendService } from './services/backend';

function VmListWithQuery() {
  const queryClient = useQueryClient();

  const { data: vms, isLoading, error } = useQuery({
    queryKey: ['vms'],
    queryFn: BackendService.getVms,
  });

  const startVmMutation = useMutation({
    mutationFn: BackendService.startVm,
    onSuccess: () => {
      // Invalidate and refetch VMs
      queryClient.invalidateQueries({ queryKey: ['vms'] });
    },
  });

  if (isLoading) return <div>Loading...</div>;
  if (error) return <div>Error: {String(error)}</div>;

  return (
    <div>
      {vms?.map((vm) => (
        <div key={vm.id}>
          <h3>{vm.name}</h3>
          <button
            onClick={() => startVmMutation.mutate(vm.id)}
            disabled={startVmMutation.isPending}
          >
            Start
          </button>
        </div>
      ))}
    </div>
  );
}
```

## Error Handling

All commands return `Result<T, String>` in Rust, which means:
- **Success**: Promise resolves with the value
- **Error**: Promise rejects with error message string

```typescript
try {
  await BackendService.startVm(vmId);
  toast.success('VM started successfully');
} catch (error) {
  // error is a string from Rust
  toast.error(`Failed to start VM: ${error}`);
}
```

## Common Error Messages

From `utils/error.rs`:
- `"libvirtd is not running"` - Need to start libvirt service
- `"Permission denied. Add user to libvirt group"` - Permission issue
- `"VM not found: {id}"` - Invalid VM ID
- `"VM is already running"` - Invalid state transition
- `"Libvirt error: {details}"` - Other libvirt errors

## Type Safety Notes

1. **snake_case → camelCase**: Tauri automatically converts
   - Rust: `cpu_count` → TypeScript: `cpuCount`
   - Rust: `memory_mb` → TypeScript: `memoryMb`

2. **Option<T> → T | null**: Rust Option becomes nullable
   - Rust: `Option<String>` → TypeScript: `string | null`

3. **Vec<T> → T[]**: Rust vectors become arrays
   - Rust: `Vec<VM>` → TypeScript: `VM[]`

4. **Enums**: Must match exactly
   - VmState values: exactly `'running' | 'stopped' | 'paused' | 'suspended'`

## Testing Types

```typescript
import { describe, it, expect } from 'vitest';
import { BackendService } from './services/backend';

describe('Backend Service', () => {
  it('should return array of VMs', async () => {
    const vms = await BackendService.getVms();
    expect(Array.isArray(vms)).toBe(true);

    if (vms.length > 0) {
      expect(vms[0]).toHaveProperty('id');
      expect(vms[0]).toHaveProperty('name');
      expect(vms[0]).toHaveProperty('state');
    }
  });

  it('should return host info', async () => {
    const info = await BackendService.getHostInfo();
    expect(info).toHaveProperty('hostname');
    expect(info).toHaveProperty('libvirtVersion');
  });
});
```

## Future Additions

Types for upcoming features (Week 2+):
- Storage pools
- Networks
- VM creation wizard
- Snapshots
- Guest agent info

---

**Status**: These types match the Week 1 backend implementation.
**Last Updated**: 2025-12-07
**For questions**: Coordinate with Backend Agent via `.agents/status/backend-status.md`
