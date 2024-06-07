import { fillOrder } from "./fill-order";
import { asks, bids, latestPrice } from "./memory";
import {
  Asset,
  Order,
  OrderSide,
  Price,
  Quantity,
  TotalEqualPriceOrders,
  UserId,
} from "./types";
import { getUser } from "./user";

export function orderbook() {
  return { bids: bids.fromArray(), asks: asks.fromArray() };
}
export function order(order: Order) {
  if (order.FOK) {
    throw new Error("Not taking marketOrder rn");
    // return marketOrder(order);
  }
  let remainingQ = order.assetQuantity;
  if (order.orderSide === "BUY") {
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
    if (!userBalance || userBalance < order.assetQuantity) {
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
  }
  return price;
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
    bidDepth: Array.from(bidDepth),
    askDepth: Array.from(askDepth),
  };
}

// interface GetQuote {
//   userId: UserId;
//   quantityToTrade: Quantity;
//   orderSide: OrderSide;
// }
// export function getQuote({ orderSide, quantityToTrade, userId }: GetQuote) {
//   let remainingQ = quantityToTrade;
//   let totalCost = 0;
//   if (orderSide === "BUY") {
//     for (const currentAskOrder of asks) {
//       if (currentAskOrder.userId === userId) {
//         continue;
//       }
//       if (currentAskOrder.assetQuantity >= remainingQ) {
//         totalCost += remainingQ * currentAskOrder.orderPrice;
//         let avgPrice = totalCost / quantityToTrade;
//         return { quantityTradeable: quantityToTrade, avgPrice };
//       } else {
//         remainingQ -= currentAskOrder.assetQuantity;
//         totalCost += currentAskOrder.assetQuantity * currentAskOrder.orderPrice;
//         continue;
//       }
//     }
//   } else if (orderSide === "SELL") {
//     for (const currentBidOrder of bids) {
//       if (currentBidOrder.userId === userId) {
//         continue;
//       }
//       if (currentBidOrder.assetQuantity >= remainingQ) {
//         totalCost += remainingQ * currentBidOrder.orderPrice;
//         let avgPrice = totalCost / quantityToTrade;
//         return { quantityTradeable: quantityToTrade, avgPrice };
//       } else {
//         remainingQ -= currentBidOrder.assetQuantity;
//         totalCost += currentBidOrder.assetQuantity * currentBidOrder.orderPrice;
//         continue;
//       }
//     }
//   }
//   return {
//     quantityTradeable: quantityToTrade - remainingQ,
//     avgPrice: totalCost / (quantityToTrade - remainingQ),
//   };
// }
// export function marketOrder(order: Order) {
//   const canMatchAll = checkOrder(order);
//   if (!canMatchAll) {
//     return null;
//   } else {
//     const remainingQ = fillOrder(order);
//     if (remainingQ === 0) {
//       return { filledQuantity: order.assetQuantity };
//     }
//     throw new Error(`Could not exchange ${remainingQ} amount of asset.`);
//   }
// }

// function checkOrder(order: Order) {
//   let remainingQ = order.assetQuantity;
//   if (order.orderSide === "BUY") {
//     for (const currentAskOrder of asks) {
//       if (currentAskOrder.userId === order.userId) {
//         continue;
//       }
//       if (currentAskOrder.orderPrice > order.orderPrice) {
//         return true;
//       }
//       if (currentAskOrder.assetQuantity > remainingQ) {
//         return true;
//       } else {
//         remainingQ -= currentAskOrder.assetQuantity;
//         continue;
//       }
//     }
//   } else {
//     for (const currentBidOrder of bids) {
//       if (currentBidOrder.userId === order.userId) {
//         continue;
//       }
//       if (currentBidOrder.orderPrice > order.orderPrice) {
//         return true;
//       }
//       if (currentBidOrder.assetQuantity > remainingQ) {
//         return true;
//       } else {
//         remainingQ -= currentBidOrder.assetQuantity;
//         continue;
//       }
//     }
//   }
//   return false;
// }
