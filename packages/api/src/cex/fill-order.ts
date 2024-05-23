import { asks, bids, latestPrice, users } from "./memory";
import { Asset, Order, Price, Quantity, User, UserId } from "./types";

export function fillOrder(order: Order) {
  let remainingQ = order.assetQuantity;
  if (order.orderType === "BUY") {
    for (const currentAskOrder of asks) {
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
        flipBalance({
          asset,
          assetQuantity: remainingQ,
          price: currentAskOrder.orderPrice, // price for buying asset is more, but we only pay, what its being sold at
          secondaryAsset: order.secondaryAsset,
          deductAssetFrom,
          addAssetTo,
        });
        latestPrice.set(asset, currentAskOrder.orderPrice);
        return 0; // stop looping all quantities are filled
      } else {
        remainingQ -= currentAskOrder.assetQuantity;
        flipBalance({
          asset,
          assetQuantity: currentAskOrder.assetQuantity,
          price: currentAskOrder.orderPrice, // price for buying asset is more, but we only pay, what its being sold at
          secondaryAsset: order.secondaryAsset,
          deductAssetFrom,
          addAssetTo,
        });
        asks.removeFront();
        // start next loop trying to fill remaining quantity
      }
    }
  } else if (order.orderType === "SELL") {
    for (const currentBidOrder of bids) {
      if (currentBidOrder.orderPrice < order.orderPrice) {
        // cuts to entry in the orderbook
        break;
      }
      // we got a match! here price of selling the asset is <= the maximum price other side is willing to pay for
      const asset = order.asset;
      const deductAssetFrom = order.userId;
      const addAssetTo = currentBidOrder.userId;

      if (currentBidOrder.assetQuantity > remainingQ) {
        currentBidOrder.assetQuantity -= remainingQ;
        flipBalance({
          asset,
          assetQuantity: remainingQ,
          price: currentBidOrder.orderPrice, // the price of selling the asset is lesser here, and we that much only
          secondaryAsset: order.secondaryAsset,
          deductAssetFrom,
          addAssetTo,
        });
        latestPrice.set(asset, currentBidOrder.orderPrice);
        return 0;
      } else {
        remainingQ -= currentBidOrder.assetQuantity;
        flipBalance({
          asset,
          assetQuantity: currentBidOrder.assetQuantity, // the price of selling the asset is lesser here, and we that much only
          price: currentBidOrder.orderPrice,
          secondaryAsset: order.secondaryAsset,
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
  secondaryAsset: Asset;
  price: Price;
}

function flipBalance({
  asset,
  assetQuantity,
  price,
  secondaryAsset,
  deductAssetFrom,
  addAssetTo,
}: FlipBalance) {
  const user1 = getUser(deductAssetFrom);
  const user2 = getUser(addAssetTo);
  const user_1_asset_quantity = user_asset_balance(user1, asset);
  const user_1_balance = user_asset_balance(user1, secondaryAsset);
  const user_2_asset_quantity = user_asset_balance(user2, asset);
  const user_2_balance = user_asset_balance(user2, secondaryAsset);

  user1.assets.set(asset, user_1_asset_quantity - assetQuantity); // deduct asset quantity
  user1.assets.set(secondaryAsset, user_1_balance + price * assetQuantity); // add price
  user2.assets.set(asset, user_2_asset_quantity + assetQuantity); // add assets
  user2.assets.set(secondaryAsset, user_2_balance - price * assetQuantity); // deduct price
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
