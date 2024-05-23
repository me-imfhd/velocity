import { fillOrder } from "./fill-order";
import { asks, bids, latestPrice } from "./memory";
import {
  Asset,
  Order,
  OrderTypeAndPrice,
  TotalEqualPriceOrders,
} from "./types";

export function orderbook() {
  return { bids, asks };
}
export function limitOrder(order: Order) {
  const remainingQ = fillOrder(order);
  if (remainingQ === 0) {
    return { filledQuantity: order.assetQuantity }; // all filled, nothing remained
  }
  if (order.orderType === "BUY") {
    bids.addBuyOrders(order);
  } else {
    asks.addSellOrders(order);
  }
  return {
    filledQuantity: order.assetQuantity - remainingQ,
  };
}

export function getAssetsLtsPrice(asset: Asset) {
  const price = latestPrice.get(asset);
  if (!price) {
    return 0;
  } else return price;
}

export function getDepth() {
  const depth = new Map<OrderTypeAndPrice, TotalEqualPriceOrders>();

  for (const order of bids) {
    const orderTypeAndPrice: OrderTypeAndPrice = {
      orderPrice: order.orderPrice,
      orderType: "BUY",
    };
    const prevQuantity = depth.get(orderTypeAndPrice);
    if (prevQuantity) {
      depth.set(orderTypeAndPrice, prevQuantity + 1);
    } else {
      depth.set(orderTypeAndPrice, 1);
    }
  }
  for (const order of asks) {
    const orderTypeAndPrice: OrderTypeAndPrice = {
      orderPrice: order.orderPrice,
      orderType: "SELL",
    };
    const prevQuantity = depth.get(orderTypeAndPrice);
    if (prevQuantity) {
      depth.set(orderTypeAndPrice, prevQuantity + 1);
    } else {
      depth.set(orderTypeAndPrice, 1);
    }
  }
  return depth;
}

export function getQuote() {}
export function marketOrder() {}
