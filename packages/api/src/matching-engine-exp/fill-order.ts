import { asks, bids, latestPrice } from "./memory";
import { Asset, Order, Price, Quantity, UserId } from "./types";
import { getUser, user_asset_balance } from "./user";

export function fillOrder(order: Order) {
  console.log(`Recieved ${order.orderSide} order.`);
  let remainingQ = order.assetQuantity;
  if (order.orderSide === "BUY") {
    if (!asks || asks.isEmpty()) {
      console.log("\tAsks List is Empty");
      return remainingQ;
    }
    for (const currentAskOrder of asks) {
      if (currentAskOrder.userId === order.userId) {
        continue;
      }
      if (currentAskOrder.orderPrice > order.orderPrice) {
        // cuts to entry in the orderbook
        break;
      }
      // we got a match! here price of buying the asset is >= the price least price other side is willing to buy at
      const asset = order.asset;
      const deductAssetFrom = currentAskOrder.userId;
      const addAssetTo = order.userId;

      if (currentAskOrder.assetQuantity > remainingQ) {
        currentAskOrder.assetQuantity -= remainingQ; // will deduct buy order quantity from the currentAskOrder
        console.log("\tOrder Matched");
        flipBalance({
          asset,
          assetQuantity: remainingQ,
          price: currentAskOrder.orderPrice, // price for buying asset is more, but we only pay, what its being sold at
          deductAssetFrom,
          addAssetTo,
        });
        latestPrice.set(asset, currentAskOrder.orderPrice);
        return 0; // stop looping all quantities are filled
      } else {
        console.log(`\tOrder Splited `);
        remainingQ -= currentAskOrder.assetQuantity;
        flipBalance({
          asset,
          assetQuantity: currentAskOrder.assetQuantity,
          price: currentAskOrder.orderPrice, // price for buying asset is more, but we only pay, what its being sold at
          deductAssetFrom,
          addAssetTo,
        });
        asks.removeFront();
        // start next loop trying to fill remaining quantity
      }
    }
  } else if (order.orderSide === "SELL") {
    if (!bids || bids.isEmpty()) {
      console.log("\tBids List is Empty");
      return remainingQ;
    }
    for (const currentBidOrder of bids) {
      if (currentBidOrder.userId === order.userId) {
        continue;
      }
      if (currentBidOrder.orderPrice < order.orderPrice) {
        // cuts to entry in the orderbook
        break;
      }
      // we got a match! here price of selling the asset is <= the maximum price other side is willing to pay for
      const asset = order.asset;
      const deductAssetFrom = order.userId;
      const addAssetTo = currentBidOrder.userId;

      if (currentBidOrder.assetQuantity > remainingQ) {
        console.log("\tOrder Matched");
        currentBidOrder.assetQuantity -= remainingQ;
        flipBalance({
          asset,
          assetQuantity: remainingQ,
          price: currentBidOrder.orderPrice, // the price of selling the asset is lesser here, and we that much only
          deductAssetFrom,
          addAssetTo,
        });
        latestPrice.set(asset, currentBidOrder.orderPrice);
        return 0;
      } else {
        console.log("\tOrder Splited");
        remainingQ -= currentBidOrder.assetQuantity;
        flipBalance({
          asset,
          assetQuantity: currentBidOrder.assetQuantity, // the price of selling the asset is lesser here, and we that much only
          price: currentBidOrder.orderPrice,
          deductAssetFrom,
          addAssetTo,
        });
        bids.removeFront();
        // start next loop trying to fill remaining quantity
      }
    }
  }
  return remainingQ;
}

interface FlipBalance {
  deductAssetFrom: UserId;
  addAssetTo: UserId;
  asset: Asset;
  assetQuantity: Quantity;
  price: Price;
}

function flipBalance({
  asset,
  assetQuantity,
  price,
  deductAssetFrom,
  addAssetTo,
}: FlipBalance) {
  console.log("Fliping Balance");
  const user1 = getUser(deductAssetFrom);
  const user2 = getUser(addAssetTo);
  const user_1_asset_quantity = user_asset_balance(user1, asset);
  const user_1_balance = user_asset_balance(user1, "USDC");
  const user_2_asset_quantity = user_asset_balance(user2, asset);
  const user_2_balance = user_asset_balance(user2, "USDC");

  user1.assets.set(asset, user_1_asset_quantity - assetQuantity); // deduct asset quantity
  user1.assets.set("USDC", user_1_balance + price * assetQuantity); // add price
  user2.assets.set(asset, user_2_asset_quantity + assetQuantity); // add assets
  user2.assets.set("USDC", user_2_balance - price * assetQuantity); // deduct price
}
