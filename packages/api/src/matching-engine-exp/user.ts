import { users } from "./memory";
import { Asset, Quantity, User, UserId } from "./types";
export function createUser(userId: UserId) {
  if (users.has(userId)) {
    return users.get(userId)!;
  }
  const data = {
    assets: new Map<Asset, Quantity>().set("SOL", 0).set("USDC", 0),
  };
  users.set(userId, data);
  return users.get(userId)!;
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
  if (assetBalance === undefined) {
    throw new Error("User does not have this asset");
  }
  return assetBalance;
}
export function deposit(
  userId: UserId,
  asset: Asset,
  depositAmonunt: Quantity
) {
  const user = getUser(userId);
  const balance = user_asset_balance(user, asset);
  user.assets.set(asset, depositAmonunt + balance);
  return { message: "Amount deposited Successfully" };
}
export function withdraw(
  userId: UserId,
  asset: Asset,
  withdrawAmount: Quantity
) {
  const user = getUser(userId);
  const balance = user_asset_balance(user, asset);
  user.assets.set(asset, balance - withdrawAmount);
  return { message: "Amount withdrawn Successfully" };
}
