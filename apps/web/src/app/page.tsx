"use client";
import React, { useState } from "react";
import { Button } from "@repo/ui/components";
import { Orderbook } from "../components/orderbook-components";
import { useOrderContext } from "../lib/hooks/order.hooks";

export default function Page() {
  return (
    <div>
      <div className="flex h-screen w-full items-center justify-center gap-10 p-4">
        <Orderbook />
        <OrderComponent />
      </div>
    </div>
  );
}

const OrderComponent = () => {
  const asset = "SOL";
  const { isBid, setIsBid } = useOrderContext();
  return (
    <div className="flex flex-col w-[300px] border rounded-lg">
      <div className="flex h-[60px]">
        <div
          onClick={() => setIsBid(true)}
          className={`flex-1 flex justify-center items-center cursor-pointer border-b-2 p-4 text-green-400 ${
            isBid
              ? "bg-green-950 border-b-green-800"
              : "bg-transparent hover:border-b-green-800"
          }`}
        >
          <p className="text-sm font-semibold">Buy</p>
        </div>
        <div
          onClick={() => setIsBid(false)}
          className={`flex-1 flex justify-center items-center cursor-pointer border-b-2 p-4 text-red-400 ${
            !isBid
              ? "bg-red-950 border-b-red-800"
              : "hover:border-b-red-800 bg-transparent"
          }`}
        >
          <p className="text-sm font-semibold">Sell</p>
        </div>
      </div>
      <OrderForm asset={asset} />
    </div>
  );
};

interface OrderFormProps {
  asset: string;
}

const OrderForm = ({ asset }: OrderFormProps) => {
  const { isBid, isLimit, setIsLimit, quantity, price } = useOrderContext();
  const [orderValueIsUSDC, setOrderValueIsUSDC] = useState(true);
  return (
    <div className="flex flex-col gap-1 px-3 w-[300px]">
      <div className="flex gap-5">
        <div
          onClick={() => setIsLimit(true)}
          className="flex justify-center items-center cursor-pointer py-2"
        >
          <p
            className={`text-sm font-medium py-1 hover:text-white border-b-2 hover:border-white ${
              isLimit ? "text-white border-b-white" : "text-muted-foreground"
            }`}
          >
            Limit
          </p>
        </div>
        <div
          onClick={() => setIsLimit(false)}
          className="flex justify-center items-center py-2 cursor-not-allowed"
        >
          <p
            className={`text-sm font-medium py-1 hover:text-white border-b-2 hover:border-white ${
              !isLimit ? "text-white border-b-white" : "text-muted-foreground"
            }`}
          >
            Market
          </p>
        </div>
      </div>
      {isLimit ? (
        <div className="flex flex-col gap-2">
          <OrderBalance userId={1} asset={isBid ? "USDC" : asset} />
          <OrderPrice />
          <OrderQuantity asset={asset} />
        </div>
      ) : (
        <div className="flex flex-col">
          <p
            onClick={() => setOrderValueIsUSDC(!orderValueIsUSDC)}
            className="w-max text-start cursor-pointer text-sm hover:bg-transparent hover:text-muted-foreground/80 text-muted-foreground"
          >
            Change Order Value
          </p>
          <OrderBalance userId={1} asset={orderValueIsUSDC ? "USDC" : asset} />
          <MarketOrderQuantity asset={orderValueIsUSDC ? "USDC" : asset} />
        </div>
      )}
      {isLimit && <OrderEstimation />}
      <Button
        variant="ghost"
        size="lg"
        className={`my-2 border rounded-xl ${
          isBid
            ? "border-green-900 bg-green-950 hover:bg-green-900"
            : "border-red-900 bg-red-950 hover:bg-red-900"
        }`}
      >
        {isBid ? "Buy" : "Sell"}
      </Button>
    </div>
  );
};

interface OrderBalanceProps {
  userId: number;
  asset: string;
}

const OrderBalance = ({ asset }: OrderBalanceProps) => {
  const solBalance = 0;
  const usdcBalance = 0;
  return (
    <div className="flex justify-between py-2">
      <p className="text-xs">Available Balance</p>
      <p className="text-xs font-medium">
        {asset === "USDC" ? usdcBalance.toFixed(2) : solBalance?.toFixed(2)}{" "}
        {asset}
      </p>
    </div>
  );
};

const MarketOrderQuantity = ({ asset }: { asset: string }) => {
  const { marketOrderQuantity, setMarketOrderQuantity } = useOrderContext();
  return (
    <div className="flex flex-col gap-2">
      <div className="relative">
        <input
          placeholder="0"
          className="h-12 w-[280px] pr-12 text-right text-2xl border-2 rounded-lg bg-[var(--background)]"
          step={0.01}
          inputMode="numeric"
          type="number"
          value={marketOrderQuantity}
          onChange={(e) => setMarketOrderQuantity(Number(e.target.value))}
        />
        <div className="absolute right-1 top-1 flex items-center p-2">
          <img
            loading="lazy"
            width="24"
            height="24"
            className="rounded-full"
            src={`https://backpack.exchange/_next/image?url=%2Fcoins%2F${asset.toLowerCase()}.png&w=48&q=75`}
          />
        </div>
      </div>
    </div>
  );
};

const OrderPrice = () => {
  const { price, setPrice } = useOrderContext();
  return (
    <div className="flex flex-col gap-2">
      <p className="text-xs text-baseTextMedEmphasis">Price</p>
      <div className="relative">
        <input
          placeholder="0"
          className="h-12 w-[280px] pr-12 text-right text-2xl border-2 rounded-lg bg-[var(--background)]"
          step={0.01}
          inputMode="numeric"
          type="number"
          value={price}
          onChange={(e) => setPrice(Number(e.target.value))}
        />
        <div className="absolute right-1 top-1 flex items-center p-2">
          <img
            loading="lazy"
            width="24"
            height="24"
            className="rounded-full"
            src={`https://backpack.exchange/_next/image?url=%2Fcoins%2F${"usdc"}.png&w=48&q=75`}
          />
        </div>
      </div>
    </div>
  );
};
const OrderQuantity = ({ asset }: { asset: string }) => {
  const { quantity, setQuantity } = useOrderContext();
  return (
    <div className="flex flex-col gap-2">
      <p className="text-xs text-baseTextMedEmphasis">Quantity</p>
      <div className="relative">
        <input
          placeholder="0"
          className="h-12 w-[280px] pr-12 text-right text-2xl border-2 rounded-lg bg-[var(--background)]"
          step={0.01}
          inputMode="numeric"
          type="number"
          value={quantity}
          onChange={(e) => setQuantity(Number(e.target.value))}
        />
        <div className="absolute right-1 top-1 flex items-center p-2">
          <img
            loading="lazy"
            width="24"
            height="24"
            className="rounded-full"
            src={`https://backpack.exchange/_next/image?url=%2Fcoins%2F${asset.toLowerCase()}.png&w=48&q=75`}
          />
        </div>
      </div>
    </div>
  );
};

const OrderEstimation = () => {
  const { price, quantity } = useOrderContext();
  const value = price * quantity;

  return (
    <div className="flex justify-end">
      <p className="text-xs font-medium text-baseTextMedEmphasis pr-2">
        ≈ {value.toFixed(2)} USDC
      </p>
    </div>
  );
};
