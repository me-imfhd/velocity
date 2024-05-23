import OrderLinkedList from "./OrderLinkedList";
import { Asset, Price, User, UserId } from "./types";

export const bids = new OrderLinkedList();
export const asks = new OrderLinkedList();
export const latestPrice = new Map<Asset, Price>();
export const users = new Map<UserId, User>();
