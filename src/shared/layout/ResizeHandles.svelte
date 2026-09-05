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
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="fixed top-0 left-2 right-2 h-1.5 z-[9999] cursor-n-resize"
  onmousedown={(e) => startResize('North', e)}
  role="presentation"
></div>

<!-- Bottom border -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="fixed bottom-0 left-2 right-2 h-1.5 z-[9999] cursor-s-resize"
  onmousedown={(e) => startResize('South', e)}
  role="presentation"
></div>

<!-- Left border -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="fixed left-0 top-2 bottom-2 w-1.5 z-[9999] cursor-w-resize"
  onmousedown={(e) => startResize('West', e)}
  role="presentation"
></div>

<!-- Right border -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="fixed right-0 top-2 bottom-2 w-1.5 z-[9999] cursor-e-resize"
  onmousedown={(e) => startResize('East', e)}
  role="presentation"
></div>

<!-- Top-left corner -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="fixed top-0 left-0 w-2.5 h-2.5 z-[10000] cursor-nw-resize"
  onmousedown={(e) => startResize('NorthWest', e)}
  role="presentation"
></div>

<!-- Top-right corner -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="fixed top-0 right-0 w-2.5 h-2.5 z-[10000] cursor-ne-resize"
  onmousedown={(e) => startResize('NorthEast', e)}
  role="presentation"
></div>

<!-- Bottom-left corner -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="fixed bottom-0 left-0 w-2.5 h-2.5 z-[10000] cursor-sw-resize"
  onmousedown={(e) => startResize('SouthWest', e)}
  role="presentation"
></div>

<!-- Bottom-right corner -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="fixed bottom-0 right-0 w-2.5 h-2.5 z-[10000] cursor-se-resize"
  onmousedown={(e) => startResize('SouthEast', e)}
  role="presentation"
></div>
