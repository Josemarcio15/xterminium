<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';

  const appWindow = getCurrentWindow();

  function startResize(direction: 'East' | 'North' | 'NorthEast' | 'NorthWest' | 'South' | 'SouthEast' | 'SouthWest' | 'West', e: MouseEvent) {
    if (e.button === 0) {
      e.preventDefault();
      e.stopPropagation();
      appWindow.startResizeDragging(direction);
    }
  }
</script>

<!-- Top border -->
<div
  class="fixed top-0 left-2 right-2 h-1.5 z-[9999] cursor-n-resize"
  onmousedown={(e) => startResize('North', e)}
  role="separator"
  tabindex="-1"
  aria-orientation="horizontal"
></div>

<!-- Bottom border -->
<div
  class="fixed bottom-0 left-2 right-2 h-1.5 z-[9999] cursor-s-resize"
  onmousedown={(e) => startResize('South', e)}
  role="separator"
  tabindex="-1"
  aria-orientation="horizontal"
></div>

<!-- Left border -->
<div
  class="fixed left-0 top-2 bottom-2 w-1.5 z-[9999] cursor-w-resize"
  onmousedown={(e) => startResize('West', e)}
  role="separator"
  tabindex="-1"
  aria-orientation="vertical"
></div>

<!-- Right border -->
<div
  class="fixed right-0 top-2 bottom-2 w-1.5 z-[9999] cursor-e-resize"
  onmousedown={(e) => startResize('East', e)}
  role="separator"
  tabindex="-1"
  aria-orientation="vertical"
></div>

<!-- Top-left corner -->
<div
  class="fixed top-0 left-0 w-2.5 h-2.5 z-[10000] cursor-nw-resize"
  onmousedown={(e) => startResize('NorthWest', e)}
  role="separator"
  tabindex="-1"
></div>

<!-- Top-right corner -->
<div
  class="fixed top-0 right-0 w-2.5 h-2.5 z-[10000] cursor-ne-resize"
  onmousedown={(e) => startResize('NorthEast', e)}
  role="separator"
  tabindex="-1"
></div>

<!-- Bottom-left corner -->
<div
  class="fixed bottom-0 left-0 w-2.5 h-2.5 z-[10000] cursor-sw-resize"
  onmousedown={(e) => startResize('SouthWest', e)}
  role="separator"
  tabindex="-1"
></div>

<!-- Bottom-right corner -->
<div
  class="fixed bottom-0 right-0 w-2.5 h-2.5 z-[10000] cursor-se-resize"
  onmousedown={(e) => startResize('SouthEast', e)}
  role="separator"
  tabindex="-1"
></div>
