"use client";
import { getAssetsLtsPrice, getDepth } from "@repo/api/src/cex";
import { Price, Quantity } from "@repo/api/src/cex/types";
import { getUser, user_asset_balance } from "@repo/api/src/cex/user";
import {
  createContext,
  useState,
  useContext,
  Dispatch,
  SetStateAction,
} from "react";

interface OrderContextType {
  isBid: boolean;
  setIsBid: Dispatch<SetStateAction<boolean>>;
  isLimit: boolean;
  setIsLimit: Dispatch<SetStateAction<boolean>>;
  price: Price;
  setPrice: Dispatch<SetStateAction<Price>>;
  quantity: Quantity;
  setQuantity: Dispatch<SetStateAction<Quantity>>;
  marketOrderQuantity: Quantity | Price;
  setMarketOrderQuantity: Dispatch<SetStateAction<Quantity | Price>>;
}

export const OrderContext = createContext<OrderContextType>(
  {} as OrderContextType
);

interface OrderProviderProps {
  children: React.ReactNode;
}
export const OrderProvider = ({ children }: OrderProviderProps) => {
  const ltsPrice = getAssetsLtsPrice("SOL");
  const [isBid, setIsBid] = useState(true);
  const [isLimit, setIsLimit] = useState(true);
  const [price, setPrice] = useState(ltsPrice);
  const [quantity, setQuantity] = useState(0);
  const [marketOrderQuantity, setMarketOrderQuantity] = useState(0);
  return (
    <OrderContext.Provider
      value={{
        isBid,
        setIsBid,
        isLimit,
        setIsLimit,
        price,
        setPrice,
        quantity,
        setQuantity,
        marketOrderQuantity,
        setMarketOrderQuantity,
      }}
    >
      {children}
    </OrderContext.Provider>
  );
};

export const useOrderContext = () => {
  const context = useContext(OrderContext);
  if (context === undefined) {
    throw new Error("use useOrderContext hook within OrderContextProvider");
  }
  return context;
};
