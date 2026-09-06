export interface SftpNotificationItem {
  id: string;
  message: string;
  timestamp: string;
}

export function createSftpNotifications() {
  let notifications = $state<SftpNotificationItem[]>([]);
  let unreadCount = $state(0);
  let activeToast = $state<{ id: string; message: string; collapsing: boolean } | null>(null);
  let showHistoryModal = $state(false);

  let toastTimer: any = null;
  let animTimer: any = null;

  function addNotification(message: string) {
    if (!message || !message.trim()) return;

    const id = `${Date.now()}-${Math.random().toString(36).substring(2, 7)}`;
    const now = new Date();
    const timeStr = now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });

    notifications.unshift({
      id,
      message,
      timestamp: timeStr,
    });
    unreadCount += 1;

    clearTimeout(toastTimer);
    clearTimeout(animTimer);

    // Exibe o balãozinho
    activeToast = {
      id,
      message,
      collapsing: false,
    };

    // Após 2.5s, dispara animação que encolhe e "entra" em direção ao sininho
    toastTimer = setTimeout(() => {
      if (activeToast && activeToast.id === id) {
        activeToast.collapsing = true;
        animTimer = setTimeout(() => {
          if (activeToast && activeToast.id === id) {
            activeToast = null;
          }
        }, 450);
      }
    }, 2500);
  }

  function openHistory() {
    showHistoryModal = true;
    unreadCount = 0; // Zera o contador visual ao ler o sininho
  }

  function closeHistory() {
    showHistoryModal = false;
  }

  function clearNotifications() {
    notifications = [];
    unreadCount = 0;
  }

  return {
    get notifications() { return notifications; },
    get unreadCount() { return unreadCount; },
    get activeToast() { return activeToast; },
    get showHistoryModal() { return showHistoryModal; },
    addNotification,
    openHistory,
    closeHistory,
    clearNotifications,
  };
}
