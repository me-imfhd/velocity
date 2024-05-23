export type Timestamp = number; // new Date(),getTime();
export type OrderType = "BUY" | "SELL";
export type Quantity = number;
export type UserId = string;
export type OrderId = string;
export type Asset = "SOL" | "APPL" | "BTC" | "USDC";
export type Price = number;
export type Balance = number;

export interface Order {
  userId: UserId;
  orderType: OrderType;
  asset: Asset;
  assetQuantity: Quantity;
  secondaryAsset: Asset;
  orderPrice: Price;
  timestamp: Timestamp;
}

export interface UserAsset {
  userId: UserId;
  asset: Asset;
}

export type TotalEqualPriceOrders = number;
export interface OrderTypeAndPrice {
  orderPrice: Price;
  orderType: OrderType;
}
export interface User {
  assets: Map<Asset, Quantity>;
}