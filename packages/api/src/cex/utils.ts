import { Order } from "./types";
export function generateRandomOrder() {
  const basePrice = Math.floor(Math.random() * (17100 - 17000) + 17000) / 100;
  const randomDigit = Math.floor(Math.random() * 10);
  const randomQuantity = (Math.random() * (10.0 - 0.01) + 0.01).toFixed(2);
  const isBuyOrder = Math.random() < 0.5;
  const order: Order = {
    asset: "SOL",
    assetQuantity: parseFloat(randomQuantity),
    orderPrice: parseFloat(`${basePrice}.${randomDigit}`),
    orderType: isBuyOrder ? "BUY" : "SELL",
    timestamp: new Date().getTime(),
    userId: isBuyOrder ? "1" : "2",
  };
  return order;
}
