import { fillOrder, getUser } from "./fill-order";
import { asks, bids, latestPrice } from "./memory";
import {
  Asset,
  Order,
  OrderType,
  Price,
  Quantity,
  TotalEqualPriceOrders,
  UserId,
} from "./types";

export function orderbook() {
  return { bids, asks };
}
export function limitOrder(order: Order) {
  let remainingQ = order.assetQuantity;
  if (order.orderType === "BUY") {
    const userBalance = getUser(order.userId).assets.get("USDC");
    if (!userBalance || userBalance < order.assetQuantity * order.orderPrice) {
      throw new Error("Not Enough Balance");
    }
    remainingQ = fillOrder(order);
    if (remainingQ === 0) {
      return { filledQuantity: order.assetQuantity }; // all filled, nothing remained
    }
    console.log("\t\t\tAdding Buy Order to orderbook");
    bids.addBuyOrders(order);
  } else {
    const userBalance = getUser(order.userId).assets.get(order.asset);
    if (!userBalance || userBalance < order.assetQuantity * order.orderPrice) {
      throw new Error(`Not Enough ${order.asset}`);
    }
    remainingQ = fillOrder(order);
    if (remainingQ === 0) {
      return { filledQuantity: order.assetQuantity }; // all filled, nothing remained
    }
    console.log("\t\t\tAdding Sell Order to orderbook");
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
  const bidDepth = new Map<Price, TotalEqualPriceOrders>();
  const askDepth = new Map<Price, TotalEqualPriceOrders>();

  for (const order of bids) {
    const prevQuantity = bidDepth.get(order.orderPrice);
    if (prevQuantity) {
      bidDepth.set(order.orderPrice, prevQuantity + order.assetQuantity);
    } else {
      bidDepth.set(order.orderPrice, order.assetQuantity);
    }
  }
  for (const order of asks) {
    const prevQuantity = askDepth.get(order.orderPrice);
    if (prevQuantity) {
      askDepth.set(order.orderPrice, prevQuantity + order.assetQuantity);
    } else {
      askDepth.set(order.orderPrice, order.assetQuantity);
    }
  }
  return {
    bidDepth: Array.from(bidDepth).slice(0, 20),
    askDepth: Array.from(askDepth).reverse().slice(0, 20),
  };
}
interface GetQuote {
  userId: UserId;
  quantityToTrade: Quantity;
  orderType: OrderType;
}
export function getQuote({ orderType, quantityToTrade, userId }: GetQuote) {
  let remainingQ = quantityToTrade;
  let totalCost = 0;
  if (orderType === "BUY") {
    for (const currentAskOrder of asks) {
      if (currentAskOrder.userId === userId) {
        continue;
      }
      if (currentAskOrder.assetQuantity >= remainingQ) {
        totalCost += remainingQ * currentAskOrder.orderPrice;
        let avgPrice = totalCost / quantityToTrade;
        // only return when all quantity is tradeable
        return { avgPrice };
      } else {
        remainingQ -= currentAskOrder.assetQuantity;
        totalCost += currentAskOrder.assetQuantity * currentAskOrder.orderPrice;
        continue;
      }
    }
  } else if (orderType === "SELL") {
    for (const currentBidOrder of bids) {
      if (currentBidOrder.userId === userId) {
        continue;
      }
      if (currentBidOrder.assetQuantity >= remainingQ) {
        totalCost += remainingQ * currentBidOrder.orderPrice;
        let avgPrice = totalCost / quantityToTrade;
        return { avgPrice };
      } else {
        remainingQ -= currentBidOrder.assetQuantity;
        totalCost += currentBidOrder.assetQuantity * currentBidOrder.orderPrice;
        continue;
      }
    }
  }
  return null;
}
export function marketOrder(order: Order) {
  const canMatchAll = checkOrder(order);
  if (!canMatchAll) {
    return null;
  } else {
    const remainingQ = fillOrder(order);
    if (remainingQ === 0) {
      return { filledQuantity: order.assetQuantity };
    }
    throw new Error("Could not exchange all quantity");
  }
}

function checkOrder(order: Order) {
  let remainingQ = order.assetQuantity;
  if (order.orderType === "BUY") {
    for (const currentAskOrder of asks) {
      if (currentAskOrder.userId === order.userId) {
        continue;
      }
      if (currentAskOrder.orderPrice > order.orderPrice) {
        return true;
      }
      if (currentAskOrder.assetQuantity > remainingQ) {
        return true;
      } else {
        remainingQ -= currentAskOrder.assetQuantity;
        continue;
      }
    }
  } else {
    for (const currentBidOrder of bids) {
      if (currentBidOrder.userId === order.userId) {
        continue;
      }
      if (currentBidOrder.orderPrice > order.orderPrice) {
        return true;
      }
      if (currentBidOrder.assetQuantity > remainingQ) {
        return true;
      } else {
        remainingQ -= currentBidOrder.assetQuantity;
        continue;
      }
    }
  }
  return false;
}
