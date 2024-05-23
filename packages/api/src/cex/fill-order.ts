import { asks, bids, latestPrice, users } from "./memory";
import { Asset, Order, Price, Quantity, User, UserId } from "./types";

export function fillOrder(order: Order) {
  let remainingQ = order.assetQuantity;
  if (order.orderType === "BUY") {
    const leastPriceAskOrder = asks.firstItem();
    if (!leastPriceAskOrder) {
      return remainingQ; // no asks to match with
    }
    for (const _ of asks) {
      if (leastPriceAskOrder.orderPrice > order.orderPrice) {
        // cuts to entry in the orderbook
        break;
      }
      // we got a match! here price of buying the asset is >= the price least price other side is willing to buy at
      const asset = order.asset;
      const userId1 = leastPriceAskOrder.userId;
      const userId2 = order.userId;

      if (leastPriceAskOrder.assetQuantity > remainingQ) {
        asks.deductQuantity(remainingQ);
        flipBalance({
          asset,
          assetQuantity: remainingQ,
          price: leastPriceAskOrder.orderPrice, // price for buying asset is more, but we only pay, what its being sold at
          secondaryAsset: order.secondaryAsset,
          userId1,
          userId2,
        });
        latestPrice.set(asset, leastPriceAskOrder.orderPrice);
        return 0;
      } else {
        remainingQ -= leastPriceAskOrder.assetQuantity;
        flipBalance({
          asset,
          assetQuantity: leastPriceAskOrder.assetQuantity,
          price: leastPriceAskOrder.orderPrice, // price for buying asset is more, but we only pay, what its being sold at
          secondaryAsset: order.secondaryAsset,
          userId1,
          userId2,
        });
        asks.removeFront();
      }
    }
  } else if (order.orderType === "SELL") {
    const maxPriceBuyOrder = bids.firstItem();
    if (!maxPriceBuyOrder) {
      return remainingQ; // no asks to match with
    }
    for (const _ of bids) {
      if (maxPriceBuyOrder.orderPrice < order.orderPrice) {
        // cuts to entry in the orderbook
        break;
      }
      // we got a match! here price of selling the asset is <= the maximum price other side is willing to pay for
      const asset = order.asset;
      const userId1 = order.userId;
      const userId2 = maxPriceBuyOrder.userId;

      if (maxPriceBuyOrder.assetQuantity > remainingQ) {
        asks.deductQuantity(remainingQ);
        flipBalance({
          asset,
          assetQuantity: remainingQ,
          price: maxPriceBuyOrder.orderPrice, // the price of selling the asset is lesser here, and we that much only
          secondaryAsset: order.secondaryAsset,
          userId1,
          userId2,
        });
        latestPrice.set(asset, maxPriceBuyOrder.orderPrice);
        return 0;
      } else {
        remainingQ -= maxPriceBuyOrder.assetQuantity;
        flipBalance({
          asset,
          assetQuantity: maxPriceBuyOrder.assetQuantity, // the price of selling the asset is lesser here, and we that much only
          price: maxPriceBuyOrder.orderPrice,
          secondaryAsset: order.secondaryAsset,
          userId1,
          userId2,
        });
        bids.removeFront();
      }
    }
  }
  return remainingQ;
}

interface FlipBalance {
  userId1: UserId;
  userId2: UserId;
  asset: Asset;
  assetQuantity: Quantity;
  secondaryAsset: Asset;
  price: Price;
}

function flipBalance({
  asset,
  assetQuantity,
  price,
  secondaryAsset,
  userId1,
  userId2,
}: FlipBalance) {
  const user1 = getUser(userId1);
  const user2 = getUser(userId2);
  const user_1_asset_quantity = user_asset_balance(user1, asset);
  const user_1_balance = user_asset_balance(user1, secondaryAsset);
  const user_2_asset_quantity = user_asset_balance(user2, asset);
  const user_2_balance = user_asset_balance(user2, secondaryAsset);

  user1.assets.set(asset, user_1_asset_quantity - assetQuantity); // deduct asset quantity
  user1.assets.set(secondaryAsset, user_1_balance + price); // add price
  user2.assets.set(asset, user_2_asset_quantity + assetQuantity); // add assets
  user2.assets.set(secondaryAsset, user_2_balance - price); // deduct price
}

export function getUser(userId: UserId) {
  const userData = users.get(userId);
  if (!userData) {
    throw new Error("User does not exist");
  }
  return userData;
}

export function user_asset_balance(user: User, asset: Asset) {
  const assetBalance = user.assets.get(asset);
  if (!assetBalance) {
    throw new Error("User does not have this asset");
  }
  return assetBalance;
}
