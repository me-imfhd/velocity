import { Order } from "./types";
export function generateRandomOrder() {
  const basePrice = Math.floor(Math.random() * (17100 - 17000) + 17000) / 100;
  const randomDigit = Math.floor(Math.random() * 10);
  const randomQuantity = Number(
    (Math.random() * (10.0 - 0.01) + 0.01).toFixed(2)
  );
  const isBuyOrder = Math.random() < 0.5;
  const orderPrice = Number(`${basePrice}.${randomDigit}`);
  const order: Order = {
    asset: "SOL",
    assetQuantity: randomQuantity,
    orderPrice: orderPrice,
    FOK: false,
    orderSide: isBuyOrder ? "BUY" : "SELL",
    timestamp: new Date().getTime(),
    userId: isBuyOrder ? "1" : "2",
  };
  return order;
}
