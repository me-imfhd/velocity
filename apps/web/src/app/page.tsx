import React from "react";
import { Button } from "@repo/ui/components";
import { checkAuth } from "@repo/auth/server";

export default async function Page() {
  await checkAuth();
  return (
    <div className="flex h-screen w-full items-center justify-evenly p-4">
      <Orderbook />
      <OrderComponent />
    </div>
  );
}

const Orderbook = () => {
  return (
    <div className="flex flex-col w-[250px] overflow-hidden border rounded-lg">
      <OrderbookHeader />
      <OrderbookContent />
    </div>
  );
};

const OrderbookHeader = () => {
  return (
    <div className="flex items-center justify-between border-b px-3 py-2 text-xs text-baseTextMedEmphasis">
      <p className="font-medium text-baseTextHighEmphasis">Price (USDC)</p>
      <p className="font-medium text-right">Total (SOL)</p>
    </div>
  );
};

const OrderbookContent = () => {
  return (
    <div className="flex flex-col grow overflow-y-hidden">
      <div className="flex flex-col h-full grow overflow-x-hidden">
        <div className="flex flex-col h-full grow overflow-y-auto font-sans no-scrollbar snap-y snap-mandatory">
          <OrderbookEntry price="171.76" total="56.85" type="ask" />
          <OrderbookMidpoint price="171.75" />
          <OrderbookEntry price="171.75" total="0.02" type="bid" />
        </div>
      </div>
    </div>
  );
};

interface OrderbookEntryProps {
  price: string;
  total: string;
  type: "ask" | "bid";
}

const OrderbookEntry = ({ price, total, type }: OrderbookEntryProps) => {
  const textColor = type === "ask" ? "text-redText/90" : "text-greenText/90";

  return (
    <div className="flex h-[25px] items-center">
      <button type="button" className="h-full w-full">
        <div className="flex items-center justify-between px-3 h-full w-full overflow-hidden hover:border-dashed hover:border-baseBorderFocus/50">
          <p
            className={`z-10 text-left text-xs font-normal tabular-nums ${textColor}`}
          >
            {price}
          </p>
          <p className="z-10 text-right text-xs font-normal tabular-nums text-baseTextHighEmphasis/80">
            {total}
          </p>
        </div>
      </button>
    </div>
  );
};

interface OrderbookMidpointProps {
  price: string;
}

const OrderbookMidpoint = ({ price }: OrderbookMidpointProps) => {
  return (
    <div className="flex flex-col snap-center px-3 py-1">
      <div className="flex justify-between">
        <p className="font-medium tabular-nums text-redText">{price}</p>
      </div>
    </div>
  );
};

const OrderComponent = () => {
  return (
    <div className="flex flex-col w-[300px] border rounded-lg">
      <OrderTabs />
      <OrderForm />
    </div>
  );
};

const OrderTabs = () => {
  return (
    <div className="flex h-[60px]">
      <OrderTab label="Buy" active={true} />
      <OrderTab label="Sell" active={false} />
    </div>
  );
};

interface OrderTabProps {
  label: string;
  active: boolean;
}

const OrderTab = ({ label, active }: OrderTabProps) => {
  const activeClass = active
    ? "bg-green-950 border-b-green-800 text-green-400"
    : "border-b-red-950 text-red-400 hover:border-b-red-800 hover:text-red-300";

  return (
    <div
      className={`flex-1 flex justify-center items-center cursor-pointer border-b-2 p-4 ${activeClass}`}
    >
      <p className="text-sm font-semibold">{label}</p>
    </div>
  );
};

const OrderForm = () => {
  return (
    <div className="flex flex-col gap-1 px-3 w-[300px]">
      <OrderTypeSelector />
      <OrderBalance />
      <OrderInput label="Price" currency="USDC" />
      <OrderInput label="Quantity" currency="SOL" />
      <OrderEstimation />
      <Button variant="ghost" size="lg" className="my-2">
        Sign up to trade
      </Button>
    </div>
  );
};

const OrderTypeSelector = () => {
  return (
    <div className="flex gap-5 mb-3">
      <OrderTypeOption label="Limit" active={true} />
      <OrderTypeOption label="Market" active={false} />
    </div>
  );
};

interface OrderTypeOptionProps {
  label: string;
  active: boolean;
}

const OrderTypeOption = ({ label, active }: OrderTypeOptionProps) => {
  const activeClass = active
    ? "border-b-blue-500 text-white"
    : "border-transparent text-muted-foreground hover:border-white hover:text-white";

  return (
    <div
      className={`flex justify-center items-center cursor-pointer py-2 ${activeClass}`}
    >
      <p
        className={`text-sm font-medium py-1 hover:text-white border-b-2 hover:border-white ${
          active && "border-b-white"
        }`}
      >
        {label}
      </p>
    </div>
  );
};

const OrderBalance = () => {
  return (
    <div className="flex justify-between pb-2">
      <p className="text-xs text-baseTextMedEmphasis">Available Balance</p>
      <p className="text-xs font-medium text-baseTextHighEmphasis">0.00 USDC</p>
    </div>
  );
};

interface OrderInputProps {
  label: string;
  currency: string;
}

const OrderInput = ({ label, currency }: OrderInputProps) => {
  return (
    <div className="flex flex-col gap-2">
      <p className="text-xs text-baseTextMedEmphasis">{label}</p>
      <div className="relative">
        <input
          step="0.01"
          placeholder="0"
          className="h-12 w-[280px] pr-12 text-right text-2xl border-2 rounded-lg bg-[var(--background)]"
          type="text"
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

const OrderEstimation = () => {
  return (
    <div className="flex justify-end">
      <p className="text-xs font-medium text-baseTextMedEmphasis pr-2">
        ≈ 0.00 USDC
      </p>
    </div>
  );
};
