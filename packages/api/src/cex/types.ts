export type Timestamp = number; // new Date(),getTime();
export type OrderSide = "BUY" | "SELL";
export type OrderType = "LIMIT" | "MARKET";
export type Quantity = number;
export type UserId = string;
export type OrderId = string;
export type Asset = "SOL" | "USDC"; // only sol and usdc exchange supported
export type Price = number;
export type Balance = number;

export interface Order {
  userId: UserId;
  orderSide: OrderSide;
  FOK: boolean;
  asset: Asset;
  assetQuantity: Quantity;
  orderPrice: Price;
  timestamp: Timestamp;
}

export type UserAsset = {
  userId: UserId;
  asset: Asset;
};

export type TotalEqualPriceOrders = number;
export type User = {
  assets: Map<Asset, Quantity>;
};
