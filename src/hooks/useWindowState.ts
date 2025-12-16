import { useEffect, useCallback, useRef } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { LogicalPosition, LogicalSize } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';

interface WindowState {
  x: number;
  y: number;
  width: number;
  height: number;
}

/**
 * Hook to persist and restore window position and size
 * Automatically saves window state on move/resize with debouncing
 * Restores window state on mount if available
 */
export function useWindowState() {
  const saveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const windowRef = useRef<any>(null);

  // Save window state with debouncing to avoid excessive saves
  const saveWindowState = useCallback(async () => {
    try {
      const window = await getCurrentWindow();
      const position = await window.outerPosition();
      const size = await window.outerSize();

      await invoke('save_window_state', {
        label: window.label,
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
      });

      console.log(`Saved window state for ${window.label}:`, {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
      });
    } catch (error) {
      console.error('Failed to save window state:', error);
    }
  }, []);

  // Debounced save - waits 500ms after last change before saving
  const debouncedSave = useCallback(() => {
    if (saveTimeoutRef.current) {
      clearTimeout(saveTimeoutRef.current);
    }
    saveTimeoutRef.current = setTimeout(() => {
      saveWindowState();
    }, 500);
  }, [saveWindowState]);

  // Restore window state on mount
  const restoreWindowState = useCallback(async () => {
    try {
      const window = await getCurrentWindow();
      const state = await invoke<WindowState | null>('load_window_state', {
        label: window.label,
      });

      if (state) {
        // Simple validation - just check if values are reasonable
        // (prevents windows from appearing off-screen if monitor configuration changed)
        const isValid =
          state.x >= -2000 &&
          state.y >= -2000 &&
          state.width > 100 &&
          state.height > 100;

        if (isValid) {
          // Restore position and size
          await window.setPosition(new LogicalPosition(state.x, state.y));
          await window.setSize(new LogicalSize(state.width, state.height));

          console.log(`Restored window state for ${window.label}:`, state);
        } else {
          console.warn(
            `Window ${window.label} had invalid position/size, not restoring`
          );
        }
      }
    } catch (error) {
      console.error('Failed to restore window state:', error);
    }
  }, []);

  useEffect(() => {
    let unlistenMove: (() => void) | null = null;
    let unlistenResize: (() => void) | null = null;

    const setupListeners = async () => {
      const window = await getCurrentWindow();
      windowRef.current = window;

      // Restore window state on mount
      await restoreWindowState();

      // Listen for window move events
      unlistenMove = await window.onMoved(() => {
        debouncedSave();
      });

      // Listen for window resize events
      unlistenResize = await window.onResized(() => {
        debouncedSave();
      });
    };

    setupListeners();

    // Cleanup
    return () => {
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current);
      }
      if (unlistenMove) unlistenMove();
      if (unlistenResize) unlistenResize();
    };
  }, [restoreWindowState, debouncedSave]);

  return {
    saveWindowState,
    restoreWindowState,
  };
}

/**
 * Utility function to clear all saved window states
 * Useful for "Reset Window Positions" functionality
 */
export async function clearAllWindowStates(): Promise<void> {
  try {
    await invoke('clear_window_states');
    console.log('Cleared all window states');
  } catch (error) {
    console.error('Failed to clear window states:', error);
    throw error;
  }
}
