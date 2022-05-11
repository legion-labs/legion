import { createNotificationsStore } from "@lgn/web-client/src/stores/notifications";

export type {
  NotificationsValue,
  NotificationsStore,
} from "@lgn/web-client/src/stores/notifications";

export default createNotificationsStore<Fluent>();
