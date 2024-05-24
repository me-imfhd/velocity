"use client";
import React, { useState } from "react";
import { Button } from "@repo/ui/components";
import { getUser } from "@repo/api/src/cex/fill-order";
import { Asset, UserId } from "@repo/api/src/cex/types";
import { Orderbook } from "../components/orderbook-components";

export default function Page() {
  return (
    <div className="flex h-screen w-full items-center justify-center gap-10 p-4">
      <Orderbook />
      <OrderComponent />
    </div>
  );
}

const OrderComponent = () => {
  const asset: Asset = "SOL";
  const [isBid, setIsBid] = useState(true);

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
      <OrderForm isBid={isBid} asset={asset} />
    </div>
  );
};

interface OrderFormProps {
  isBid: boolean;
  asset: Asset;
}

const OrderForm = ({ isBid, asset }: OrderFormProps) => {
  const [isLimit, setIsLimit] = useState(true);
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
          className="flex justify-center items-center cursor-pointer py-2"
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
        <div className="flex flex-col">
          <OrderBalance userId="1" asset={isBid ? "USDC" : asset} />
          <OrderInput label="Price" currency="USDC" />
          <OrderInput label="Quantity" currency={asset} />
        </div>
      ) : (
        <div className="flex flex-col">
          <p
            onClick={() => setOrderValueIsUSDC(!orderValueIsUSDC)}
            className="w-max text-start cursor-pointer text-sm hover:bg-transparent hover:text-muted-foreground/80 text-muted-foreground"
          >
            Change Order Value
          </p>
          <OrderBalance userId="1" asset={orderValueIsUSDC ? "USDC" : asset} />
          <OrderInput currency={orderValueIsUSDC ? "USDC" : asset} />
        </div>
      )}
      {isLimit && <OrderEstimation price={0} quantity={0} />}
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
  userId: UserId;
  asset: Asset;
}

const OrderBalance = ({ userId, asset }: OrderBalanceProps) => {
  const balance = getUser(userId).assets.get(asset);
  return (
    <div className="flex justify-between py-2">
      <p className="text-xs">Available Balance</p>
      <p className="text-xs font-medium">
        {balance?.toFixed(2)} {asset}
      </p>
    </div>
  );
};

interface OrderInputProps {
  label?: string;
  currency: string;
}

const OrderInput = ({ label, currency }: OrderInputProps) => {
  return (
    <div className="flex flex-col gap-2">
      {label && <p className="text-xs text-baseTextMedEmphasis">{label}</p>}
      <div className="relative">
        <input
          placeholder="0"
          className="h-12 w-[280px] pr-12 text-right text-2xl border-2 rounded-lg bg-[var(--background)]"
          step={0.01}
          inputMode="numeric"
          type="number"
        />
        <div className="absolute right-1 top-1 flex items-center p-2">
          <img
            alt={currency}
            loading="lazy"
            width="24"
            height="24"
            className="rounded-full"
            src={`https://backpack.exchange/_next/image?url=%2Fcoins%2F${currency.toLowerCase()}.png&w=48&q=75`}
          />
        </div>
      </div>
    </div>
  );
};

interface OrderEstimationProps {
  price: number;
  quantity: number;
}

const OrderEstimation = ({ price, quantity }: OrderEstimationProps) => {
  return (
    <div className="flex justify-end">
      <p className="text-xs font-medium text-baseTextMedEmphasis pr-2">
        ≈ {quantity * price} USDC
      </p>
    </div>
  );
};
