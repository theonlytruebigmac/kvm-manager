import { invoke } from '@tauri-apps/api/core'
import { normalizeInvokeFailure, SafeFailureError } from './types'

interface ConfirmationPreview {
  token: string
}

export async function confirmationToken(
  operation: string,
  resourceKind: string,
  stableId: string,
  effect: string,
  displayName?: string,
): Promise<string> {
  try {
    const preview = await invoke<ConfirmationPreview>('request_destructive_confirmation', {
      operation,
      resourceKind,
      stableId,
      displayName,
      effect,
    })
    return preview.token
  } catch (error) {
    throw new SafeFailureError(normalizeInvokeFailure(error))
  }
}
