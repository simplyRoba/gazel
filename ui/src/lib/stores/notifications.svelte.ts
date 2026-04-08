export type NotificationVariant = "success" | "info" | "error";

export interface NotificationAction {
  label: string;
  onClick: () => void;
}

export interface NotificationInput {
  title?: string;
  message: string;
  variant?: NotificationVariant;
  action?: NotificationAction;
}

export interface Notification extends NotificationInput {
  id: string;
  variant: NotificationVariant;
}

const MAX_VISIBLE = 3;

let notifications = $state<Notification[]>([]);
let nextId = 1;

export function pushNotification(input: NotificationInput): string {
  const id = `notification-${nextId++}`;
  const notification: Notification = {
    ...input,
    id,
    variant: input.variant ?? "info",
  };
  notifications = [notification, ...notifications];
  return id;
}

export function dismissNotification(id: string): void {
  notifications = notifications.filter((n) => n.id !== id);
}

export function clearNotifications(): void {
  notifications = [];
}

export function getVisibleNotifications(): Notification[] {
  return notifications.slice(0, MAX_VISIBLE);
}

export function getAllNotifications(): Notification[] {
  return notifications;
}
