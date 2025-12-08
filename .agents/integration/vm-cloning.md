# VM Cloning Feature - Implementation Summary

## Overview
Implemented complete VM cloning functionality allowing users to create exact copies of existing VMs with new UUIDs and MAC addresses.

## Backend Implementation

### Service Layer (`src-tauri/src/services/vm_service.rs`)

**Main Method: `clone_vm()`**
- Fetches source VM XML configuration
- Modifies XML for the clone (name, UUID, MAC addresses)
- Defines new VM in libvirt
- Returns the new VM's ID

**Helper Methods:**
1. **`modify_xml_for_clone()`**
   - Removes `<uuid>` element (libvirt auto-generates new one)
   - Changes VM name to new name
   - Uses regex-based XML manipulation

2. **`regenerate_mac_addresses()`**
   - Finds all MAC addresses in XML
   - Generates new random MAC addresses with 52:54:00 prefix
   - Ensures uniqueness to avoid network conflicts

**Dependencies:**
- Added `rand = "0.8"` to Cargo.toml for MAC generation

### Command Layer (`src-tauri/src/commands/vm.rs`)

**Command: `clone_vm`**
- Parameters: `source_vm_id: String`, `new_name: String`
- Returns: New VM ID
- Emits event: `vm-cloned` with source/cloned IDs and timestamp

**Registration:**
- Added to `lib.rs` invoke_handler: `commands::vm::clone_vm`

## Frontend Implementation

### Types (`src/lib/types.ts`)
```typescript
export interface CloneVmConfig {
  sourceVmId: string
  newName: string
}
```

### API Wrapper (`src/lib/tauri.ts`)
```typescript
cloneVm: (sourceVmId: string, newName: string) => invoke<string>('clone_vm', { sourceVmId, newName })
```

### UI Component (`src/components/vm/CloneVmDialog.tsx`)

**Features:**
- Dialog-based UI with name input
- Pre-fills with "{original-name}-clone"
- Shows descriptive text about clone behavior
- Disable clone button when name is empty
- Loading state during clone operation
- Toast notifications for success/error
- Auto-invalidates VM list query on success

**UI Dependencies:**
- `@radix-ui/react-dialog` for Dialog primitive
- Created `src/components/ui/dialog.tsx` with shadcn/ui styling

### Integration (`src/components/vm/VmCard.tsx`)
- Added Clone button (only visible when VM is stopped)
- Positioned between "View Details" and "Console" buttons
- Uses CloneVmDialog component

## User Flow

1. User sees stopped VM in VM list
2. Clicks "Clone" button on VM card
3. Dialog opens with default name "{vm-name}-clone"
4. User edits name if desired
5. Clicks "Clone VM"
6. Backend creates copy with new UUID and MAC addresses
7. Success toast appears
8. VM list refreshes showing new VM
9. New VM is in "stopped" state, ready to start

## Technical Details

**MAC Address Generation:**
- Prefix: `52:54:00` (QEMU/KVM reserved range)
- Random bytes: 3 octets generated with `rand::random::<u8>()`
- Format: `52:54:00:XX:XX:XX`

**XML Modification Strategy:**
- Uses regex for targeted replacements
- Removes UUID entirely (libvirt auto-generates)
- Changes `<name>` element
- Replaces all `<mac address='...'/>` entries

**State Management:**
- New VM created in "stopped" state
- No storage duplication (shares backing file if applicable)
- Network configuration preserved
- All other VM settings preserved

## Testing Recommendations

1. Clone a running VM (should fail gracefully or stop first)
2. Clone a VM with snapshots (verify snapshot handling)
3. Clone with special characters in name
4. Clone multiple times from same source
5. Verify MAC addresses are unique
6. Verify new VM can start without conflicts

## Future Enhancements

- **Option to copy disks**: Currently shares storage, could add full copy
- **Clone to different pool**: Allow selecting destination storage pool
- **Batch cloning**: Clone multiple VMs at once
- **Clone with modifications**: Change CPU/memory during clone
- **Template system**: Save VMs as templates for cloning

## Events

**Emitted Events:**
- `vm-cloned`: Emitted when clone completes
  ```json
  {
    "sourceVmId": "string",
    "clonedVmId": "string",
    "newName": "string",
    "timestamp": 1234567890
  }
  ```

## Total Lines Added
- Backend: ~120 lines (service + command)
- Frontend: ~110 lines (dialog component + integration)
- Total: ~230 lines

## Completion Status
✅ Backend service implementation
✅ Backend command handler
✅ Frontend types
✅ Frontend API wrapper
✅ UI component with dialog
✅ Integration into VM card
✅ Event emission
✅ Build verification

**Status: COMPLETE** - Ready for testing and user feedback.
